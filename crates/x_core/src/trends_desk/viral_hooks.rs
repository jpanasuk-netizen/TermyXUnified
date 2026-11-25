//! Ben Chan's 44 viral Twitter hooks, transcribed from
//! https://x.com/ItsMeBenChan/status/2125197992579748169
//!
//! Used to open Funny / Smart / Spicy takes and to check drafts.

use std::cell::RefCell;

thread_local! {
    static ACTIVE_HOOK: RefCell<Option<u8>> = const { RefCell::new(None) };
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ViralHook {
    pub id: u8,
    pub name: &'static str,
    pub category: &'static str,
    pub template: &'static str,
    pub tones: &'static [&'static str],
}

pub const VIRAL_HOOKS: &[ViralHook] = &[
    ViralHook {
        id: 1,
        name: "From-to result",
        category: "Results & Growth",
        template: "I went from [starting number] to [end number] [metric] in [timeframe]. Here's what actually worked.",
        tones: &["smart"],
    },
    ViralHook {
        id: 2,
        name: "One change, Nx metric",
        category: "Results & Growth",
        template: "I [one specific change I made] and [number]x my [metric] in under [timeframe].",
        tones: &["smart"],
    },
    ViralHook {
        id: 3,
        name: "Big outcome, small following",
        category: "Results & Growth",
        template: "You can [impressive outcome] with fewer than [small number] followers.",
        tones: &["smart", "funny"],
    },
    ViralHook {
        id: 4,
        name: "Their excuse, my number",
        category: "Results & Growth",
        template: "'[The excuse everyone in your space gives].' Meanwhile I just [result with a number] in [timeframe].",
        tones: &["spicy"],
    },
    ViralHook {
        id: 5,
        name: "Faster than they think",
        category: "Results & Growth",
        template: "Most people think [result] takes [long timeframe]. I did it in [short timeframe].",
        tones: &["smart", "spicy"],
    },
    ViralHook {
        id: 6,
        name: "Popular tool, unused angle",
        category: "Results & Growth",
        template: "Everyone's using [popular tool] right now. Almost nobody is using it to [specific outcome].",
        tones: &["smart", "spicy"],
    },
    ViralHook {
        id: 7,
        name: "Impatient vs patient",
        category: "Attention & Curiosity",
        template: "People are incredibly impatient with [the thing they just started] and incredibly patient with [the thing they hate doing].",
        tones: &["funny"],
    },
    ViralHook {
        id: 8,
        name: "Most painful call",
        category: "Attention & Curiosity",
        template: "Just got off the most painful [type of call or meeting] of my career.",
        tones: &["funny", "spicy"],
    },
    ViralHook {
        id: 9,
        name: "Shouldn't give this away",
        category: "Attention & Curiosity",
        template: "I probably shouldn't be giving this [framework, template or playbook] away for free.",
        tones: &["smart"],
    },
    ViralHook {
        id: 10,
        name: "Unexpected after a change",
        category: "Attention & Curiosity",
        template: "[Number] things I didn't expect after [bold change you made]:",
        tones: &["smart"],
    },
    ViralHook {
        id: 11,
        name: "If I had X time",
        category: "Attention & Curiosity",
        template: "If I had [short timeframe] to [big goal], here's exactly what I'd do:",
        tones: &["smart"],
    },
    ViralHook {
        id: 12,
        name: "Fastest way to lose them",
        category: "Attention & Curiosity",
        template: "The fastest way to lose a [person you most want to win over]?",
        tones: &["spicy", "funny"],
    },
    ViralHook {
        id: 13,
        name: "Platform just revealed",
        category: "Insights & Personal Stories",
        template: "[Platform] just revealed [something about how it picks what to show people].",
        tones: &["smart"],
    },
    ViralHook {
        id: 14,
        name: "Beginner signs",
        category: "Insights & Personal Stories",
        template: "Signs you're still a beginner at [skill]:",
        tones: &["spicy", "funny"],
    },
    ViralHook {
        id: 15,
        name: "Living in this place",
        category: "Insights & Personal Stories",
        template: "Living in [place] in my [age range] has been one of the best things that's happened to me.",
        tones: &["funny"],
    },
    ViralHook {
        id: 16,
        name: "Up late watching again",
        category: "Insights & Personal Stories",
        template: "Was up late watching [a talk, interview or match] again. Probably the [number]th time.",
        tones: &["funny"],
    },
    ViralHook {
        id: 17,
        name: "They quietly test you",
        category: "Insights & Personal Stories",
        template: "[The people you most want to work with] love to quietly test whether you're the real deal.",
        tones: &["spicy", "smart"],
    },
    ViralHook {
        id: 18,
        name: "Early-mover year",
        category: "Insights & Personal Stories",
        template: "We're literally in [the year early movers won] [your platform or industry] right now.",
        tones: &["spicy"],
    },
    ViralHook {
        id: 19,
        name: "Tiny views, big win",
        category: "Results & Bold Claims",
        template: "Even if your posts get [tiny number] views, one [type of post] can still get you [big number].",
        tones: &["smart"],
    },
    ViralHook {
        id: 20,
        name: "Replace your salary",
        category: "Results & Bold Claims",
        template: "Most people could replace their salary by [one simple thing] every day for [timeframe].",
        tones: &["spicy", "smart"],
    },
    ViralHook {
        id: 21,
        name: "X is dead, meanwhile",
        category: "Results & Bold Claims",
        template: "'[Thing] is dead.' Meanwhile [thing] just [paid for something big].",
        tones: &["spicy", "funny"],
    },
    ViralHook {
        id: 22,
        name: "Credible person, embarrassing number",
        category: "Results & Bold Claims",
        template: "A [person with huge credibility] was getting [embarrassing number] on [platform].",
        tones: &["spicy"],
    },
    ViralHook {
        id: 23,
        name: "Impressive to strangers",
        category: "Results & Bold Claims",
        template: "Be careful not to become [impressive to strangers] and [hard to reach for the people who matter].",
        tones: &["smart"],
    },
    ViralHook {
        id: 24,
        name: "Beat someone bigger",
        category: "Results & Bold Claims",
        template: "You can beat someone [number] times your size, just not at the thing they're known for.",
        tones: &["spicy", "smart"],
    },
    ViralHook {
        id: 25,
        name: "Every project failed",
        category: "Experiences & Advice",
        template: "I tried [number] [side projects or businesses] before this one. Every single one failed.",
        tones: &["funny", "smart"],
    },
    ViralHook {
        id: 26,
        name: "Years ago I was struggling",
        category: "Experiences & Advice",
        template: "[Number] years ago I [was struggling with something specific].",
        tones: &["smart"],
    },
    ViralHook {
        id: 27,
        name: "I moved my work",
        category: "Experiences & Advice",
        template: "I moved all of my [part of my work] to [new tool or system] last month.",
        tones: &["smart"],
    },
    ViralHook {
        id: 28,
        name: "The real secret",
        category: "Experiences & Advice",
        template: "The real secret to [outcome people want]:",
        tones: &["smart", "spicy"],
    },
    ViralHook {
        id: 29,
        name: "Crazy timeframe",
        category: "Experiences & Advice",
        template: "This was a crazy [timeframe] for me:",
        tones: &["funny", "smart"],
    },
    ViralHook {
        id: 30,
        name: "How to become X",
        category: "Experiences & Advice",
        template: "How to become [trait people envy] ([number] ways to do it)",
        tones: &["smart"],
    },
    ViralHook {
        id: 31,
        name: "Tier list",
        category: "Practical Advice",
        template: "The [skill] tier list for [big goal]:",
        tones: &["funny", "smart"],
    },
    ViralHook {
        id: 32,
        name: "Sentences that changed me",
        category: "Practical Advice",
        template: "Sentences that changed how I think about [topic]:",
        tones: &["smart"],
    },
    ViralHook {
        id: 33,
        name: "Get recommended",
        category: "Practical Advice",
        template: "If you want [platform or AI tool] to start recommending you, here's what to do:",
        tones: &["smart"],
    },
    ViralHook {
        id: 34,
        name: "I'm begging you to start",
        category: "Practical Advice",
        template: "I'm begging you to start [the thing you keep putting off]. You don't have as much time as you think.",
        tones: &["spicy"],
    },
    ViralHook {
        id: 35,
        name: "You're not a kid anymore",
        category: "Practical Advice",
        template: "Please [the thing you know you should be doing]. You're not a kid anymore.",
        tones: &["spicy", "funny"],
    },
    ViralHook {
        id: 36,
        name: "Do me a favor this week",
        category: "Personal Stories & Observations",
        template: "Do me a favor this week: [one small action].",
        tones: &["smart"],
    },
    ViralHook {
        id: 37,
        name: "Patterns I've noticed",
        category: "Personal Stories & Observations",
        template: "[Number] patterns I've noticed in every [type of successful person] I know:",
        tones: &["smart"],
    },
    ViralHook {
        id: 38,
        name: "Just spoke to the senior",
        category: "Personal Stories & Observations",
        template: "Just spoke to the [senior role] at a [big or famous company].",
        tones: &["smart", "spicy"],
    },
    ViralHook {
        id: 39,
        name: "I'm [age]. Trust me",
        category: "Personal Stories & Observations",
        template: "I'm [your age]. Trust me on this:",
        tones: &["funny", "spicy"],
    },
    ViralHook {
        id: 40,
        name: "When I was [age]",
        category: "Personal Stories & Observations",
        template: "When I was [age], I was [what you were known for back then].",
        tones: &["funny", "smart"],
    },
    ViralHook {
        id: 41,
        name: "Stopped me and asked",
        category: "Stories & Hot Takes",
        template: "Halfway through my [pitch or presentation], a [high-status person] stopped me and asked:",
        tones: &["smart", "funny"],
    },
    ViralHook {
        id: 42,
        name: "Side project made $X",
        category: "Stories & Hot Takes",
        template: "My little [side project] just made $[amount] in [timeframe]. I still have my [day job].",
        tones: &["smart", "funny"],
    },
    ViralHook {
        id: 43,
        name: "Never do what everyone does",
        category: "Stories & Hot Takes",
        template: "Never [the thing everyone in your space does].",
        tones: &["spicy"],
    },
    ViralHook {
        id: 44,
        name: "The old role is dead",
        category: "Stories & Hot Takes",
        template: "The [old role] is dead. The age of the [new role] is here.",
        tones: &["spicy"],
    },
];

