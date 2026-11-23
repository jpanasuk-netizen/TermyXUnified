//! Tweet cards, inline reply state, and TermyXL take selection.
//!
//! Sending is a separate explicit step. Drafting a take never posts.
//! A failed provider stays an error and does not invent a successful draft.

use crate::io::Http;
#[cfg(test)]
use crate::io::HttpRequest;
use crate::reply_guy::{self, Secret};

use super::domain::SourcePost;

/// Windows Termy opens at 1280×820. Termy XL sits beside it at twice the width
/// and the same height. The window stays resizable.
pub const COMPANION_WIDTH: f32 = 2560.0;
pub const COMPANION_HEIGHT: f32 = 820.0;
/// Slim dock that sits beside Termy X until XL is opened.
pub const DOCK_WIDTH: f32 = 72.0;
/// Logical width Jeremy measured on the 125% display (843 physical px).
/// Launch and re-expand always use this. A saved width does not replace it.
pub const LOCKED_OPEN_WIDTH: f32 = 687.0;
/// Nominal take-card width. Wider windows add columns instead of stretching one card.
pub const TAKE_CARD_WIDTH: f32 = 480.0;
pub const TAKE_CARD_MAX: f32 = 520.0;
/// Inset between the window edge and a take card's outer border.
pub const WINDOW_PAD: f32 = 16.0;
/// Pink selected-card border. Cyan is the selected text.
pub const SELECT_PINK: u32 = 0xff4fa3;
pub const SELECT_CYAN: u32 = 0x7af6ff;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct DeskFrame {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
    pub expanded: bool,
    pub expanded_width: f32,
}

pub fn frame_from_parts(
    docked: bool,
    x: f32,
    y: f32,
    expanded_width: f32,
    height: f32,
) -> DeskFrame {
    let expanded_width = if expanded_width <= DOCK_WIDTH {
        COMPANION_WIDTH
    } else {
        expanded_width
    };
    let height = if height < 240.0 {
        COMPANION_HEIGHT
    } else {
        height
    };
    DeskFrame {
        x,
        y,
        width: if docked { DOCK_WIDTH } else { expanded_width },
        height,
        expanded: !docked,
        expanded_width,
    }
}

/// Grow to the right. `x` and `y` are the left and top edges.
pub fn expand_right(frame: DeskFrame) -> DeskFrame {
    let width = if frame.expanded_width <= DOCK_WIDTH {
        COMPANION_WIDTH
    } else {
        frame.expanded_width
    };
    DeskFrame {
        x: frame.x,
        y: frame.y,
        width,
        height: frame.height,
        expanded: true,
        expanded_width: width,
    }
}

pub fn toggle_dock(frame: DeskFrame) -> DeskFrame {
    if frame.expanded {
        collapse_dock(frame)
    } else {
        expand_right(frame)
    }
}

/// Preferred width, or the monitor's free width when the preferred width does not fit.
/// A missing measurement never shrinks the window.
pub fn open_width(preferred: f32, available: Option<f32>) -> f32 {
    let full = if preferred <= DOCK_WIDTH {
        COMPANION_WIDTH
    } else {
        preferred
    };
    match available {
        Some(room) if room.is_finite() && room + 0.5 < full && room > DOCK_WIDTH => room,
        _ => full,
    }
}

/// One resize to the full workspace. The left edge stays at `frame.x`.
/// `expanded_width` stays the preferred width even when the monitor clamps the open size.
/// One step to the locked open width. A saved width does not replace it.
pub fn expand_locked(frame: DeskFrame, available: Option<f32>) -> DeskFrame {
    DeskFrame {
        x: frame.x,
        y: frame.y,
        width: open_width(LOCKED_OPEN_WIDTH, available),
        height: frame.height,
        expanded: true,
        expanded_width: LOCKED_OPEN_WIDTH,
    }
}

pub fn expand_to_full(frame: DeskFrame, available: Option<f32>) -> DeskFrame {
    let preferred = if frame.expanded_width <= DOCK_WIDTH {
        COMPANION_WIDTH
    } else {
        frame.expanded_width
    };
    DeskFrame {
        x: frame.x,
        y: frame.y,
        width: open_width(preferred, available),
        height: frame.height,
        expanded: true,
        expanded_width: preferred,
    }
}

