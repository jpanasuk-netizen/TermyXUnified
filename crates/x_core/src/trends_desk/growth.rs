//! Reply-growth rules from Pascio's "Twitter Bible" and Daniel Smidstrup's
//! 0 to 14.9K notes. They shape take prompts, the daily reply goal on the
//! HUD, and the opener check in Drafts. Nothing here posts.

/// Pascio's daily routine: "Leave 20 useful replies."
pub const DEFAULT_REPLY_GOAL: u32 = 20;
/// Goals the palette cycles through. 200 is Smidstrup's early daily volume.
pub const REPLY_GOAL_CHOICES: [u32; 4] = [20, 50, 100, 200];
/// Smidstrup: posts under 15 words got more reach.
pub const SHORT_POST_WORDS: usize = 15;

/// Pascio's Golden Reply, folded into every take prompt.
pub const GOLDEN_REPLY_RULES: &str = "Golden reply rules: give readers something the original post does not. Do one of these: extend the idea with a practical detail, tell a small story of one time this happened, ask the next question the author can actually answer, or offer a counterexample with its circumstances. It must stand alone for a stranger. No bare compliments, no passing on news, no broad questions anyone could ask.";

/// Smidstrup: invite people in with a specific question or two concrete
/// options, keep it short and text-only.
pub const QUESTION_TONE: &str = "Question writes one short text-only line, ideally under 15 words, that ends in a specific question the author can answer from experience, or offers two concrete options (this or that?). No broad questions, no hashtags, no links, no lists.";

pub fn word_count(text: &str) -> usize {
    text.split_whitespace()
        .filter(|word| word.chars().any(|ch| ch.is_alphanumeric()))
        .count()
}

/// True when the text ends in a question or offers an either/or choice.
pub fn invites_an_answer(text: &str) -> bool {
    let trimmed = text.trim_end();
    if trimmed.ends_with('?') {
        return true;
    }
    let lower = trimmed.to_ascii_lowercase();
    lower.contains('?') && lower.contains(" or ")
}

/// One line for each draft in Drafts. Counts are from the draft itself.
pub fn opener_report(text: &str) -> String {
    let words = word_count(text);
    if words == 0 {
        return "Opener: empty draft".into();
    }
    let length = if words < SHORT_POST_WORDS {
        format!("{words} words · under {SHORT_POST_WORDS} ✓")
    } else {
        format!("{words} words · trim under {SHORT_POST_WORDS} for reach")
    };
    let ask = if invites_an_answer(text) {
        "invites an answer ✓"
    } else {
        "add a specific question or two options"
    };
    format!("Opener: {length} · {ask}")
}

pub fn reply_goal_or_default(goal: u32) -> u32 {
    if goal == 0 { DEFAULT_REPLY_GOAL } else { goal }
}

pub fn next_reply_goal(current: u32) -> u32 {
    let current = reply_goal_or_default(current);
    let index = REPLY_GOAL_CHOICES
        .iter()
        .position(|goal| *goal == current)
        .map_or(0, |index| (index + 1) % REPLY_GOAL_CHOICES.len());
    REPLY_GOAL_CHOICES[index]
}

/// "3 / 20 sent today" plus a check mark once the goal is met.
pub fn reply_goal_line(today: usize, goal: u32) -> String {
    let goal = reply_goal_or_default(goal);
    if today >= goal as usize {
        format!("{today} / {goal} sent today ✓")
    } else {
        format!("{today} / {goal} sent today")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn opener_report_flags_length_and_the_ask() {
        let short = opener_report("Raised rates last month. What did you tell existing clients?");
        assert!(short.contains("10 words"), "{short}");
        assert!(short.contains("under 15 ✓"));
        assert!(short.contains("invites an answer ✓"));
        let long = opener_report(
            "This is a long tutorial style post that keeps going on and on about every single step you could take today",
        );
        assert!(long.contains("trim under 15"));
        assert!(long.contains("add a specific question"));
        assert!(invites_an_answer(
            "Notion or Obsidian? Which one stuck for you"
        ));
        assert!(!invites_an_answer("Notion or Obsidian, both fine."));
        assert_eq!(opener_report("   "), "Opener: empty draft");
    }

    #[test]
    fn reply_goal_cycles_and_marks_done() {
        assert_eq!(next_reply_goal(20), 50);
        assert_eq!(next_reply_goal(200), 20);
        assert_eq!(next_reply_goal(0), 50);
        assert_eq!(next_reply_goal(37), 20);
        assert_eq!(reply_goal_line(0, 20), "0 / 20 sent today");
        assert_eq!(reply_goal_line(20, 20), "20 / 20 sent today ✓");
        assert_eq!(reply_goal_line(3, 0), "3 / 20 sent today");
    }

    #[test]
    fn golden_rules_and_question_tone_carry_the_playbook() {
        assert!(GOLDEN_REPLY_RULES.contains("next question"));
        assert!(GOLDEN_REPLY_RULES.contains("counterexample"));
        assert!(GOLDEN_REPLY_RULES.contains("stand alone"));
        assert!(QUESTION_TONE.contains("under 15 words"));
        assert!(QUESTION_TONE.contains("two concrete options"));
        assert!(!GOLDEN_REPLY_RULES.contains('\n'));
        assert!(!QUESTION_TONE.contains('\n'));
    }
}