pub fn hook_by_id(id: u8) -> Option<&'static ViralHook> {
    VIRAL_HOOKS.iter().find(|hook| hook.id == id)
}

pub fn hook_catalog_brief() -> String {
    VIRAL_HOOKS
        .iter()
        .map(|hook| format!("{:02} {}", hook.id, hook.name))
        .collect::<Vec<_>>()
        .join("; ")
}

pub fn parse_hook_ids(raw: &str) -> Vec<u8> {
    let mut ids = Vec::new();
    for part in raw.split(|ch: char| !ch.is_ascii_digit()) {
        if part.is_empty() {
            continue;
        }
        if let Ok(id) = part.parse::<u8>()
            && hook_by_id(id).is_some()
            && !ids.contains(&id)
        {
            ids.push(id);
        }
        if ids.len() == 3 {
            break;
        }
    }
    ids
}

fn normalize(text: &str) -> String {
    text.replace(['’', '‘', '“', '”'], "'")
        .to_ascii_lowercase()
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

fn literal_parts(template: &str) -> Vec<String> {
    let mut parts = Vec::new();
    let mut rest = template;
    while let Some(start) = rest.find('[') {
        let lit = rest[..start].trim();
        if !lit.is_empty() {
            parts.push(normalize(lit));
        }
        if let Some(end) = rest[start..].find(']') {
            rest = &rest[start + end + 1..];
        } else {
            break;
        }
    }
    let tail = rest.trim();
    if !tail.is_empty() {
        parts.push(normalize(tail));
    }
    parts
}

fn useful_part(part: &str) -> bool {
    part.chars().any(|ch| ch.is_alphanumeric()) && part.len() >= 2
}

fn starts_with_placeholder(template: &str) -> bool {
    template
        .trim_start_matches(['\'', '"'])
        .trim_start()
        .starts_with('[')
}

/// True when `text` opens with this hook's literal skeleton.
pub fn opens_with_hook(text: &str, hook: &ViralHook) -> bool {
    let text = normalize(text);
    let parts: Vec<String> = literal_parts(hook.template)
        .into_iter()
        .filter(|part| useful_part(part))
        .collect();
    if parts.is_empty() {
        return false;
    }
    let mut rest = text.trim_start_matches(['\'', '"']).trim();
    let placeholder_first = starts_with_placeholder(hook.template);
    for (index, part) in parts.iter().enumerate() {
        if index == 0 && !placeholder_first {
            if !rest.starts_with(part.as_str()) {
                return false;
            }
            rest = rest[part.len()..].trim_start();
            continue;
        }
        if let Some(at) = rest.find(part.as_str()) {
            if index == 0 && at > 80 {
                return false;
            }
            rest = rest[at + part.len()..].trim_start();
        } else {
            return false;
        }
    }
    true
}

pub fn match_opening_hook(text: &str) -> Option<&'static ViralHook> {
    VIRAL_HOOKS.iter().find(|hook| opens_with_hook(text, hook))
}

