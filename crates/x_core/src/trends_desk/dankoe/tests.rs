use super::*;
use crate::trends_desk::SourcePost;

fn post(id: &str, author: &str, likes: u64, reposts: u64, replies: u64, text: &str) -> SourcePost {
    SourcePost {
        id: id.into(),
        author: author.into(),
        text: text.into(),
        url: format!("https://x.com/{author}/status/{id}"),
        likes,
        reposts,
        replies,
        ..SourcePost::default()
    }
}

fn filled_map() -> ContentMap {
    let mut map = ContentMap {
        point_a: "Stuck in a 9-5 they hate".into(),
        point_b: "Running a one-person business".into(),
        ..ContentMap::default()
    };
    for topic in ["Business", "AI"] {
        let t = map.add_topic(topic).unwrap();
        for sub in ["Writing", "Products", "Audience"] {
            let s = map.add_subtopic(t, sub).unwrap();
            for idea in ["one", "two", "three"] {
                map.add_idea(t, s, &format!("{topic} {sub} {idea}"))
                    .unwrap();
            }
        }
    }
    map
}

#[test]
fn content_map_checklist_follows_dan_koe_ranges() {
    let empty = ContentMap::default();
    assert!(empty.checklist().iter().all(|(_, ok)| !ok));
    let map = filled_map();
    let list = map.checklist();
    assert!(list.iter().all(|(_, ok)| *ok), "{list:?}");
    assert!(map_status_line(&map).contains("5/5 map steps done"));
    assert_eq!(map.idea_count(), 18);
    let mut one_topic = map;
    one_topic.remove_topic(1);
    assert!(!one_topic.checklist()[2].1);
}

#[test]
fn content_map_rejects_blank_duplicate_and_overflow() {
    let mut map = ContentMap::default();
    assert!(map.add_topic("   ").is_err());
    map.add_topic("Health").unwrap();
    assert!(map.add_topic("health").is_err());
    assert!(map.add_subtopic(9, "Sleep").is_err());
    assert!(map.add_idea(0, 0, "idea").is_err());
    for name in ["a", "b", "c", "d"] {
        map.add_topic(name).unwrap();
    }
    assert!(map.add_topic("six").is_err());
}

#[test]
fn draft_tags_cycle_through_subtopics_and_clear() {
    let mut map = filled_map();
    let paths = map.subtopic_paths();
    assert_eq!(paths.len(), 6);
    assert_eq!(paths[0], "Business › Writing");
    assert_eq!(map.next_tag(None).as_deref(), Some("Business › Writing"));
    assert_eq!(
        map.next_tag(Some("Business › Writing")).as_deref(),
        Some("Business › Products")
    );
    assert_eq!(map.next_tag(Some("AI › Audience")), None);
    map.set_tag("d1", Some("AI › Writing".into()));
    map.set_tag("d2", Some("AI › Writing".into()));
    assert_eq!(map.tag_count("AI › Writing"), 2);
    map.remove_subtopic(1, 0);
    assert_eq!(map.tag_for("d1"), None);
    map.set_tag("d3", Some("Business › Audience".into()));
    map.remove_topic(0);
    assert!(map.tags.is_empty());
}

#[test]
fn map_draft_keeps_the_idea_and_names_the_subtopic() {
    let map = filled_map();
    let draft = map_draft(&map, 0, 1, 2).unwrap();
    assert_eq!(draft.text, "Business Products three");
    assert_eq!(
        draft.tone.as_deref(),
        Some("Content Map · Business › Products")
    );
    assert_eq!(draft.reply_to, None);
    assert!(map_draft(&map, 5, 0, 0).is_none());
}

#[test]
fn content_map_and_vault_round_trip_on_disk() {
    let dir = tempfile::tempdir().unwrap();
    let map = filled_map();
    save_content_map(dir.path(), &map).unwrap();
    assert_eq!(load_content_map(dir.path()), map);
    let mut vault = ViralVault::default();
    vault
        .add(VaultEntry::from_post(
            &post("1", "a", 50, 1, 1, "hi"),
            None,
            7,
        ))
        .unwrap();
    save_vault(dir.path(), &vault).unwrap();
    assert_eq!(load_vault(dir.path()), vault);
    std::fs::write(vault_file(dir.path()), "not json").unwrap();
    assert!(load_vault(dir.path()).entries.is_empty());
}

#[test]
fn hook_check_scores_a_strong_post_ready() {
    let text = "Most people quit X after 3 weeks.\nThey're stuck posting random topics.\nI built a content map instead and grew to 10k followers.\nStop guessing. Map it.";
    let check = hook_check(text);
    assert!(check.ready(), "{check:?}");
    assert!(check.score() >= 5, "{}", check.marks());
    assert!(check.summary().ends_with("ready"));
}

#[test]
fn hook_check_flags_a_weak_post_and_names_what_to_add() {
    let check = hook_check("Thoughts on the weather today in general and other things happening");
    assert!(!check.ready());
    assert!(check.summary().contains("add"));
    assert_eq!(check.items.len(), 6);
    assert_eq!(hook_check("   ").score(), 0);
}

#[test]
fn hook_check_does_not_read_hi_as_first_person() {
    let check = hook_check("hi there friends");
    let solution = check
        .items
        .iter()
        .find(|item| item.part == HookPart::Solution)
        .unwrap();
    assert!(!solution.hit);
}

