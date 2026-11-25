//! Per-post AI briefing that steers Funny, Smart, and Spicy takes.
//!
//! The briefing is cached by post id so a second tone click does not redo
//! the analysis. A failed analysis is skipped; nothing is invented.

use std::cell::RefCell;
use std::collections::HashMap;
use std::sync::Mutex;

use super::domain::SourcePost;

thread_local! {
    static ACTIVE: RefCell<Option<String>> = const { RefCell::new(None) };
}

static CACHE: Mutex<Option<HashMap<String, PostAnalysis>>> = Mutex::new(None);

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct PostAnalysis {
    pub why_popular: String,
    pub topic: String,
    pub hook: String,
    pub tension: String,
    pub replies: String,
    pub open_angle: String,
    pub audience: String,
    pub why_popping: String,
    pub hooks: Vec<u8>,
}

impl PostAnalysis {
    pub fn is_empty(&self) -> bool {
        self.why_popping.is_empty()
            && self.topic.is_empty()
            && self.hook.is_empty()
            && self.tension.is_empty()
    }

    /// Compact briefing folded into the take prompt.
    pub fn as_prompt_block(&self) -> String {
        let popping = if self.why_popping.is_empty() {
            self.why_popular.as_str()
        } else {
            self.why_popping.as_str()
        };
        let mut block = format!(
            "Post briefing (use this; do not ignore it): Why it's popping: {popping} Topic: {} Hook: {} Tension: {} Replies argue: {} Open angle: {} Audience: {} Funny should exploit this post's absurdity or irony. Smart should add the non-obvious insight or data angle. Spicy should take the contrarian side of the actual tension.",
            self.topic, self.hook, self.tension, self.replies, self.open_angle, self.audience
        );
        if !self.hooks.is_empty() {
            let names: Vec<String> = self
                .hooks
                .iter()
                .copied()
                .filter_map(super::viral_hooks::hook_by_id)
                .map(|hook| format!("{:02} {}", hook.id, hook.name))
                .collect();
            block.push_str(" Preferred hooks: ");
            block.push_str(&names.join(", "));
            block.push('.');
        }
        block
    }
}

/// One-line note for the pink box. Empty when analysis did not run.
pub fn why_popping_line(analysis: &PostAnalysis) -> String {
    let line = if !analysis.why_popping.is_empty() {
        analysis.why_popping.trim()
    } else {
        analysis.why_popular.trim()
    };
    if line.is_empty() {
        String::new()
    } else {
        format!("Why it's popping: {line}")
    }
}

pub fn cached_analysis(post_id: &str) -> Option<PostAnalysis> {
    let cache = CACHE.lock().ok()?;
    cache.as_ref()?.get(post_id).cloned()
}

pub fn cached_why_popping(post_id: &str) -> String {
    cached_analysis(post_id)
        .map(|analysis| why_popping_line(&analysis))
        .unwrap_or_default()
}

fn cache_put(post_id: &str, analysis: PostAnalysis) {
    if let Ok(mut cache) = CACHE.lock() {
        cache
            .get_or_insert_with(HashMap::new)
            .insert(post_id.to_string(), analysis);
    }
}

/// Analyze the post once, then reuse the cached briefing on later clicks.
/// `run` is the AI call. If it fails or returns nothing usable, this returns
/// `None` and the take still runs on the tone guidance alone.
pub fn steer_post(
    post: &SourcePost,
    run: impl FnOnce(&str) -> Result<String, String>,
) -> Option<PostAnalysis> {
    if let Some(hit) = cached_analysis(&post.id) {
        return Some(hit);
    }
    let raw = run(&analysis_prompt(post)).ok()?;
    let analysis = parse_analysis(&raw)?;
    if analysis.is_empty() {
        return None;
    }
    cache_put(&post.id, analysis.clone());
    Some(analysis)
}

pub struct AnalysisGuard;