const STOP: &[&str] = &[
    "from", "this", "that", "with", "have", "your", "you", "just", "about", "after", "before",
    "here", "what", "when", "then", "them", "they", "their", "into", "for", "and", "the", "was",
    "were", "been", "being", "still",
];

fn distinctive_words(template: &str) -> Vec<String> {
    let mut plain = String::new();
    let mut chars = template.chars().peekable();
    while let Some(ch) = chars.next() {
        if ch == '[' {
            while chars.next().is_some_and(|next| next != ']') {}
            plain.push(' ');
        } else {
            plain.push(ch);
        }
    }
    normalize(&plain)
        .split_whitespace()
        .filter(|word| word.len() > 4 && !STOP.contains(word))
        .map(str::to_string)
        .collect()
}

/// Pick up to three hooks. Prefers ids already chosen by the post briefing,
/// otherwise scores by tone and words that also show up in the post.
pub fn recommend_hooks(tone: &str, post_text: &str, preferred: &[u8]) -> Vec<u8> {
    let preferred: Vec<u8> = preferred
        .iter()
        .copied()
        .filter(|id| hook_by_id(*id).is_some())
        .take(3)
        .collect();
    if !preferred.is_empty() {
        return preferred;
    }
    let tone = tone.trim().to_ascii_lowercase();
    let tone = match tone.as_str() {
        "data" => "smart".to_string(),
        "savage" => "spicy".to_string(),
        other => other.to_string(),
    };
    let hay = normalize(post_text);
    let mut scored: Vec<(i32, u8)> = VIRAL_HOOKS
        .iter()
        .map(|hook| {
            let mut score = 0i32;
            if hook.tones.iter().any(|item| *item == tone) {
                score += 4;
            }
            for word in distinctive_words(hook.template) {
                if hay.contains(&word) {
                    score += 3;
                }
            }
            (score, hook.id)
        })
        .collect();
    scored.sort_by(|left, right| right.0.cmp(&left.0).then(left.1.cmp(&right.1)));
    scored.into_iter().take(3).map(|(_, id)| id).collect()
}

