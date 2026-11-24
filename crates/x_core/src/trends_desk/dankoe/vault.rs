//! Dan Koe step 2: save posts that got 2x+ their author's usual engagement,
//! 20-30 of them, each with 3-5 reasons it worked. The "usual" is the median
//! of that author's other posts the desk has seen. With fewer than two of
//! those, the column's median stands in and the label says so.

use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use super::hook::{HookPart, hook_check};
use crate::trends_desk::SourcePost;

pub const VAULT_FILE: &str = "viral_vault.json";
pub const VAULT_MAX: usize = 30;
pub const VAULT_MIN_TARGET: usize = 20;
pub const VIRAL_MULTIPLE: f64 = 2.0;
const MIN_SCORE: u64 = 10;
const AUTHOR_SAMPLES: usize = 2;
const COLUMN_SAMPLES: usize = 4;

/// Likes + 2 x reposts + replies. Views are left out: OpenCLI does not
/// always return them.
pub fn engagement_score(post: &SourcePost) -> u64 {
    post.likes
        .saturating_add(post.reposts.saturating_mul(2))
        .saturating_add(post.replies)
}

#[derive(Debug, Clone, PartialEq)]
pub enum Baseline {
    Author { median: f64, samples: usize },
    Column { median: f64, samples: usize },
}

#[derive(Debug, Clone, PartialEq)]
pub struct ViralFlag {
    pub post_id: String,
    pub score: u64,
    pub multiple: f64,
    pub baseline: Baseline,
}

fn handle(author: &str) -> String {
    author.trim().trim_start_matches('@').to_lowercase()
}

fn median(mut values: Vec<u64>) -> f64 {
    values.sort_unstable();
    let n = values.len();
    if n == 0 {
        return 0.0;
    }
    if n % 2 == 1 {
        values[n / 2] as f64
    } else {
        (values[n / 2 - 1] + values[n / 2]) as f64 / 2.0
    }
}

impl ViralFlag {
    /// "2.4x @dankoe's usual (median of 5 of their posts)".
    pub fn label(&self, author: &str) -> String {
        match &self.baseline {
            Baseline::Author { median, samples } => format!(
                "{:.1}x @{}'s usual ({} vs median {:.0} of {samples} of their posts)",
                self.multiple,
                handle(author),
                self.score,
                median
            ),
            Baseline::Column { median, samples } => format!(
                "{:.1}x this column's usual ({} vs median {:.0} of {samples} posts; too few of @{}'s posts seen)",
                self.multiple,
                self.score,
                median,
                handle(author)
            ),
        }
    }

    pub fn short(&self) -> String {
        match self.baseline {
            Baseline::Author { .. } => format!("VIRAL {:.1}x author's usual", self.multiple),
            Baseline::Column { .. } => format!("VIRAL {:.1}x column's usual", self.multiple),
        }
    }
}

/// Flag `post` when it beat its baseline by 2x or more.
pub fn viral_flag(
    post: &SourcePost,
    seen: &[&SourcePost],
    column: &[&SourcePost],
) -> Option<ViralFlag> {
    let score = engagement_score(post);
    if score < MIN_SCORE {
        return None;
    }
    let who = handle(&post.author);
    let mut ids = HashSet::new();
    let author_scores: Vec<u64> = seen
        .iter()
        .filter(|other| other.id != post.id && handle(&other.author) == who)
        .filter(|other| ids.insert(other.id.clone()))
        .map(|other| engagement_score(other))
        .collect();
    let baseline = if author_scores.len() >= AUTHOR_SAMPLES {
        Baseline::Author {
            samples: author_scores.len(),
            median: median(author_scores),
        }
    } else {
        let mut ids = HashSet::new();
        let column_scores: Vec<u64> = column
            .iter()
            .filter(|other| other.id != post.id)
            .filter(|other| ids.insert(other.id.clone()))
            .map(|other| engagement_score(other))
            .collect();
        if column_scores.len() < COLUMN_SAMPLES {
            return None;
        }
        Baseline::Column {
            samples: column_scores.len(),
            median: median(column_scores),
        }
    };
    let base = match &baseline {
        Baseline::Author { median, .. } | Baseline::Column { median, .. } => median.max(1.0),
    };
    let multiple = score as f64 / base;
    (multiple >= VIRAL_MULTIPLE).then(|| ViralFlag {
        post_id: post.id.clone(),
        score,
        multiple,
        baseline,
    })
}