/// Keep a user resize. A width that only exists because the monitor was smaller
/// does not replace a larger preferred width.
pub fn remember_width(preferred: f32, current: f32, available: Option<f32>) -> f32 {
    if current <= DOCK_WIDTH + 0.5 {
        return if preferred <= DOCK_WIDTH {
            COMPANION_WIDTH
        } else {
            preferred
        };
    }
    if let Some(room) = available
        && current + 4.0 >= room
        && preferred > current + 4.0
    {
        return preferred;
    }
    current
}

pub fn collapse_dock(frame: DeskFrame) -> DeskFrame {
    let from_live = if frame.width > DOCK_WIDTH {
        frame.width
    } else {
        0.0
    };
    let expanded_width = frame.expanded_width.max(from_live);
    let expanded_width = if expanded_width <= DOCK_WIDTH {
        COMPANION_WIDTH
    } else {
        expanded_width
    };
    DeskFrame {
        x: frame.x,
        y: frame.y,
        width: DOCK_WIDTH,
        height: frame.height,
        expanded: false,
        expanded_width,
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TweetCard {
    pub id: String,
    pub author_name: String,
    pub handle: String,
    pub text: String,
    pub media: Vec<String>,
    pub media_items: Vec<crate::model::MediaItem>,
    pub quoted: Option<super::media::QuotedTweet>,
    pub likes: u64,
    pub reposts: u64,
    pub replies: u64,
    pub views: u64,
    pub created_at: Option<String>,
    pub url: String,
}

pub fn card_from_post(post: &SourcePost) -> TweetCard {
    let handle = if post.author.is_empty() {
        "unknown".into()
    } else {
        post.author.clone()
    };
    let author_name = if post.author_name.trim().is_empty() {
        handle.clone()
    } else {
        post.author_name.clone()
    };
    TweetCard {
        id: post.id.clone(),
        author_name,
        handle,
        text: post.text.clone(),
        media: post.media.clone(),
        media_items: post.media_items.clone(),
        quoted: post.quoted.clone(),
        likes: post.likes,
        reposts: post.reposts,
        replies: post.replies,
        views: post.views,
        created_at: post.created_at.clone(),
        url: post.url.clone(),
    }
}

pub fn card_columns(width: f32) -> usize {
    if width >= 1800.0 { 2 } else { 1 }
}

#[derive(Clone, Debug, PartialEq)]
pub struct TakeFrame {
    pub id: String,
    pub x: f32,
    pub width: f32,
}

impl TakeFrame {
    pub fn right(&self) -> f32 {
        self.x + self.width
    }
}

/// One frame per take. Every card shares the same inset so its right edge
/// stays inside the window at any width.
pub fn layout_takes(viewport: f32, ids: &[impl AsRef<str>]) -> Vec<TakeFrame> {
    let gap = 12.0;
    let inner = (viewport - WINDOW_PAD * 2.0).max(0.0);
    let mut columns = ((inner / TAKE_CARD_WIDTH).floor() as usize).max(1);
    while columns > 1 && columns as f32 * 420.0 + (columns - 1) as f32 * gap > inner {
        columns -= 1;
    }
    let width = if columns == 1 {
        inner
    } else {
        ((inner - gap * (columns as f32 - 1.0)) / columns as f32).min(TAKE_CARD_MAX)
    };
    ids.iter()
        .enumerate()
        .map(|(index, id)| {
            let column = index % columns;
            TakeFrame {
                id: id.as_ref().to_string(),
                x: WINDOW_PAD + column as f32 * (width + gap),
                width,
            }
        })
        .collect()
}

pub fn take_column_count(viewport: f32) -> usize {
    let frames = layout_takes(viewport, &["a"]);
    let _ = frames;
    let gap = 12.0;
    let inner = (viewport - WINDOW_PAD * 2.0).max(0.0);
    let mut columns = ((inner / TAKE_CARD_WIDTH).floor() as usize).max(1);
    while columns > 1 && columns as f32 * 420.0 + (columns - 1) as f32 * gap > inner {
        columns -= 1;
    }
    columns
}

/// One free-template take per saved post. Empty input stays empty. Not a fixture.
pub fn takes_for_posts(posts: &[super::domain::SourcePost]) -> Vec<(String, String)> {
    posts
        .iter()
        .map(|post| (post.id.clone(), super::domain::idea_from(post)))
        .collect()
}

/// Break long tokens so a URL or handle can wrap inside the card.
pub fn wrap_token(text: &str) -> String {
    const LIMIT: usize = 24;
    let mut out = String::new();
    for token in text.split_inclusive(char::is_whitespace) {
        let chars: Vec<char> = token.chars().collect();
        if chars.len() <= LIMIT {
            out.extend(chars);
            continue;
        }
        for (index, ch) in chars.into_iter().enumerate() {
            if index > 0 && index % LIMIT == 0 && !ch.is_whitespace() {
                out.push('\u{200b}');
            }
            out.push(ch);
        }
    }
    out
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SubjectPage {
    pub id: String,
    pub name: String,
}

pub fn subject_pages(pairs: &[(&str, &str)]) -> Vec<SubjectPage> {
    pairs
        .iter()
        .filter(|(id, _)| !id.is_empty())
        .map(|(id, name)| SubjectPage {
            id: (*id).to_string(),
            name: if name.is_empty() {
                (*id).to_string()
            } else {
                (*name).to_string()
            },
        })
        .collect()
}

pub fn page_index(pages: &[SubjectPage], saved_id: &str) -> usize {
    pages
        .iter()
        .position(|page| page.id == saved_id)
        .unwrap_or(0)
}

/// Move one page. Next and Prev both wrap.
pub fn turn_page(index: usize, count: usize, delta: i32) -> usize {
    if count == 0 {
        return 0;
    }
    let count_i = count as i32;
    (index as i32 + delta).rem_euclid(count_i) as usize
}

pub fn page_label(index: usize, count: usize, name: &str) -> String {
    if count == 0 {
        "0 / 0".into()
    } else {
        format!("{} / {} · {}", index + 1, count, name)
    }
}

pub fn move_selection(ids: &[String], current: Option<&str>, delta: i32) -> Option<String> {
    if ids.is_empty() {
        return None;
    }
    let pos = current.and_then(|id| ids.iter().position(|item| item == id));
    let next = match (pos, delta.signum()) {
        (None, 1..) => 0,
        (None, _) => ids.len() - 1,
        (Some(index), _) => {
            let next = index as i32 + delta;
            if next < 0 {
                0
            } else if next >= ids.len() as i32 {
                ids.len() - 1
            } else {
                next as usize
            }
        }
    };
    Some(ids[next].clone())
}

pub fn scroll_by(
    offsets: &mut std::collections::HashMap<String, f32>,
    id: &str,
    delta: f32,
    limit: f32,
) -> f32 {
    let limit = limit.max(0.0);
    let next = (offsets.get(id).copied().unwrap_or(0.0) + delta).clamp(0.0, limit);
    offsets.insert(id.to_string(), next);
    next
}

pub fn scroll_of(offsets: &std::collections::HashMap<String, f32>, id: &str) -> f32 {
    offsets.get(id).copied().unwrap_or(0.0)
}

/// Reply editor drawn inside the gold card. Closing it never posts.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EditPopout {
    pub card_id: String,
    pub open: bool,
    pub draft: String,
    pub tone: String,
    pub variants: Vec<String>,
    pub provider: TakeProvider,
}

pub fn select_take(card_id: &str) -> String {
    format!("take:{card_id}")
}

pub fn selection_colors(selected: bool) -> (u32, u32) {
    if selected {
        (SELECT_PINK, SELECT_CYAN)
    } else {
        (0xffd60a, 0xffe56a)
    }
}

/// A card click replaces any previous selection. Empty space and Esc clear it.
pub fn selection_after_card(card_id: &str) -> Option<String> {
    Some(select_take(card_id))
}

pub fn selection_after_empty() -> Option<String> {
    None
}

pub fn send_is_busy(phase: &ReplyPhase) -> bool {
    matches!(phase, ReplyPhase::Pending { .. })
}

pub fn open_editor(card_id: &str, draft: &str, provider: TakeProvider) -> EditPopout {
    let tone = crate::model::Tone::Reply;
    EditPopout {
        card_id: card_id.to_string(),
        open: true,
        draft: draft.to_string(),
        tone: tone.as_str().to_string(),
        variants: crate::trends_desk::distinct_variants(draft, &[]),
        provider,
    }
}

pub fn close_editor(pop: &EditPopout) -> EditPopout {
    EditPopout {
        open: false,
        ..pop.clone()
    }
}

/// Shown in the reply box until the one take for this card is ready.
pub const DRAFTING_PLACEHOLDER: &str = "Drafting…";

/// The pink box holds the reply, three voice chips, Copy, and Send.
pub fn pink_box_controls() -> [&'static str; 5] {
    ["Funny", "Smart", "Spicy", "Copy", "Post reply"]
}

/// Draft take shows the box. It does not write a draft file in the background.
pub fn draft_click_queues_a_file() -> bool {
    false
}

/// Draft take opens the in-card editor on the click, before any provider runs.
pub fn editor_on_draft_click(card_id: &str, provider: TakeProvider) -> EditPopout {
    open_editor(card_id, DRAFTING_PLACEHOLDER, provider)
}

fn reply_is_status(text: &str) -> bool {
    let trimmed = text.trim();
    trimmed.is_empty()
        || trimmed == DRAFTING_PLACEHOLDER
        || trimmed.starts_with(super::takes::HONEST_PREFIX)
        || trimmed.starts_with("Hermes ·")
        || trimmed.starts_with("Vyce ·")
        || trimmed.starts_with("Grok ·")
        || trimmed.starts_with("Muse ")
        || trimmed.starts_with("Agy ")
}

/// A finished take replaces Drafting…. Errors stay out of the reply.
pub fn text_after_take(waiting: &str, generated: &str, _error: &str) -> String {
    if !reply_is_status(waiting) {
        return waiting.to_string();
    }
    let generated = generated.trim();
    if generated.is_empty() {
        String::new()
    } else {
        generated.to_string()
    }
}

/// Replace the placeholder when the take arrives. Typed text stays.
pub fn fill_reply_if_waiting(current: &str, generated: &str) -> String {
    let generated = generated.trim();
    if current == DRAFTING_PLACEHOLDER || current.trim().is_empty() {
        if generated.is_empty() {
            current.to_string()
        } else {
            generated.to_string()
        }
    } else {
        current.to_string()
    }
}

/// Reply and Edit open the panel. They draft only when the box has no take yet.
pub fn reply_needs_a_take(existing: &str) -> bool {
    let trimmed = existing.trim();
    trimmed.is_empty() || trimmed == DRAFTING_PLACEHOLDER
}

/// Clicking a variant copies that line into the reply box.
pub fn apply_variant(draft: &mut String, variants: &[String], index: usize) -> bool {
    let Some(text) = variants.get(index) else {
        return false;
    };
    *draft = text.clone();
    true
}

/// Send is allowed only from an open editor and an explicit click.
/// This does not post. The caller posts only after `Ok`.
pub fn editor_send(pop: &EditPopout, explicit: bool) -> Result<String, String> {
    if !explicit {
        return Err("Send requires an explicit click.".into());
    }
    if !pop.open {
        return Err("The editor is closed. Nothing was posted.".into());
    }
    let text = text_to_post(pop.draft.trim());
    if text.is_empty()
        || text == DRAFTING_PLACEHOLDER
        || text.starts_with(super::takes::HONEST_PREFIX)
    {
        return Err("Write a reply first.".into());
    }
    Ok(text)
}

/// The string that is saved and posted. A display caret at the end is removed.
/// A `|` the user typed between words stays.
pub fn text_to_post(text: &str) -> String {
    let trimmed = text.trim();
    let Some(rest) = trimmed.strip_suffix('|') else {
        return trimmed.to_string();
    };
    rest.trim_end().to_string()
}

pub fn tone_variants(idea: &str, tone: crate::model::Tone) -> Vec<String> {
    crate::ai::template_variants(idea, tone, 3)
        .into_iter()
        .map(|draft| draft.text)
        .take(3)
        .collect()
}

pub fn char_count(text: &str) -> usize {
    crate::text::weighted_len_limited(text, crate::text::STANDARD_CHAR_LIMIT).weighted
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ReplyPhase {
    Idle,
    Editing { text: String },
    Pending { text: String },
    Confirmed { text: String, receipt: String },
    Failed { text: String, error: String },
}

impl ReplyPhase {
    pub fn text(&self) -> &str {
        match self {
            Self::Editing { text }
            | Self::Pending { text }
            | Self::Confirmed { text, .. }
            | Self::Failed { text, .. } => text,
            Self::Idle => "",
        }
    }

    pub fn label(&self) -> &'static str {
        match self {
            Self::Idle => "idle",
            Self::Editing { .. } => "editing",
            Self::Pending { .. } => "pending",
            Self::Confirmed { .. } => "confirmed",
            Self::Failed { .. } => "failed",
        }
    }
}

pub fn edit_reply(phase: &ReplyPhase, text: String) -> ReplyPhase {
    if matches!(phase, ReplyPhase::Pending { .. }) {
        return phase.clone();
    }
    ReplyPhase::Editing { text }
}

/// Move to pending only when the user explicitly chooses Send.
pub fn request_send(phase: &ReplyPhase, explicit: bool) -> Result<ReplyPhase, String> {
    if !explicit {
        return Err("Send requires an explicit click.".into());
    }
    let text = phase.text().trim().to_string();
    if text.is_empty() {
        return Err("Write a reply first.".into());
    }
    if matches!(phase, ReplyPhase::Pending { .. } | ReplyPhase::Idle) {
        return Err("Nothing is ready to send.".into());
    }
    Ok(ReplyPhase::Pending { text })
}

/// The bright cyan Post button inside the pink box.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PinkPost {
    /// Shown once a take or typed draft exists (or a send is in flight/done).
    pub visible: bool,
    /// Clickable. Never while posting, after a confirmed post of the same
    /// text, or when the box holds nothing postable.
    pub enabled: bool,
    pub label: &'static str,
    /// One short inline line under the text: the error, or the posted link.
    pub note: Option<String>,
}

/// State of the in-box Post button for the box text and the card's phase.
/// It only describes the button; posting still needs the explicit click.
pub fn pink_post_button(text: &str, phase: &ReplyPhase) -> PinkPost {
    let postable = !reply_is_status(text) && !text_to_post(text).is_empty();
    match phase {
        ReplyPhase::Pending { .. } => PinkPost {
            visible: true,
            enabled: false,
            label: "Posting…",
            note: None,
        },
        ReplyPhase::Confirmed { receipt, .. } => PinkPost {
            visible: true,
            enabled: false,
            label: "Posted ✓",
            note: super::pack::reply_link(receipt),
        },
        ReplyPhase::Failed { error, .. } => PinkPost {
            visible: true,
            enabled: postable,
            label: "Retry post",
            note: Some(error.clone()),
        },
        ReplyPhase::Idle | ReplyPhase::Editing { .. } => PinkPost {
            visible: postable,
            enabled: postable,
            label: "Post",
            note: None,
        },
    }
}

pub fn apply_send_result(pending: &ReplyPhase, result: Result<String, String>) -> ReplyPhase {
    let ReplyPhase::Pending { text } = pending else {
        return pending.clone();
    };
    match result {
        Ok(receipt) => ReplyPhase::Confirmed {
            text: text.clone(),
            receipt,
        },
        Err(error) => ReplyPhase::Failed {
            text: text.clone(),
            error,
        },
    }
}

pub trait ReplyPoster {
    fn post_reply(&self, target_url: &str, text: &str) -> Result<String, String>;
}

/// Calls the poster only after an explicit send. A false `explicit` leaves the
/// phase unchanged and does not call the poster.
pub fn commit_send(
    phase: &ReplyPhase,
    explicit: bool,
    target_url: &str,
    poster: &dyn ReplyPoster,
) -> ReplyPhase {
    let Ok(pending) = request_send(phase, explicit) else {
        return phase.clone();
    };
    let ReplyPhase::Pending { text } = &pending else {
        return pending;
    };
    apply_send_result(&pending, poster.post_reply(target_url, text))
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TakeProvider {
    Treg,
    Free,
    Hermes,
    Grok,
    Vyce,
    Muse,
    Agy,
    Custom,
}

impl TakeProvider {
    pub fn parse(value: &str) -> Self {
        match value.trim().to_ascii_lowercase().as_str() {
            "treg" => Self::Treg,
            "hermes" => Self::Hermes,
            "grok" => Self::Grok,
            "vyce" | "vyce-direct" => Self::Vyce,
            "muse" => Self::Muse,
            "agy" | "antigravity" => Self::Agy,
            "custom" => Self::Custom,
            _ => Self::Free,
        }
    }

    pub fn caption(self) -> &'static str {
        match self {
            Self::Treg => "Provider · Treg",
            Self::Free => "Provider · Free",
            Self::Hermes => "Hermes · Vyce glm-5.3-flash",
            Self::Grok => "Provider · Grok",
            Self::Vyce => "Vyce · glm-5.3-flash",
            Self::Muse => "Provider · Muse",
            Self::Agy => "Provider · Agy",
            Self::Custom => "Provider · Custom",
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Treg => "treg",
            Self::Free => "free",
            Self::Hermes => "hermes",
            Self::Grok => "grok",
            Self::Vyce => "vyce",
            Self::Muse => "muse",
            Self::Agy => "agy",
            Self::Custom => "custom",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TakeDraft {
    pub text: String,
    pub source: &'static str,
    pub note: String,
    /// Model replies when the provider returned more than one. Empty means
    /// the editor builds three distinct lines from `text`.
    pub variants: Vec<String>,
}

pub fn short_take_note(note: &str) -> String {
    let lower = note.to_lowercase();
    if lower.contains("402") || lower.contains("credit") || lower.contains("payment required") {
        "Free model needs credits (402) · fallback angle".into()
    } else if lower.contains("401") || lower.contains("unauthorized") {
        "Free model needs a key (401) · fallback angle".into()
    } else if lower.contains("not installed") {
        "Free model is not installed · fallback angle".into()
    } else if lower.contains("unreachable") {
        "Free model unreachable · fallback angle".into()
    } else {
        "fallback angle".into()
    }
}

pub fn offline_take(post: &SourcePost) -> TakeDraft {
    TakeDraft {
        text: super::domain::idea_from(post),
        source: "offline",
        note: "offline take".into(),
        variants: Vec::new(),
    }
}

/// The house free model (configured OpenAI-compatible endpoint, otherwise the
/// local free default). A miss becomes an extractive take from this post only.
pub fn free_take(config: &crate::config::XConfig, http: &dyn Http, post: &SourcePost) -> TakeDraft {
    let (base, _) = config.ai_endpoint();
    let mut config = config.clone();
    if config.ai_model.trim().is_empty() {
        match crate::ai::choose_chat_model(&config, http) {
            Ok(choice) => config.ai_model = choice.model,
            Err(reason) => return offline_with(post, &reason),
        }
    }
    let model = config.ai_endpoint().1;
    let system = "You write one short X reply under 220 characters. Take a side: a question, a counterpoint, or why it matters. Use a detail from this post. No hashtags. Output only the reply.";
    let body: String = post.text.chars().take(500).collect();
    let user = format!(
        "Author: {} @{}\nPost:\n{body}",
        post.author_name, post.author
    );
    let where_from = format!("{model} @ {base}");
    match crate::ai::chat_completion(&config, http, system, &user) {
        Ok(text) if text.trim().chars().count() >= 8 && !is_canned(&text) => TakeDraft {
            text,
            source: "model",
            note: format!("Free model draft ({where_from}). Not posted."),
            variants: Vec::new(),
        },
        Ok(_) => offline_with(
            post,
            &format!("free model unreachable: empty or generic draft ({where_from})"),
        ),
        Err(error) => {
            let error = redact_token(&error, &config.api_key().unwrap_or_default());
            let error = shorten_reason(&error);
            offline_with(
                post,
                &format!("free model unreachable: {error} ({where_from})"),
            )
        }
    }
}

fn offline_with(_post: &SourcePost, reason: &str) -> TakeDraft {
    TakeDraft {
        text: String::new(),
        source: "offline",
        note: super::takes::honest_failure(&[reason.to_string()]),
        variants: Vec::new(),
    }
}

fn is_canned(text: &str) -> bool {
    let text = text.to_lowercase();
    text.contains("not another hype pile-on")
        || text.contains("people are actually engaging with this")
}

fn shorten_reason(error: &str) -> String {
    let error = error.split_whitespace().collect::<Vec<_>>().join(" ");
    let mut chars = error.chars();
    let short: String = chars.by_ref().take(160).collect();
    if chars.next().is_some() {
        format!("{short}…")
    } else {
        short
    }
}

/// Treg reads the existing Termy X key and checks it. It does not write a new
/// key and it does not fall back to a fake success when the check fails.
/// The wording is still the free TermyXL template, labeled as such.
pub fn treg_take(
    config: &crate::config::XConfig,
    http: &dyn Http,
    post: &SourcePost,
) -> Result<TakeDraft, String> {
    let token = if config.token_store == "file" {
        reply_guy::load_file_secret(config, Secret::Treg)
    } else {
        reply_guy::load_secret(config, Secret::Treg)
    };
    let Some(token) = token else {
        return Err("No Treg key is saved in Termy X.".into());
    };
    treg_take_with_token(&token, http, post)
}

pub fn treg_take_with_token(
    token: &str,
    http: &dyn Http,
    post: &SourcePost,
) -> Result<TakeDraft, String> {
    let prompt = format!(
        "Write one short X reply under 240 characters. No hashtags. Post by @{}: {}",
        post.author, post.text
    );
    let body = serde_json::json!({
        "messages": [{"role": "user", "content": prompt}]
    })
    .to_string();
    let response = http
        .request(reply_guy::treg_request(
            "POST",
            "https://treg.to/call/openai.chat.completions",
            token,
            Some(body),
        ))
        .map_err(|error| redact_token(&error, token))?;
    let text = parse_treg_draft(&response).map_err(|error| redact_token(&error, token))?;
    Ok(TakeDraft {
        text,
        source: "treg",
        note: "Treg draft. Not posted.".into(),
        variants: Vec::new(),
    })
}

pub fn parse_treg_draft(body: &str) -> Result<String, String> {
    let value: serde_json::Value =
        serde_json::from_str(body).map_err(|error| format!("treg: bad response ({error})"))?;
    if let Some(text) = value
        .pointer("/error/message")
        .and_then(|item| item.as_str())
        .or_else(|| value.get("detail").and_then(|item| item.as_str()))
        .or_else(|| value.get("error").and_then(|item| item.as_str()))
        .or_else(|| value.get("message").and_then(|item| item.as_str()))
        && value.pointer("/choices/0/message/content").is_none()
        && value.pointer("/output/data/text").is_none()
        && value.pointer("/output/text").is_none()
    {
        return Err(format!("treg: {text}"));
    }
    let text = value
        .pointer("/choices/0/message/content")
        .and_then(|item| item.as_str())
        .or_else(|| {
            value
                .pointer("/output/data/text")
                .and_then(|item| item.as_str())
        })
        .or_else(|| value.pointer("/output/text").and_then(|item| item.as_str()))
        .unwrap_or("")
        .trim()
        .to_string();
    if text.is_empty() {
        Err("treg returned an empty draft".into())
    } else {
        Ok(text)
    }
}

pub fn draft_take(
    provider: TakeProvider,
    config: &crate::config::XConfig,
    post: &SourcePost,
    http: &dyn Http,
) -> Result<TakeDraft, String> {
    match provider {
        TakeProvider::Free => Ok(free_take(config, http, post)),
        TakeProvider::Treg => treg_take(config, http, post),
        TakeProvider::Hermes | TakeProvider::Vyce | TakeProvider::Custom => {
            Ok(super::hermes_draft::hermes_take(config, http, post))
        }
        TakeProvider::Muse | TakeProvider::Agy => Ok(TakeDraft {
            text: String::new(),
            source: "offline",
            note: format!(
                "{} drafts through its hidden CLI. Nothing was posted.",
                provider.as_str()
            ),
            variants: Vec::new(),
        }),
        TakeProvider::Grok => {
            if let Some(stub) = super::takes::grok_stub_output() {
                return Ok(super::takes::grok_draft_from_output(post, &stub));
            }
            match super::takes::grok_cli(post, "Reply") {
                Ok(text) => Ok(super::takes::grok_draft_from_output(post, &text)),
                Err(error) => Ok(TakeDraft {
                    text: String::new(),
                    source: "offline",
                    note: super::honest_failure(&[error]),
                    variants: Vec::new(),
                }),
            }
        }
    }
}

fn redact_token(error: &str, token: &str) -> String {
    if token.is_empty() {
        return error.to_string();
    }
    error.replace(token, "[redacted]")
}

/// Request shape kept so a future catalog draft tool can be swapped in
/// without changing the state machine. Tests assert it carries the token header
/// and never targets a post/reply publish URL.
#[cfg(test)]
fn treg_draft_probe(token: &str) -> HttpRequest {
    reply_guy::treg_request("GET", reply_guy::TREG_ME_URL, token, None)
}

#[cfg(test)]
mod tests;
