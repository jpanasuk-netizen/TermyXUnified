//! The Content Map and Viral Vault tabs.

use gpui_kit::{Context, IntoElement, ParentElement, SharedString, Styled, div, px, rgb};
use termy_x::trends_desk::{MapPick, VIRAL_MULTIPLE, map_status_line, vault_status_line};

use super::super::design::*;
use super::super::{Field, TrendsDesk, button};
use super::{clip, heading, muted, panel};

impl TrendsDesk {
    pub(crate) fn content_map_view(&self, cx: &mut Context<Self>) -> gpui_kit::AnyElement {
        let map = &self.dk.map;
        let pick = self.dk.pick;
        let topic_name = pick
            .topic
            .and_then(|t| map.topics.get(t))
            .map(|topic| topic.name.clone());
        let sub_name = pick.topic.zip(pick.sub).and_then(|(t, s)| map.path(t, s));
        let mut checks = div().flex().flex_col().gap(px(2.0));
        for (label, ok) in map.checklist() {
            checks = checks.child(
                div()
                    .text_size(px(13.0))
                    .text_color(rgb(if ok { AC_OK } else { TX_SECONDARY }))
                    .child(format!("{} {label}", if ok { "✓" } else { "○" })),
            );
        }
        let add_row = div()
            .flex()
            .flex_wrap()
            .items_center()
            .gap(px(8.0))
            .child(self.field_chip(
                "desk-map-input",
                "New item",
                &self.dk.input,
                Field::MapInput,
                cx,
            ))
            .child(button(
                "desk-map-add-topic",
                "Add topic",
                false,
                cx.listener(|this, _, _, cx| {
                    this.map_add(0);
                    cx.notify();
                }),
            ))
            .child(button(
                "desk-map-add-sub",
                match &topic_name {
                    Some(name) => format!("Add subtopic → {}", clip(name, 24)),
                    None => "Add subtopic (pick a topic)".into(),
                },
                false,
                cx.listener(|this, _, _, cx| {
                    this.map_add(1);
                    cx.notify();
                }),
            ))
            .child(button(
                "desk-map-add-idea",
                match &sub_name {
                    Some(name) => format!("Add idea → {}", clip(name, 32)),
                    None => "Add idea (pick a subtopic)".into(),
                },
                false,
                cx.listener(|this, _, _, cx| {
                    this.map_add(2);
                    cx.notify();
                }),
            ));
        let mut topics = div().flex().flex_row().flex_wrap().gap(px(12.0));
        if map.topics.is_empty() {
            topics = topics.child(muted(
                "No topics yet. Go broader than you think (health, online business, AI). Type one in New item and press Add topic.",
            ));
        }
        for (t, topic) in map.topics.iter().enumerate() {
            topics = topics.child(self.topic_card(t, topic, pick, cx));
        }
        div()
            .flex()
            .flex_col()
            .gap(px(10.0))
            .child(heading("Content Map · Dan Koe step 1"))
            .child(muted(
                "Every post is a step from Point A to Point B. Pick 2-3 broad topics, 3-5 subtopics each, and 3-5 post ideas per subtopic. Saved in the Termy XL data folder only.",
            ))
            .child(muted(map_status_line(map)))
            .children((!self.dk.note.is_empty()).then(|| {
                div()
                    .text_size(px(13.0))
                    .text_color(rgb(AC_GOLD))
                    .child(self.dk.note.clone())
            }))
            .child(
                div()
                    .flex()
                    .flex_wrap()
                    .gap(px(8.0))
                    .child(self.field_chip(
                        "desk-map-a",
                        "Point A (life they want to escape)",
                        &map.point_a,
                        Field::MapPointA,
                        cx,
                    ))
                    .child(self.field_chip(
                        "desk-map-b",
                        "Point B (life you help them reach)",
                        &map.point_b,
                        Field::MapPointB,
                        cx,
                    )),
            )
            .child(checks)
            .child(add_row)
            .child(muted(
                "Enter in New item adds at the level you picked. Idea types that work: pain points, how-to steps, lessons learned, why it matters, harsh truths, your take on popular advice.",
            ))
            .child(topics)
            .into_any_element()
    }

