//! Dan Koe's audience loop in Termy XL: the Content Map tab (step 1), the
//! Viral Vault tab and card badge (step 2) and the Hook check on each draft
//! (steps 3-5). Files stay in the Termy XL data folder. Nothing here posts.

use std::cell::RefCell;
use std::collections::{HashMap, HashSet};
use std::hash::{Hash, Hasher};
use std::path::Path;

use gpui_kit::{
    Context, FontWeight, IntoElement, ParentElement, SharedString, Styled, div, px, rgb,
};
use termy_x::trends_desk::{
    ContentMap, MapPick, SourcePost, VaultEntry, ViralFlag, ViralVault, hook_check,
    hook_pattern_report, load_content_map, load_vault, map_draft, parse_ai_reasons,
    save_content_map, save_vault, vault_reasons_prompt, viral_flags, within_termy_x_config,
};

use super::design::*;
use super::{Field, TrendsDesk, button};

type FlagCache = (u64, HashMap<String, ViralFlag>);

#[derive(Default)]
pub(crate) struct LoopUi {
    pub(crate) map: ContentMap,
    pub(crate) pick: MapPick,
    pub(crate) input: String,
    pub(crate) vault: ViralVault,
    pub(crate) busy: HashSet<String>,
    pub(crate) note: String,
    flags: RefCell<Option<FlagCache>>,
}

impl LoopUi {
    pub(crate) fn load(root: &Path, writable: bool) -> Self {
        if !writable {
            return Self::default();
        }
        Self {
            map: load_content_map(root),
            vault: load_vault(root),
            ..Self::default()
        }
    }
}

fn heading(text: &str) -> gpui_kit::AnyElement {
    div()
        .text_size(px(16.0))
        .font_weight(FontWeight::SEMIBOLD)
        .text_color(rgb(AC_CYAN))
        .child(text.to_string())
        .into_any_element()
}

fn muted(text: impl Into<SharedString>) -> gpui_kit::AnyElement {
    div()
        .text_size(px(12.0))
        .whitespace_normal()
        .text_color(rgb(TX_SECONDARY))
        .child(text.into())
        .into_any_element()
}

fn panel() -> gpui_kit::Div {
    div()
        .p(px(10.0))
        .rounded(px(10.0))
        .bg(rgb(BG_CARD))
        .border_1()
        .border_color(rgb(BD_FOCUS))
        .flex()
        .flex_col()
        .gap(px(6.0))
}

fn clip(text: &str, max: usize) -> String {
    let text = text.split_whitespace().collect::<Vec<_>>().join(" ");
    if text.chars().count() <= max {
        text
    } else {
        format!("{}…", text.chars().take(max - 1).collect::<String>())
    }
}

impl TrendsDesk {
    // ── storage ────────────────────────────────────────────────────────────

    pub(crate) fn save_map(&mut self) {
        if !self.writable {
            self.dk.note = "The data folder is read-only. The map was not saved.".into();
            return;
        }
        if let Err(error) = save_content_map(&self.data_root, &self.dk.map) {
            self.dk.note = format!("Map save failed: {error}");
        }
    }

    fn save_vault_file(&mut self) {
        if !self.writable {
            self.dk.note = "The data folder is read-only. The vault was not saved.".into();
            return;
        }
        if let Err(error) = save_vault(&self.data_root, &self.dk.vault) {
            self.dk.note = format!("Vault save failed: {error}");
        }
    }

    /// Point A and Point B save as you type.
    pub(crate) fn after_field_edit(&mut self, field: Field) {
        if matches!(field, Field::MapPointA | Field::MapPointB) {
            self.save_map();
        }
    }

    // ── content map actions ────────────────────────────────────────────────

    fn map_add(&mut self, level: u8) {
        let text = self.dk.input.clone();
        let pick = self.dk.pick;
        let result = match (level, pick.topic, pick.sub) {
            (0, ..) => self.dk.map.add_topic(&text).map(|t| MapPick {
                topic: Some(t),
                sub: None,
            }),
            (1, Some(t), _) => self.dk.map.add_subtopic(t, &text).map(|s| MapPick {
                topic: Some(t),
                sub: Some(s),
            }),
            (2, Some(t), Some(s)) => self.dk.map.add_idea(t, s, &text).map(|_| pick),
            (1, None, _) => Err("Click a topic first, then add a subtopic.".into()),
            _ => Err("Click a subtopic first, then add an idea.".into()),
        };
        match result {
            Ok(next) => {
                self.dk.pick = next;
                self.dk.input.clear();
                self.dk.note = "Added. Saved in the Termy XL data folder.".into();
                self.save_map();
            }
            Err(error) => self.dk.note = error,
        }
    }

