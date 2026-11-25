//! Per-card history of generated takes. Funny, Smart, and Spicy push the
//! text that is in the box here, then a new take replaces it.

use super::cards::{DRAFTING_PLACEHOLDER, reply_is_status, text_after_take};
use super::pack::{PackFile, SavedTake, remember_saved_take};

pub const TAKE_HISTORY_MAX: usize = 20;

/// True when `current` is a real reply that should be saved before a new tone take.
pub fn take_should_be_archived(current: &str) -> bool {
    let current = current.trim();
    !current.is_empty() && !reply_is_status(current)
}

fn trim_history(pack: &mut PackFile, card_id: &str) {
    if let Some(list) = pack.takes.get_mut(card_id)
        && list.len() > TAKE_HISTORY_MAX
    {
        let drop = list.len() - TAKE_HISTORY_MAX;
        list.drain(0..drop);
    }
}

/// Save the current box text (if it is a real take) and return the Drafting…
/// placeholder that should go in the box while the next take is generated.
pub fn begin_tone_replace(
    pack: &mut PackFile,
    card_id: &str,
    current: &str,
    provider: &str,
    tone: &str,
) -> String {
    if take_should_be_archived(current) {
        remember_saved_take(pack, card_id, current, provider, tone);
        trim_history(pack, card_id);
    }
    DRAFTING_PLACEHOLDER.to_string()
}

pub fn takes_for_card<'a>(pack: &'a PackFile, card_id: &str) -> &'a [SavedTake] {
    pack.takes.get(card_id).map(Vec::as_slice).unwrap_or(&[])
}

pub fn restore_saved_take(list: &[SavedTake], index: usize) -> Option<&SavedTake> {
    list.get(index)
}

/// After a tone click has put Drafting… in the box, the new take replaces it.
/// Typed text entered while that draft is in flight is still kept.
pub fn text_after_tone_take(waiting: &str, generated: &str) -> String {
    text_after_take(waiting, generated, "")
}

/// Sort a subject by most-engaged posts. Leaves the search provider alone
/// so the free OpenCLI path stays the default.
pub fn apply_viral_sort(sort: &mut String) {
    *sort = "popular".into();
}

pub const TREG_DESK_LIMIT: usize = 12;

/// likes + 2*reposts + replies. Used to rank which posts could go viral.
pub fn engagement_score(likes: u64, reposts: u64, replies: u64) -> u64 {
    likes
        .saturating_add(reposts.saturating_mul(2))
        .saturating_add(replies)
}

/// Rank fetched posts for the Viral/Top view. When `sort` is popular, the
/// most engaged posts come first. `cap` is only used to limit paid Treg
/// fetches; the free OpenCLI path leaves it `None`.
pub fn rank_fetched_posts<T, F>(
    mut posts: Vec<T>,
    sort: &str,
    cap: Option<usize>,
    score: F,
) -> Vec<T>
where
    F: Fn(&T) -> u64,
{
    if sort == "popular" {
        posts.sort_by_key(|post| std::cmp::Reverse(score(post)));
    }
    if let Some(cap) = cap {
        posts.truncate(cap);
    }
    posts
}

pub fn treg_missing_key_message() -> &'static str {
    "Treg key not found. Save it in Termy X (TREG_API_KEY), or set TREG_API_KEY or TREG_TOKEN. Nothing was fetched."
}

#[cfg(test)]
mod tests {
    use super::*;

    fn pack() -> PackFile {
        PackFile::default()
    }

    #[test]
    fn tone_click_archives_the_old_take_and_clears_the_box() {
        let mut pack = pack();
        let next = begin_tone_replace(&mut pack, "c1", "old funny take", "hermes", "Funny");
        assert_eq!(next, DRAFTING_PLACEHOLDER);
        let list = takes_for_card(&pack, "c1");
        assert_eq!(list.len(), 1);
        assert_eq!(list[0].text, "old funny take");
        assert_eq!(list[0].tone, "Funny");
        assert_eq!(
            text_after_tone_take(DRAFTING_PLACEHOLDER, "new spicy take"),
            "new spicy take"
        );
    }

    #[test]
    fn drafting_or_empty_is_not_archived() {
        let mut pack = pack();
        begin_tone_replace(&mut pack, "c1", DRAFTING_PLACEHOLDER, "hermes", "Smart");
        begin_tone_replace(&mut pack, "c1", "   ", "hermes", "Smart");
        assert!(takes_for_card(&pack, "c1").is_empty());
    }

    #[test]
    fn a_duplicate_of_the_last_take_is_not_stored_twice() {
        let mut pack = pack();
        begin_tone_replace(&mut pack, "c1", "same take", "hermes", "Funny");
        begin_tone_replace(&mut pack, "c1", "same take", "hermes", "Funny");
        assert_eq!(takes_for_card(&pack, "c1").len(), 1);
    }

    #[test]
    fn history_caps_at_twenty_and_restore_returns_the_saved_text() {
        let mut pack = pack();
        for i in 0..25 {
            begin_tone_replace(&mut pack, "c1", &format!("take {i}"), "hermes", "Smart");
        }
        let list = takes_for_card(&pack, "c1");
        assert_eq!(list.len(), TAKE_HISTORY_MAX);
        assert_eq!(list[0].text, "take 5");
        assert_eq!(list[19].text, "take 24");
        assert_eq!(restore_saved_take(list, 0).unwrap().text, "take 5");
        assert!(restore_saved_take(list, 99).is_none());
    }

    #[test]
    fn typed_text_during_a_tone_draft_is_kept() {
        assert_eq!(
            text_after_tone_take("I rewrote this", "generated"),
            "I rewrote this"
        );
    }

    #[test]
    fn viral_sort_sets_popular_and_leaves_the_provider_alone() {
        let mut source = "opencli".to_string();
        let mut sort = "newest".to_string();
        apply_viral_sort(&mut sort);
        assert_eq!(source, "opencli");
        assert_eq!(sort, "popular");
        assert_eq!(TREG_DESK_LIMIT, 12);
        assert!(treg_missing_key_message().contains("TREG_API_KEY"));
        assert!(treg_missing_key_message().contains("Nothing was fetched"));
    }

    #[test]
    fn popular_results_are_ranked_by_engagement_and_capped() {
        let posts = vec![
            ("low", 1u64, 0u64, 0u64),
            ("high", 10, 4, 2),
            ("mid", 8, 0, 1),
        ];
        let ranked = rank_fetched_posts(posts, "popular", Some(2), |p| {
            engagement_score(p.1, p.2, p.3)
        });
        assert_eq!(ranked.len(), 2);
        assert_eq!(ranked[0].0, "high"); // 10+8+2=20
        assert_eq!(ranked[1].0, "mid"); // 8+0+1=9
    }

    #[test]
    fn newest_results_keep_provider_order() {
        let posts = vec![("a", 1u64, 0u64, 0u64), ("b", 99, 0, 0)];
        let ranked = rank_fetched_posts(posts, "newest", None, |p| engagement_score(p.1, p.2, p.3));
        assert_eq!(ranked[0].0, "a");
        assert_eq!(ranked[1].0, "b");
    }
}