    fn topic_card(
        &self,
        t: usize,
        topic: &termy_x::trends_desk::MapTopic,
        pick: MapPick,
        cx: &mut Context<Self>,
    ) -> gpui_kit::AnyElement {
        let picked_topic = pick.topic == Some(t);
        let mut card = panel()
            .w(px(420.0))
            .border_color(rgb(if picked_topic { AC_PINK } else { BD_FOCUS }))
            .child(
                div()
                    .flex()
                    .flex_wrap()
                    .gap(px(6.0))
                    .child(button(
                        SharedString::from(format!("desk-map-topic-{t}")),
                        format!("{} · {} subtopics", topic.name, topic.subtopics.len()),
                        picked_topic && pick.sub.is_none(),
                        cx.listener(move |this, _, _, cx| {
                            this.dk.pick = MapPick {
                                topic: Some(t),
                                sub: None,
                            };
                            cx.notify();
                        }),
                    ))
                    .child(button(
                        SharedString::from(format!("desk-map-topic-del-{t}")),
                        "Remove",
                        false,
                        cx.listener(move |this, _, _, cx| {
                            this.dk.map.remove_topic(t);
                            this.dk.pick = MapPick::default();
                            this.dk.note = "Topic removed.".into();
                            this.save_map();
                            cx.notify();
                        }),
                    )),
            );
        for (s, sub) in topic.subtopics.iter().enumerate() {
            let picked = picked_topic && pick.sub == Some(s);
            let path = format!("{} › {}", topic.name, sub.name);
            let tagged = self.dk.map.tag_count(&path);
            let mut block = div()
                .ml(px(10.0))
                .pl(px(8.0))
                .border_l_2()
                .border_color(rgb(if picked { AC_CYAN } else { BD_DEFAULT }))
                .flex()
                .flex_col()
                .gap(px(4.0))
                .child(
                    div()
                        .flex()
                        .flex_wrap()
                        .gap(px(6.0))
                        .child(button(
                            SharedString::from(format!("desk-map-sub-{t}-{s}")),
                            format!("{} · {} ideas · {tagged} drafts", sub.name, sub.ideas.len()),
                            picked,
                            cx.listener(move |this, _, _, cx| {
                                this.dk.pick = MapPick {
                                    topic: Some(t),
                                    sub: Some(s),
                                };
                                cx.notify();
                            }),
                        ))
                        .child(button(
                            SharedString::from(format!("desk-map-sub-del-{t}-{s}")),
                            "Remove",
                            false,
                            cx.listener(move |this, _, _, cx| {
                                this.dk.map.remove_subtopic(t, s);
                                this.dk.pick = MapPick {
                                    topic: Some(t),
                                    sub: None,
                                };
                                this.save_map();
                                cx.notify();
                            }),
                        )),
                );
            for (i, idea) in sub.ideas.iter().enumerate() {
                let drafted = idea.draft_id.is_some();
                block = block.child(
                    div()
                        .flex()
                        .flex_wrap()
                        .items_center()
                        .gap(px(6.0))
                        .child(
                            div()
                                .text_size(px(13.0))
                                .whitespace_normal()
                                .max_w(px(220.0))
                                .child(format!("• {}", idea.text)),
                        )
                        .child(button(
                            SharedString::from(format!("desk-map-idea-draft-{t}-{s}-{i}")),
                            if drafted { "Drafted ✓" } else { "Draft it" },
                            drafted,
                            cx.listener(move |this, _, _, cx| {
                                this.map_draft_idea(t, s, i);
                                cx.notify();
                            }),
                        ))
                        .child(button(
                            SharedString::from(format!("desk-map-idea-del-{t}-{s}-{i}")),
                            "×",
                            false,
                            cx.listener(move |this, _, _, cx| {
                                this.dk.map.remove_idea(t, s, i);
                                this.save_map();
                                cx.notify();
                            }),
                        )),
                );
            }
            card = card.child(block);
        }
        card.into_any_element()
    }