    /// Enter in the New item box adds at the deepest picked level.
    pub(crate) fn map_enter(&mut self) {
        let level = match (self.dk.pick.topic, self.dk.pick.sub) {
            (Some(_), Some(_)) => 2,
            (Some(_), None) => 1,
            _ => 0,
        };
        self.map_add(level);
    }

    fn map_draft_idea(&mut self, t: usize, s: usize, i: usize) {
        let path = termy_x::trends_desk::drafts_file(&self.data_root);
        if !self.writable || within_termy_x_config(&path) {
            self.dk.note = "Drafts can't be saved from here. Nothing was written.".into();
            return;
        }
        let Some(draft) = map_draft(&self.dk.map, t, s, i) else {
            return;
        };
        let id = draft.id.clone();
        let tag = self.dk.map.path(t, s);
        match termy_x::drafts::upsert_drafts(&path, vec![draft]) {
            Ok(all) => {
                self.draft_count = all.len();
                self.dk.map.set_tag(&id, tag);
                if let Some(idea) = self
                    .dk
                    .map
                    .topics
                    .get_mut(t)
                    .and_then(|topic| topic.subtopics.get_mut(s))
                    .and_then(|sub| sub.ideas.get_mut(i))
                {
                    idea.draft_id = Some(id.clone());
                }
                self.save_map();
                self.dk.note = format!(
                    "Saved draft {id}, tagged to its subtopic. Open Drafts to write it. Nothing was posted."
                );
            }
            Err(error) => self.dk.note = format!("Draft save failed: {error}"),
        }
    }

    fn cycle_tag(&mut self, draft_id: &str) {
        let next = self.dk.map.next_tag(self.dk.map.tag_for(draft_id));
        self.action_note = match &next {
            Some(tag) => format!("Tagged to {tag}."),
            None if self.dk.map.subtopic_paths().is_empty() => {
                "Add subtopics in Content Map first.".into()
            }
            None => "Tag cleared.".into(),
        };
        self.dk.map.set_tag(draft_id, next);
        self.save_map();
    }

    // ── viral vault actions ────────────────────────────────────────────────

    fn flag_key(&self) -> u64 {
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        for column in &self.columns {
            for post in &column.view.posts {
                (&post.id, post.likes, post.reposts, post.replies).hash(&mut hasher);
            }
        }
        self.history.len().hash(&mut hasher);
        hasher.finish()
    }

    fn with_flags<T>(&self, read: impl FnOnce(&HashMap<String, ViralFlag>) -> T) -> T {
        let key = self.flag_key();
        let mut cache = self.dk.flags.borrow_mut();
        if cache.as_ref().is_none_or(|(old, _)| *old != key) {
            let columns: Vec<&[SourcePost]> = self
                .columns
                .iter()
                .map(|column| column.view.posts.as_slice())
                .collect();
            let history: Vec<SourcePost> = self
                .history
                .iter()
                .flat_map(|report| report.posts.iter().cloned())
                .collect();
            *cache = Some((key, viral_flags(&columns, &history)));
        }
        let empty = HashMap::new();
        read(cache.as_ref().map_or(&empty, |(_, flags)| flags))
    }

    pub(crate) fn viral_for(&self, post_id: &str) -> Option<ViralFlag> {
        self.with_flags(|flags| flags.get(post_id).cloned())
    }

    fn flagged_now(&self) -> Vec<(SourcePost, ViralFlag)> {
        let mut seen = HashSet::new();
        let mut rows: Vec<(SourcePost, ViralFlag)> = self.with_flags(|flags| {
            self.columns
                .iter()
                .flat_map(|column| column.view.posts.iter())
                .filter(|post| seen.insert(post.id.clone()))
                .filter_map(|post| flags.get(&post.id).map(|flag| (post.clone(), flag.clone())))
                .collect()
        });
        rows.sort_by(|a, b| b.1.multiple.total_cmp(&a.1.multiple));
        rows
    }