/// Flags for every column post. `history` adds older posts from saved
/// reports, so authors build a baseline over time.
pub fn viral_flags(
    columns: &[&[SourcePost]],
    history: &[SourcePost],
) -> HashMap<String, ViralFlag> {
    let mut ids = HashSet::new();
    let seen: Vec<&SourcePost> = columns
        .iter()
        .flat_map(|posts| posts.iter())
        .chain(history.iter())
        .filter(|post| ids.insert(post.id.clone()))
        .collect();
    let mut flags = HashMap::new();
    for posts in columns {
        let column: Vec<&SourcePost> = posts.iter().collect();
        for post in *posts {
            if let Some(flag) = viral_flag(post, &seen, &column) {
                flags.insert(post.id.clone(), flag);
            }
        }
    }
    flags
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct VaultEntry {
    pub id: String,
    pub author: String,
    #[serde(default)]
    pub author_name: String,
    pub text: String,
    pub url: String,
    pub likes: u64,
    pub reposts: u64,
    pub replies: u64,
    pub views: u64,
    /// Empty when saved by hand without a flag.
    #[serde(default)]
    pub why_flagged: String,
    #[serde(default)]
    pub reasons: Vec<String>,
    /// "rules" for the local read, or the model that wrote them.
    #[serde(default)]
    pub reasons_by: String,
    pub saved_at: i64,
}

impl VaultEntry {
    pub fn from_post(post: &SourcePost, flag: Option<&ViralFlag>, now: i64) -> Self {
        Self {
            id: post.id.clone(),
            author: post.author.clone(),
            author_name: post.author_name.clone(),
            text: post.text.clone(),
            url: post.url.clone(),
            likes: post.likes,
            reposts: post.reposts,
            replies: post.replies,
            views: post.views,
            why_flagged: flag
                .map(|flag| flag.label(&post.author))
                .unwrap_or_default(),
            reasons: rule_reasons(post, flag),
            reasons_by: "rules".into(),
            saved_at: now,
        }
    }

    pub fn as_post(&self) -> SourcePost {
        SourcePost {
            id: self.id.clone(),
            author: self.author.clone(),
            text: self.text.clone(),
            url: self.url.clone(),
            likes: self.likes,
            reposts: self.reposts,
            replies: self.replies,
            views: self.views,
            author_name: self.author_name.clone(),
            ..SourcePost::default()
        }
    }

    pub fn copy_text(&self) -> String {
        let mut out = format!("@{} · {}\n{}\n", handle(&self.author), self.url, self.text);
        if !self.why_flagged.is_empty() {
            out.push_str(&format!("Flagged: {}\n", self.why_flagged));
        }
        out.push_str(&format!("Why it worked ({}):\n", self.reasons_by));
        for reason in &self.reasons {
            out.push_str(&format!("- {reason}\n"));
        }
        out
    }
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct ViralVault {
    #[serde(default)]
    pub entries: Vec<VaultEntry>,
}

impl ViralVault {
    pub fn contains(&self, id: &str, url: &str) -> bool {
        self.entries
            .iter()
            .any(|entry| entry.id == id || (!url.is_empty() && entry.url == url))
    }

    pub fn add(&mut self, entry: VaultEntry) -> Result<usize, String> {
        if self.contains(&entry.id, &entry.url) {
            return Err("Already in the Viral Vault.".into());
        }
        if self.entries.len() >= VAULT_MAX {
            return Err(format!(
                "The vault holds {VAULT_MAX}. Remove one before saving another."
            ));
        }
        self.entries.push(entry);
        Ok(self.entries.len())
    }

    pub fn remove(&mut self, id: &str) -> bool {
        let before = self.entries.len();
        self.entries.retain(|entry| entry.id != id);
        before != self.entries.len()
    }

    pub fn set_reasons(&mut self, id: &str, reasons: Vec<String>, by: &str) -> bool {
        match self.entries.iter_mut().find(|entry| entry.id == id) {
            Some(entry) => {
                entry.reasons = reasons;
                entry.reasons_by = by.to_string();
                true
            }
            None => false,
        }
    }
}

pub fn vault_status_line(vault: &ViralVault) -> String {
    let count = vault.entries.len();
    if count >= VAULT_MAX {
        format!("{count} saved · the vault is full")
    } else if count >= VAULT_MIN_TARGET {
        format!("{count} saved · goal {VAULT_MIN_TARGET}-{VAULT_MAX} reached")
    } else {
        format!(
            "{count} saved · goal {VAULT_MIN_TARGET}-{VAULT_MAX} ({} to go)",
            VAULT_MIN_TARGET - count
        )
    }
}

fn clip(text: &str, max: usize) -> String {
    let text = text.split_whitespace().collect::<Vec<_>>().join(" ");
    if text.chars().count() <= max {
        text
    } else {
        format!("{}…", text.chars().take(max - 1).collect::<String>())
    }
}

/// 3-5 reasons read from the post's own words and numbers. No model call.
pub fn rule_reasons(post: &SourcePost, flag: Option<&ViralFlag>) -> Vec<String> {
    let mut reasons = Vec::new();
    if let Some(flag) = flag {
        reasons.push(format!("Pulled {}.", flag.label(&post.author)));
    }
    let check = hook_check(&post.text);
    for item in check.items.iter().filter(|item| item.hit) {
        let line = match item.part {
            HookPart::Hook => {
                let first = post
                    .text
                    .lines()
                    .map(str::trim)
                    .find(|line| !line.is_empty())
                    .unwrap_or("");
                format!(
                    "The first line hooks: \"{}\" ({}).",
                    clip(first, 80),
                    item.why
                )
            }
            HookPart::Problem => format!("Names a pain readers feel (\"{}\").", item.why),
            HookPart::Solution => "Shares a first-hand fix, not theory.".into(),
            HookPart::Benefit => format!("Makes the payoff plain (\"{}\").", item.why),
            HookPart::Side => format!(
                "Takes a clear side (\"{}\"), which pulls replies.",
                item.why
            ),
            HookPart::Angle => format!("Offers a fresh angle (\"{}\").", item.why),
        };
        reasons.push(line);
    }
    let lines = post
        .text
        .lines()
        .filter(|line| !line.trim().is_empty())
        .count();
    let words = post.text.split_whitespace().count();
    if lines >= 4 {
        reasons.push(format!("Easy to scan: {lines} short lines."));
    } else if words > 0 && words <= 25 {
        reasons.push(format!("Short: {words} words, read in seconds."));
    }
    if post.replies >= 5 && post.replies.saturating_mul(4) >= post.likes {
        reasons.push(format!(
            "Started a conversation: {} replies to {} likes.",
            post.replies, post.likes
        ));
    }
    if !post.media.is_empty() || !post.media_items.is_empty() {
        reasons.push("Carries media, which stops the scroll.".into());
    }
    if post.quoted.is_some() {
        reasons.push("Quotes another post and borrows its reach.".into());
    }
    if reasons.len() < 3 {
        reasons.push(format!(
            "Came from @{}, so it reached an audience that already follows them.",
            handle(&post.author)
        ));
    }
    if reasons.len() < 3 {
        reasons.push("Showed up in this column's search, so the topic was live that day.".into());
    }
    reasons.truncate(5);
    reasons
}

/// Prompt for a model to write the 3-5 reasons.
pub fn vault_reasons_prompt(entry: &VaultEntry) -> String {
    let flagged = if entry.why_flagged.is_empty() {
        String::new()
    } else {
        format!("It got {}.\n", entry.why_flagged)
    };
    format!(
        "This X post did unusually well.\nAuthor: @{}\nLikes {} · Reposts {} · Replies {} · Views {}\n{flagged}Post:\n{}\n\nWrite 3 to 5 short reasons why it worked, so a writer can copy the pattern. Think about the hook in the first line, a problem people relate to, a solution from the author's own experience, a clear benefit, picking a side, and a new way of seeing the idea. One reason per line, each starting with \"- \". Under 20 words each. Output only the reasons.",
        handle(&entry.author),
        entry.likes,
        entry.reposts,
        entry.replies,
        entry.views,
        clip(&entry.text, 1200)
    )
}

fn strip_bullet(line: &str) -> String {
    let line = line.trim().trim_start_matches(['-', '*', '•', '–']).trim();
    let digits = line.chars().take_while(char::is_ascii_digit).count();
    let rest = &line[digits..];
    let line = if digits > 0 && (rest.starts_with('.') || rest.starts_with(')')) {
        &rest[1..]
    } else {
        line
    };
    line.trim().to_string()
}

/// Pull 3-5 reason lines out of a model reply.
pub fn parse_ai_reasons(text: &str) -> Result<Vec<String>, String> {
    let reasons: Vec<String> = text
        .lines()
        .map(strip_bullet)
        .filter(|line| line.chars().count() >= 4)
        .take(5)
        .collect();
    if reasons.len() < 3 {
        Err(format!(
            "The model gave {} usable reason(s); kept the rule-based ones.",
            reasons.len()
        ))
    } else {
        Ok(reasons)
    }
}

pub fn vault_file(data_root: &Path) -> PathBuf {
    data_root.join(VAULT_FILE)
}

pub fn load_vault(data_root: &Path) -> ViralVault {
    std::fs::read_to_string(vault_file(data_root))
        .ok()
        .and_then(|text| serde_json::from_str(&text).ok())
        .unwrap_or_default()
}

pub fn save_vault(data_root: &Path, vault: &ViralVault) -> Result<(), String> {
    std::fs::create_dir_all(data_root).map_err(|error| error.to_string())?;
    let text = serde_json::to_string_pretty(vault).map_err(|error| error.to_string())?;
    std::fs::write(vault_file(data_root), text).map_err(|error| error.to_string())
}