impl Drop for AnalysisGuard {
    fn drop(&mut self) {
        ACTIVE.with(|slot| slot.borrow_mut().take());
    }
}

/// Bind a briefing for the current take thread so prompt builders can see it.
pub fn bind_take_analysis(analysis: Option<&PostAnalysis>) -> AnalysisGuard {
    ACTIVE.with(|slot| {
        *slot.borrow_mut() = analysis
            .filter(|analysis| !analysis.is_empty())
            .map(PostAnalysis::as_prompt_block);
    });
    AnalysisGuard
}

pub fn active_analysis_block() -> Option<String> {
    ACTIVE.with(|slot| slot.borrow().clone())
}

pub fn flatten_one_line(text: &str) -> String {
    text.split_whitespace().collect::<Vec<_>>().join(" ")
}

pub fn analysis_prompt(post: &SourcePost) -> String {
    let author = post.author.trim().trim_start_matches('@');
    let tweet = flatten_one_line(&post.text);
    format!(
        "Analyze this X post for a reply writer. Use only what is in the post and the real engagement counts. Do not invent numbers or quote replies you were not given. If reply text is missing, infer the likely argument from the post and mark it inferred. Reply with exactly these labels, one per line, short phrases:\nWHY: why it is popular (engagement drivers)\nTOPIC: the topic\nHOOK: the emotional hook\nTENSION: the controversy or tension\nREPLIES: what replies argue about\nOPEN: the angle nobody has taken yet\nAUDIENCE: who it is for\nPOPPING: one line on why it is popping now\nHOOKS: two or three hook ids from this catalog that best fit the post, like 21, 4, 43\nHook catalog: {}\nAuthor: @{author}\nEngagement (real counts): {} likes, {} reposts, {} replies, {} views\nPost: {tweet}",
        super::viral_hooks::hook_catalog_brief(),
        post.likes,
        post.reposts,
        post.replies,
        post.views
    )
}

pub fn parse_analysis(raw: &str) -> Option<PostAnalysis> {
    let mut analysis = PostAnalysis::default();
    for line in raw.lines() {
        let line = line.trim().trim_start_matches(['-', '*', '•']).trim();
        let (key, value) = line.split_once(':').unwrap_or(("", line));
        let value = value.trim().to_string();
        if value.is_empty() {
            continue;
        }
        match key.trim().to_ascii_uppercase().as_str() {
            "WHY" => analysis.why_popular = value,
            "TOPIC" => analysis.topic = value,
            "HOOK" => analysis.hook = value,
            "TENSION" => analysis.tension = value,
            "REPLIES" => analysis.replies = value,
            "OPEN" => analysis.open_angle = value,
            "AUDIENCE" => analysis.audience = value,
            "POPPING" => analysis.why_popping = value,
            "HOOKS" => analysis.hooks = super::viral_hooks::parse_hook_ids(&value),
            _ => {}
        }
    }
    if analysis.is_empty() {
        None
    } else {
        if analysis.why_popping.is_empty() {
            analysis.why_popping = analysis.why_popular.clone();
        }
        Some(analysis)
    }
}

