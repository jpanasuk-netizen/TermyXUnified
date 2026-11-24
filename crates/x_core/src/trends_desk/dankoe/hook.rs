//! Dan Koe steps 3-5 as a quick local check on a draft. It reads the words,
//! it does not call a model, so the score is a nudge and not a verdict.
//! Aim for 3-4 or more of the six before posting.

/// Dan Koe: "check at least 3-4 of these in every post".
pub const HOOK_READY_AT: usize = 3;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HookPart {
    Hook,
    Problem,
    Solution,
    Benefit,
    Side,
    Angle,
}

impl HookPart {
    pub const ALL: [HookPart; 6] = [
        HookPart::Hook,
        HookPart::Problem,
        HookPart::Solution,
        HookPart::Benefit,
        HookPart::Side,
        HookPart::Angle,
    ];

    pub fn label(self) -> &'static str {
        match self {
            HookPart::Hook => "Hook",
            HookPart::Problem => "Problem",
            HookPart::Solution => "Your solution",
            HookPart::Benefit => "Benefit",
            HookPart::Side => "Picks a side",
            HookPart::Angle => "New angle",
        }
    }

    pub fn hint(self) -> &'static str {
        match self {
            HookPart::Hook => {
                "Open with a short first line: a number, a question, \"you\", or a bold claim."
            }
            HookPart::Problem => "Name a pain people relate to (stuck, tired of, can't, wasting).",
            HookPart::Solution => {
                "Say what you did or learned yourself (I built, I tried, my fix)."
            }
            HookPart::Benefit => "Spell out the payoff (save hours, more, faster, without).",
            HookPart::Side => "Take a position someone could disagree with (stop, never, wrong).",
            HookPart::Angle => "Flip the usual view (most people think…, actually, instead).",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HookItem {
    pub part: HookPart,
    pub hit: bool,
    /// The word or line that triggered the hit, when there is one.
    pub why: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HookCheck {
    pub items: Vec<HookItem>,
}

impl HookCheck {
    pub fn score(&self) -> usize {
        self.items.iter().filter(|item| item.hit).count()
    }

    pub fn ready(&self) -> bool {
        self.score() >= HOOK_READY_AT
    }

    pub fn missing(&self) -> Vec<HookPart> {
        self.items
            .iter()
            .filter(|item| !item.hit)
            .map(|item| item.part)
            .collect()
    }

    /// "Hook check 4/6 · ready" or "Hook check 2/6 · add Problem, Picks a side".
    pub fn summary(&self) -> String {
        let score = self.score();
        if self.ready() {
            format!("Hook check {score}/6 · ready")
        } else {
            let add = self
                .missing()
                .into_iter()
                .take(HOOK_READY_AT - score)
                .map(HookPart::label)
                .collect::<Vec<_>>()
                .join(", ");
            format!("Hook check {score}/6 · add {add}")
        }
    }

    /// "✓ Hook · ✗ Problem · …" for one compact row.
    pub fn marks(&self) -> String {
        self.items
            .iter()
            .map(|item| format!("{} {}", if item.hit { "✓" } else { "✗" }, item.part.label()))
            .collect::<Vec<_>>()
            .join(" · ")
    }
}

const HOOK_OPENERS: &[&str] = &[
    "how ",
    "why ",
    "what ",
    "stop ",
    "most ",
    "nobody ",
    "everyone ",
    "if you",
    "you ",
    "your ",
    "here's",
    "here is",
    "the truth",
    "i spent",
    "i made",
    "i built",
    "i quit",
    "i was",
    "unpopular",
    "hot take",
    "never ",
    "don't ",
    "do not ",
    "the biggest",
    "the best",
    "the one",
];
const PROBLEM: &[&str] = &[
    "struggl",
    "stuck",
    "tired of",
    "sick of",
    "hate ",
    "can't",
    "cannot",
    "frustrat",
    "problem",
    "mistake",
    "wasting",
    "waste ",
    "broke",
    "burn out",
    "burnout",
    "overwhelm",
    "afraid",
    "scared",
    "fail",
    "no traction",
    "no one reads",
    "nobody reads",
    "pain",
    "hard to",
    "too hard",
    "keeps ",
    "never works",
    "behind",
    "lost ",
    "losing",
    "quit",
    "if you hate",
    "dread",
];
const SOLUTION_SELF: &[&str] = &[
    " i ", " i'm ", " i've ", " i'd ", " my ", " we ", " we've ", " our ", " me ",
];
const SOLUTION_ACTION: &[&str] = &[
    "built",
    "learned",
    "tried",
    "started",
    "switched",
    "fixed",
    "made",
    "use ",
    "using",
    "did ",
    "found",
    "changed",
    "stopped",
    "wrote",
    "tested",
    "shipped",
    "launched",
    "grew",
    "run ",
    "runs ",
    "doing",
    "here's how",
    "here is how",
    "step",
];
const BENEFIT: &[&str] = &[
    "save",
    "faster",
    "more ",
    "free",
    "grow",
    "without",
    "finally",
    "result",
    "earn",
    "double",
    "10x",
    "2x",
    "3x",
    "hours",
    "minutes",
    "per month",
    "a month",
    "a day",
    "followers",
    "customers",
    "sales",
    "clients",
    "income",
    "profit",
    "easier",
    "so you",
    "you'll",
    "you will",
    "get more",
    "never again",
    "peace",
    "freedom",
];
const SIDE: &[&str] = &[
    "never",
    "always",
    "stop ",
    "wrong",
    "the truth",
    "unpopular",
    "overrated",
    "underrated",
    "don't",
    "do not",
    "nobody",
    "is dead",
    "should",
    "must",
    "beats",
    "better than",
    "worse than",
    "the only",
    "hot take",
    "fight me",
    "is a scam",
    "is a lie",
    "is a trap",
    "you need to",
    "quit ",
];
const ANGLE: &[&str] = &[
    "actually",
    "instead",
    "most people think",
    "most people",
    "the real ",
    "isn't about",
    "is not about",
    "not about",
    "here's the thing",
    "nobody tells",
    "no one tells",
    "secret",
    "counterintuitive",
    "opposite",
    "reframe",
    "the trick",
    "turns out",
    "what if",
    "rather than",
    "the difference",
    "not because",
    "it's not",
    "it is not",
];

fn first_hit(haystack: &str, needles: &[&str]) -> Option<String> {
    needles
        .iter()
        .find(|needle| haystack.contains(*needle))
        .map(|needle| needle.trim().to_string())
}

fn first_line(text: &str) -> &str {
    text.lines()
        .map(str::trim)
        .find(|line| !line.is_empty())
        .unwrap_or("")
}

fn hook_hit(text: &str) -> Option<String> {
    let line = first_line(text);
    let words = line.split_whitespace().count();
    if words == 0 || words > 16 {
        return None;
    }
    let lower = format!("{} ", line.to_lowercase());
    if line.chars().any(|ch| ch.is_ascii_digit()) {
        return Some("number in the first line".into());
    }
    if line.ends_with('?') {
        return Some("question in the first line".into());
    }
    if line.ends_with(':') {
        return Some("first line sets up a list".into());
    }
    if let Some(word) = HOOK_OPENERS.iter().find(|word| lower.starts_with(*word)) {
        return Some(format!("opens with \"{}\"", word.trim()));
    }
    if lower.contains(" you ") || lower.contains(" your ") {
        return Some("speaks to \"you\" up top".into());
    }
    let more_lines = text.lines().filter(|line| !line.trim().is_empty()).count() > 1;
    (more_lines && words <= 8).then(|| "short punchy first line".into())
}

/// Score a draft against Dan Koe's six "can't scroll past" checks.
pub fn hook_check(text: &str) -> HookCheck {
    let lower = format!(" {} ", text.to_lowercase().replace(['’', '‘'], "'"));
    let solution = first_hit(&lower, SOLUTION_SELF)
        .zip(first_hit(&lower, SOLUTION_ACTION))
        .map(|(who, did)| format!("{who} … {did}"));
    let mut benefit = first_hit(&lower, BENEFIT);
    if benefit.is_none() && (lower.contains('$') || lower.contains('%')) {
        benefit = Some("a number with $ or %".into());
    }
    let found = [
        (HookPart::Hook, hook_hit(text)),
        (HookPart::Problem, first_hit(&lower, PROBLEM)),
        (HookPart::Solution, solution),
        (HookPart::Benefit, benefit),
        (HookPart::Side, first_hit(&lower, SIDE)),
        (HookPart::Angle, first_hit(&lower, ANGLE)),
    ];
    HookCheck {
        items: found
            .into_iter()
            .map(|(part, why)| HookItem {
                part,
                hit: why.is_some() && !text.trim().is_empty(),
                why: why.unwrap_or_default(),
            })
            .collect(),
    }
}