#[test]
fn viral_flag_uses_the_authors_median() {
    let a = post("1", "dan", 500, 20, 30, "big");
    let b = post("2", "@Dan", 40, 2, 3, "usual");
    let c = post("3", "dan", 60, 1, 4, "usual");
    let column = [a, b, c];
    let flags = viral_flags(&[&column], &[]);
    let flag = flags.get("1").expect("flagged");
    assert!(matches!(flag.baseline, Baseline::Author { samples: 2, .. }));
    assert!(flag.multiple >= 2.0);
    assert!(flag.label("dan").contains("@dan's usual"));
    assert!(!flags.contains_key("2"));
}

#[test]
fn viral_flag_falls_back_to_the_column_and_says_so() {
    let posts: Vec<SourcePost> = (0..5)
        .map(|i| post(&i.to_string(), &format!("u{i}"), 20, 0, 0, "x"))
        .chain([post("big", "solo", 200, 0, 0, "x")])
        .collect();
    let flags = viral_flags(&[&posts], &[]);
    let flag = flags.get("big").unwrap();
    assert!(matches!(flag.baseline, Baseline::Column { samples: 5, .. }));
    assert!(flag.label("solo").contains("column"));
    assert!(flag.short().contains("column"));
}

#[test]
fn viral_flag_needs_a_baseline_and_real_numbers() {
    let lonely = [post("1", "a", 900, 0, 0, "x"), post("2", "b", 1, 0, 0, "x")];
    assert!(viral_flags(&[&lonely], &[]).is_empty());
    let tiny = post("t", "a", 4, 0, 0, "x");
    let others = [post("o1", "a", 1, 0, 0, "x"), post("o2", "a", 1, 0, 0, "x")];
    let seen: Vec<&SourcePost> = others.iter().collect();
    assert!(viral_flag(&tiny, &seen, &[]).is_none());
}

#[test]
fn history_builds_an_author_baseline() {
    let now = [post("new", "dan", 300, 0, 0, "x")];
    let old = vec![
        post("o1", "dan", 50, 0, 0, "x"),
        post("o2", "dan", 70, 0, 0, "x"),
    ];
    let flags = viral_flags(&[&now], &old);
    assert!(matches!(
        flags.get("new").unwrap().baseline,
        Baseline::Author { .. }
    ));
}

#[test]
fn vault_caps_at_thirty_and_skips_duplicates() {
    let mut vault = ViralVault::default();
    for i in 0..VAULT_MAX {
        vault
            .add(VaultEntry::from_post(
                &post(&i.to_string(), "a", 1, 0, 0, "x"),
                None,
                0,
            ))
            .unwrap();
    }
    assert!(vault_status_line(&vault).contains("full"));
    let extra = VaultEntry::from_post(&post("99", "a", 1, 0, 0, "x"), None, 0);
    assert!(vault.add(extra).is_err());
    assert!(vault.remove("0"));
    let dup = VaultEntry::from_post(&post("1", "a", 1, 0, 0, "x"), None, 0);
    assert!(vault.add(dup).is_err());
    assert!(vault_status_line(&ViralVault::default()).contains("20 to go"));
}

#[test]
fn rule_reasons_give_three_to_five_grounded_lines() {
    let p = post(
        "1",
        "dan",
        900,
        40,
        300,
        "Stop posting random topics.\nMost people are stuck because they never map it.\nI built a map and grew faster.",
    );
    let others = [
        post("2", "dan", 50, 0, 0, "x"),
        post("3", "dan", 60, 0, 0, "x"),
    ];
    let seen: Vec<&SourcePost> = others.iter().collect();
    let flag = viral_flag(&p, &seen, &[]).unwrap();
    let reasons = rule_reasons(&p, Some(&flag));
    assert!((3..=5).contains(&reasons.len()), "{reasons:?}");
    assert!(reasons[0].starts_with("Pulled"));
    let bare = rule_reasons(&post("9", "zed", 0, 0, 0, "ok"), None);
    assert!((3..=5).contains(&bare.len()), "{bare:?}");
}

#[test]
fn ai_reasons_parse_bullets_and_numbers() {
    let parsed = parse_ai_reasons(
        "- Strong hook\n2. Relatable pain\n* Clear payoff\n\n• Picks a side\n- one\n- six six",
    )
    .unwrap();
    assert_eq!(parsed.len(), 5);
    assert_eq!(parsed[0], "Strong hook");
    assert_eq!(parsed[1], "Relatable pain");
    assert!(parse_ai_reasons("just one line").is_err());
    let kept = parse_ai_reasons("10x more reach\n3) Bold\n- Clear ask").unwrap();
    assert_eq!(kept[0], "10x more reach");
    assert_eq!(kept[1], "Bold");
}

#[test]
fn reasons_prompt_carries_the_numbers_and_the_text() {
    let entry = VaultEntry::from_post(&post("1", "@dan", 12, 3, 4, "The post body"), None, 0);
    let prompt = vault_reasons_prompt(&entry);
    assert!(prompt.contains("@dan"));
    assert!(prompt.contains("Likes 12"));
    assert!(prompt.contains("The post body"));
    assert!(prompt.contains("3 to 5"));
    assert!(entry.copy_text().contains("Why it worked (rules)"));
}

#[test]
fn nav_reaches_the_map_and_the_vault() {
    use crate::trends_desk::{Nav, Surface, surface_after_nav};
    assert_eq!(
        surface_after_nav(Surface::Columns, Nav::ContentMap),
        Surface::ContentMap
    );
    assert_eq!(
        surface_after_nav(Surface::Draft, Nav::Vault),
        Surface::Vault
    );
}