pub fn next_hook_id(current: Option<u8>, choices: &[u8]) -> Option<u8> {
    if choices.is_empty() {
        return None;
    }
    let Some(current) = current else {
        return Some(choices[0]);
    };
    let Some(index) = choices.iter().position(|id| *id == current) else {
        return Some(choices[0]);
    };
    Some(choices[(index + 1) % choices.len()])
}

pub fn hook_prompt_line(hook: &ViralHook) -> String {
    format!(
        "Open the reply with this hook, filled in for THIS post, then continue in the named tone: {}",
        hook.template
    )
}

pub fn hook_pattern_report(text: &str, suggest_from: &[u8]) -> String {
    if let Some(hook) = match_opening_hook(text) {
        return format!("Opens with hook {:02} ({})", hook.id, hook.name);
    }
    let suggestion = suggest_from
        .iter()
        .copied()
        .find_map(hook_by_id)
        .or_else(|| {
            recommend_hooks("smart", text, &[])
                .into_iter()
                .find_map(hook_by_id)
        });
    match suggestion {
        Some(hook) => format!(
            "No recognized hook yet · try {:02} {}: {}",
            hook.id, hook.name, hook.template
        ),
        None => "No recognized hook yet.".into(),
    }
}

pub struct HookGuard;

impl Drop for HookGuard {
    fn drop(&mut self) {
        ACTIVE_HOOK.with(|slot| slot.borrow_mut().take());
    }
}

pub fn resolve_hook(
    tone: &str,
    post_text: &str,
    preferred: &[u8],
    chosen: Option<u8>,
) -> Option<u8> {
    if let Some(id) = chosen
        && hook_by_id(id).is_some()
    {
        return Some(id);
    }
    recommend_hooks(tone, post_text, preferred).first().copied()
}

pub fn hook_picks_msg(choice: u8, picks: &[u8]) -> String {
    let list = picks
        .iter()
        .map(u8::to_string)
        .collect::<Vec<_>>()
        .join(",");
    format!("HOOKPICKS:{choice}:{list}")
}

pub fn parse_hook_picks_msg(msg: &str) -> Option<(u8, Vec<u8>)> {
    let rest = msg.strip_prefix("HOOKPICKS:")?;
    let (choice, list) = rest.split_once(':')?;
    let choice = choice.parse().ok().filter(|id| hook_by_id(*id).is_some())?;
    let picks = parse_hook_ids(list);
    if picks.is_empty() {
        None
    } else {
        Some((choice, picks))
    }
}