    pub(crate) fn vault_view(&self, cx: &mut Context<Self>) -> gpui_kit::AnyElement {
        let vault = &self.dk.vault;
        let all = vault
            .entries
            .iter()
            .map(termy_x::trends_desk::VaultEntry::copy_text)
            .collect::<Vec<_>>()
            .join("\n");
        let mut flagged = div().flex().flex_row().flex_wrap().gap(px(10.0));
        let rows = self.flagged_now();
        if rows.is_empty() {
            flagged = flagged.child(muted(
                "Nothing on the desk beats its baseline right now. Refresh columns, or let a few runs build each author's history.",
            ));
        }
        for (post, flag) in rows.iter().take(24) {
            let id = post.id.clone();
            let url = post.url.clone();
            let saved = vault.contains(&post.id, &post.url);
            flagged = flagged.child(
                panel()
                    .w(px(400.0))
                    .child(
                        div()
                            .text_size(px(13.0))
                            .text_color(rgb(AC_PINK))
                            .child(format!(
                                "@{} · {}",
                                post.author.trim_start_matches('@'),
                                flag.short()
                            )),
                    )
                    .child(
                        div()
                            .text_size(px(13.0))
                            .whitespace_normal()
                            .child(clip(&post.text, 220)),
                    )
                    .child(muted(flag.label(&post.author)))
                    .child(
                        div()
                            .flex()
                            .gap(px(6.0))
                            .child(button(
                                SharedString::from(format!("desk-vault-flag-save-{id}")),
                                if saved {
                                    "In Vault ✓"
                                } else {
                                    "Save to Vault"
                                },
                                saved,
                                cx.listener(move |this, _, _, cx| {
                                    if !this.dk.vault.contains(&id, "") {
                                        this.save_to_vault(&id);
                                    }
                                    cx.notify();
                                }),
                            ))
                            .child(button(
                                SharedString::from(format!("desk-vault-flag-open-{}", post.id)),
                                "Open",
                                false,
                                cx.listener(move |this, _, _, cx| {
                                    let _ = webbrowser::open(&url);
                                    this.action_note = "Opened in browser.".into();
                                    cx.notify();
                                }),
                            )),
                    ),
            );
        }
        let mut saved = div().flex().flex_row().flex_wrap().gap(px(10.0));
        if vault.entries.is_empty() {
            saved = saved.child(muted(
                "No saved posts yet. Save from the cards or the list above.",
            ));
        }
        for entry in vault.entries.iter().rev() {
            saved = saved.child(self.vault_card(entry, cx));
        }
        div()
            .flex()
            .flex_col()
            .gap(px(10.0))
            .child(heading("Viral Vault · Dan Koe step 2"))
            .child(muted(format!(
                "Posts that got {VIRAL_MULTIPLE:.0}x+ their author's usual engagement (likes + 2×reposts + replies against the median of that author's other posts this desk has seen; the column's median when there are fewer than 2). Save 20-30, then study why each one worked."
            )))
            .child(muted(vault_status_line(vault)))
            .children((!self.dk.note.is_empty()).then(|| {
                div()
                    .text_size(px(13.0))
                    .text_color(rgb(AC_GOLD))
                    .child(self.dk.note.clone())
            }))
            .child(button(
                "desk-vault-copy-all",
                "Copy all",
                false,
                cx.listener(move |this, _, _, cx| {
                    this.copy_text(&all, cx);
                    this.action_note = termy_x::trends_desk::copied_mark().into();
                    cx.notify();
                }),
            ))
            .child(heading(&format!("Flagged on the desk now ({})", rows.len())))
            .child(flagged)
            .child(heading(&format!("Saved ({})", vault.entries.len())))
            .child(saved)
            .into_any_element()
    }

    fn vault_card(
        &self,
        entry: &termy_x::trends_desk::VaultEntry,
        cx: &mut Context<Self>,
    ) -> gpui_kit::AnyElement {
        let id = entry.id.clone();
        let busy = self.dk.busy.contains(&entry.id);
        let mut reasons = div().flex().flex_col().gap(px(2.0));
        for reason in &entry.reasons {
            reasons = reasons.child(
                div()
                    .text_size(px(13.0))
                    .whitespace_normal()
                    .child(format!("• {reason}")),
            );
        }
        let copy = entry.copy_text();
        let url = entry.url.clone();
        let ai_id = id.clone();
        let del_id = id.clone();
        panel()
            .w(px(460.0))
            .child(
                div()
                    .text_size(px(13.0))
                    .text_color(rgb(AC_CYAN))
                    .child(format!(
                        "@{} · {} likes · {} reposts · {} replies",
                        entry.author.trim_start_matches('@'),
                        entry.likes,
                        entry.reposts,
                        entry.replies
                    )),
            )
            .child(
                div()
                    .text_size(px(13.0))
                    .whitespace_normal()
                    .child(clip(&entry.text, 320)),
            )
            .children((!entry.why_flagged.is_empty()).then(|| muted(entry.why_flagged.clone())))
            .child(muted(format!("Why it worked ({}):", entry.reasons_by)))
            .child(reasons)
            .child(
                div()
                    .flex()
                    .flex_wrap()
                    .gap(px(6.0))
                    .child(button(
                        SharedString::from(format!("desk-vault-ai-{id}")),
                        if busy { "Writing…" } else { "AI reasons" },
                        busy,
                        cx.listener(move |this, _, _, cx| {
                            this.ai_reasons(ai_id.clone(), cx);
                            cx.notify();
                        }),
                    ))
                    .child(button(
                        SharedString::from(format!("desk-vault-copy-{id}")),
                        "Copy",
                        false,
                        cx.listener(move |this, _, _, cx| {
                            this.copy_text(&copy, cx);
                            this.action_note = termy_x::trends_desk::copied_mark().into();
                            cx.notify();
                        }),
                    ))
                    .child(button(
                        SharedString::from(format!("desk-vault-open-{id}")),
                        "Open",
                        false,
                        cx.listener(move |this, _, _, cx| {
                            let _ = webbrowser::open(&url);
                            this.action_note = "Opened in browser.".into();
                            cx.notify();
                        }),
                    ))
                    .child(button(
                        SharedString::from(format!("desk-vault-del-{id}")),
                        "Remove",
                        false,
                        cx.listener(move |this, _, _, cx| {
                            this.dk.vault.remove(&del_id);
                            this.save_vault_file();
                            this.dk.note = "Removed from the vault.".into();
                            cx.notify();
                        }),
                    )),
            )
            .into_any_element()
    }
}