    fn save_to_vault(&mut self, post_id: &str) {
        let Some(post) = self
            .columns
            .iter()
            .flat_map(|column| column.view.posts.iter())
            .find(|post| post.id == post_id)
            .cloned()
        else {
            self.action_note = "That post is no longer on the desk.".into();
            return;
        };
        let flag = self.viral_for(post_id);
        let entry = VaultEntry::from_post(&post, flag.as_ref(), Self::now_unix());
        match self.dk.vault.add(entry) {
            Ok(count) => {
                self.save_vault_file();
                self.action_note = format!(
                    "Saved to the Viral Vault ({count}). Reasons are a quick local read; AI reasons is on the Vault tab."
                );
            }
            Err(error) => self.action_note = error,
        }
        self.dk.note = self.action_note.clone();
    }

    fn ai_reasons(&mut self, id: String, cx: &mut Context<Self>) {
        if self.dk.busy.contains(&id) {
            return;
        }
        let Some(entry) = self.dk.vault.entries.iter().find(|entry| entry.id == id) else {
            return;
        };
        let prompt = vault_reasons_prompt(entry);
        self.dk.busy.insert(id.clone());
        self.dk.note = "Asking Grok for reasons…".into();
        cx.spawn(async move |this, cx| {
            let job = format!("vault-{id}");
            let result = cx
                .background_executor()
                .spawn(async move { termy_x::trends_desk::run_grok_prompt(&prompt, &job) })
                .await;
            let _ = this.update(cx, |desk, cx| {
                desk.dk.busy.remove(&id);
                match result.and_then(|text| parse_ai_reasons(&text)) {
                    Ok(reasons) => {
                        desk.dk.vault.set_reasons(&id, reasons, "AI · Grok CLI");
                        desk.save_vault_file();
                        desk.dk.note = "Grok wrote the reasons. Saved.".into();
                    }
                    Err(error) => {
                        desk.dk.note = format!(
                            "AI reasons failed; kept the local ones. {}",
                            clip(&termy_x::trends_desk::redact_log(&error), 160)
                        );
                    }
                }
                cx.notify();
            });
        })
        .detach();
    }

    // ── card badge ─────────────────────────────────────────────────────────

    pub(crate) fn vault_badge(
        &self,
        post: &SourcePost,
        cx: &mut Context<Self>,
    ) -> Option<gpui_kit::AnyElement> {
        let flag = self.viral_for(&post.id)?;
        let saved = self.dk.vault.contains(&post.id, &post.url);
        let id = post.id.clone();
        Some(
            div()
                .flex()
                .flex_wrap()
                .items_center()
                .gap(px(8.0))
                .child(
                    div()
                        .text_size(px(12.0))
                        .whitespace_normal()
                        .text_color(rgb(AC_PINK))
                        .child(flag.label(&post.author)),
                )
                .child(button(
                    SharedString::from(format!("desk-vault-save-{id}")),
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
                        cx.stop_propagation();
                        cx.notify();
                    }),
                ))
                .into_any_element(),
        )
    }

    // ── draft row: hook check + map tag ────────────────────────────────────

    pub(crate) fn draft_loop_row(
        &self,
        draft_id: &str,
        text: &str,
        cx: &mut Context<Self>,
    ) -> gpui_kit::AnyElement {
        let check = hook_check(text);
        let tag = self
            .dk
            .map
            .tag_for(draft_id)
            .map_or_else(|| "untagged".to_string(), str::to_string);
        let tip = check
            .missing()
            .first()
            .filter(|_| !check.ready())
            .map(|part| format!("Tip: {}", part.hint()));
        let hook_note = hook_pattern_report(text, &self.hook_picks);
        let id = draft_id.to_string();
        div()
            .flex()
            .flex_col()
            .gap(px(4.0))
            .child(
                div()
                    .text_size(px(13.0))
                    .font_weight(FontWeight::SEMIBOLD)
                    .text_color(rgb(if check.ready() { AC_OK } else { AC_GOLD }))
                    .child(check.summary()),
            )
            .child(muted(check.marks()))
            .children(tip.map(muted))
            .child(muted(hook_note))
            .child(button(
                SharedString::from(format!("desk-draft-tag-{id}")),
                format!("Map: {tag} ▸"),
                false,
                cx.listener(move |this, _, _, cx| {
                    this.cycle_tag(&id);
                    cx.stop_propagation();
                    cx.notify();
                }),
            ))
            .into_any_element()
    }
}

#[path = "xl_loop_view.rs"]
mod view;