/// Bind the hook for this take and return the UI message with the picks.
pub fn bind_take_hook_for_post(
    tone: &str,
    post_text: &str,
    preferred: &[u8],
    chosen: Option<u8>,
) -> (HookGuard, Option<String>) {
    let picks = recommend_hooks(tone, post_text, preferred);
    let hook = resolve_hook(tone, post_text, preferred, chosen);
    let msg = hook.map(|id| hook_picks_msg(id, &picks));
    (bind_take_hook(hook), msg)
}

pub fn send_bound_hook(
    tone: &str,
    post_text: &str,
    preferred: &[u8],
    chosen: Option<u8>,
    tx: &std::sync::mpsc::Sender<String>,
) -> HookGuard {
    let (guard, msg) = bind_take_hook_for_post(tone, post_text, preferred, chosen);
    if let Some(msg) = msg {
        let _ = tx.send(msg);
    }
    guard
}

pub fn bind_take_hook(id: Option<u8>) -> HookGuard {
    ACTIVE_HOOK.with(|slot| {
        *slot.borrow_mut() = id.filter(|id| hook_by_id(*id).is_some());
    });
    HookGuard
}

pub fn active_take_hook() -> Option<&'static ViralHook> {
    ACTIVE_HOOK.with(|slot| slot.borrow().and_then(hook_by_id))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn library_has_the_forty_four_ben_chan_hooks() {
        assert_eq!(VIRAL_HOOKS.len(), 44);
        for id in 1u8..=44 {
            assert!(hook_by_id(id).is_some(), "missing hook {id}");
        }
        let first = hook_by_id(1).unwrap();
        assert!(first.template.contains("starting number"));
        assert_eq!(hook_by_id(21).unwrap().name, "X is dead, meanwhile");
        assert_eq!(hook_by_id(44).unwrap().tones, &["spicy"]);
        assert!(hook_catalog_brief().contains("01 From-to result"));
        assert!(hook_catalog_brief().contains("44 The old role is dead"));
    }

    #[test]
    fn opening_text_matches_the_hook_skeleton() {
        let dead = hook_by_id(21).unwrap();
        assert!(opens_with_hook(
            "'Print is dead.' Meanwhile print just paid for a Super Bowl ad.",
            dead
        ));
        assert!(
            match_opening_hook(
                "Most people think a book deal takes ten years. I did it in six months."
            )
            .is_some_and(|hook| hook.id == 5)
        );
        assert!(match_opening_hook("hello friends, just checking in").is_none());
    }

    #[test]
    fn hook_check_report_flags_a_hit_or_suggests_one() {
        let hit = hook_pattern_report(
            "I went from 0 to 10,000 subscribers in 90 days. Here's what actually worked.",
            &[],
        );
        assert!(hit.contains("Opens with hook 01"));
        let miss = hook_pattern_report("hm", &[4]);
        assert!(miss.contains("No recognized hook yet"));
        assert!(miss.contains("04"));
    }

    #[test]
    fn recommend_prefers_analysis_picks_and_otherwise_fits_the_post() {
        assert_eq!(
            recommend_hooks("spicy", "anything", &[21, 43, 4]),
            vec![21, 43, 4]
        );
        let picked = recommend_hooks(
            "spicy",
            "Everyone says newsletters are dead, and nobody uses the popular tool for outreach.",
            &[],
        );
        assert!(
            picked.contains(&21) || picked.contains(&6) || picked.contains(&43),
            "{picked:?}"
        );
        assert_eq!(next_hook_id(Some(21), &[21, 43, 4]), Some(43));
        assert_eq!(next_hook_id(Some(4), &[21, 43, 4]), Some(21));
        assert_eq!(resolve_hook("spicy", "x", &[21, 43], Some(43)), Some(43));
        assert_eq!(resolve_hook("spicy", "x", &[21, 43], None), Some(21));
        assert_eq!(
            parse_hook_picks_msg(&hook_picks_msg(21, &[21, 43, 4])),
            Some((21, vec![21, 43, 4]))
        );
    }

    #[test]
    fn bind_exposes_the_hook_line_for_the_take_prompt() {
        let _guard = bind_take_hook(Some(7));
        let hook = active_take_hook().unwrap();
        assert_eq!(hook.id, 7);
        assert!(hook_prompt_line(hook).contains("impatient"));
        drop(_guard);
        assert!(active_take_hook().is_none());
    }
}