/// History rows with the same text shown only once, first kept.
pub fn unique_takes_by_text<T, F>(items: &[T], text: F) -> Vec<usize>
where
    F: Fn(&T) -> &str,
{
    let mut seen = std::collections::HashSet::new();
    let mut keep = Vec::new();
    for (index, item) in items.iter().enumerate() {
        let key = text(item).trim().to_string();
        if key.is_empty() || !seen.insert(key) {
            continue;
        }
        keep.push(index);
    }
    keep
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};

    fn post() -> SourcePost {
        SourcePost {
            id: "p1".into(),
            author: "ada".into(),
            text: "The court commuted the sentence after the filing went viral.".into(),
            likes: 1200,
            reposts: 80,
            replies: 40,
            views: 9000,
            ..SourcePost::default()
        }
    }

    #[test]
    fn analysis_prompt_uses_real_counts_and_asks_for_the_briefing() {
        let prompt = analysis_prompt(&post());
        assert!(prompt.contains("1200 likes"));
        assert!(prompt.contains("80 reposts"));
        assert!(prompt.contains("commuted the sentence"));
        assert!(prompt.contains("WHY:"));
        assert!(prompt.contains("OPEN:"));
        assert!(prompt.contains("HOOKS:"));
        assert!(prompt.contains("01 From-to result"));
        assert!(prompt.contains("Do not invent numbers"));
    }

    #[test]
    fn parse_analysis_reads_labels_and_builds_a_popping_line() {
        let parsed = parse_analysis(
            "WHY: outrage plus a rare official reversal\nTOPIC: a commuted sentence\nHOOK: relief after a public fight\nTENSION: mercy vs the original verdict\nREPLIES: inferred: was justice served\nOPEN: what the filing actually changed\nAUDIENCE: people following the case\nPOPPING: a rare reversal after a viral filing\nHOOKS: 21, 4, 43\n",
        )
        .unwrap();
        assert_eq!(parsed.topic, "a commuted sentence");
        assert_eq!(parsed.hooks, vec![21, 4, 43]);
        assert!(parsed.open_angle.contains("filing"));
        assert!(why_popping_line(&parsed).starts_with("Why it's popping:"));
        assert!(parsed.as_prompt_block().contains("Open angle:"));
        assert!(parsed.as_prompt_block().contains("absurdity or irony"));
        assert!(
            parsed
                .as_prompt_block()
                .contains("Preferred hooks: 21 X is dead, meanwhile")
        );
    }

    #[test]
    fn garbage_or_empty_analysis_is_not_faked() {
        assert!(parse_analysis("sorry, I cannot help").is_none());
        assert!(parse_analysis("").is_none());
    }

    #[test]
    fn steer_post_caches_and_does_not_rerun_on_a_second_click() {
        let calls = AtomicUsize::new(0);
        let once = |raw: &str| {
            calls.fetch_add(1, Ordering::SeqCst);
            Ok(raw.to_string())
        };
        let mut sample = post();
        sample.id = "cache-test-steer-post-only".into();
        let first = steer_post(&sample, |_| {
            once(
                "WHY: it is spreading on the reversal\nTOPIC: courts\nHOOK: shock\nTENSION: the ruling\nREPLIES: inferred: fairness\nOPEN: the paperwork\nAUDIENCE: law watchers\nPOPPING: a viral reversal\n",
            )
        })
        .unwrap();
        let second = steer_post(&sample, |_| {
            once("WHY: should not be used\nTOPIC: no\nHOOK: no\nTENSION: no\nREPLIES: no\nOPEN: no\nAUDIENCE: no\nPOPPING: no\n")
        })
        .unwrap();
        assert_eq!(calls.load(Ordering::SeqCst), 1);
        assert_eq!(first, second);
        assert_eq!(second.why_popping, "a viral reversal");
    }

    #[test]
    fn bind_puts_the_briefing_in_the_active_prompt_and_clears_after() {
        let analysis = parse_analysis(
            "WHY: buzz\nTOPIC: rates\nHOOK: fear\nTENSION: the cut\nREPLIES: inferred: inflation\nOPEN: who got hurt\nAUDIENCE: traders\nPOPPING: a surprise cut\n",
        )
        .unwrap();
        {
            let _guard = bind_take_analysis(Some(&analysis));
            let block = active_analysis_block().unwrap();
            assert!(block.contains("surprise cut"));
            assert!(block.contains("contrarian side"));
        }
        assert!(active_analysis_block().is_none());
    }

    #[test]
    fn unique_takes_drop_duplicate_text_and_keep_the_first() {
        let rows = ["one", "two", "one", "two", "three"];
        let keep = unique_takes_by_text(&rows, |row| *row);
        assert_eq!(keep, vec![0, 1, 4]);
    }
}
