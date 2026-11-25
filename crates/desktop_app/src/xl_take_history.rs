//! Earlier takes from Funny / Smart / Spicy, shown under the pink box.

use gpui_kit::{
    Context, InteractiveElement, IntoElement, ParentElement, SharedString, Styled, div, px, rgb,
};
use termy_x::trends_desk::{
    begin_tone_replace, take_should_be_archived, takes_for_card, unique_takes_by_text,
};

use crate::trends_desk::{TrendsDesk, button};

fn clip(text: &str, max: usize) -> String {
    let text = text.split_whitespace().collect::<Vec<_>>().join(" ");
    if text.chars().count() <= max {
        text
    } else {
        format!("{}…", text.chars().take(max - 1).collect::<String>())
    }
}

impl TrendsDesk {
    pub(crate) fn why_popping_row(&self) -> gpui_kit::AnyElement {
        let Some(id) = self.popout.as_ref().map(|pop| pop.card_id.clone()) else {
            return div().into_any_element();
        };
        if self.why_popping_for != id || self.why_popping.is_empty() {
            return div().into_any_element();
        }
        div()
            .w_full()
            .text_size(px(12.0))
            .text_color(rgb(0x2a1020))
            .whitespace_normal()
            .child(self.why_popping.clone())
            .into_any_element()
    }

    pub(crate) fn take_history_row(&self, cx: &mut Context<Self>) -> gpui_kit::AnyElement {
        let Some(id) = self.popout.as_ref().map(|pop| pop.card_id.clone()) else {
            return div().into_any_element();
        };
        let list = takes_for_card(&self.pack, &id);
        let shown = unique_takes_by_text(list, |take| take.text.as_str());
        if shown.is_empty() {
            return div()
                .text_size(px(12.0))
                .text_color(rgb(0x5a6678))
                .whitespace_normal()
                .child("Previous takes are saved here when you click Funny, Smart, or Spicy.")
                .into_any_element();
        }
        let mut row = div().flex().flex_col().gap(px(4.0)).child(
            div()
                .text_size(px(12.0))
                .text_color(rgb(0x2a1020))
                .child(format!(
                    "{} earlier take(s) — click one to put it back in the box",
                    shown.len()
                )),
        );
        for index in shown.into_iter().rev() {
            let take = &list[index];
            let tone = if take.tone.trim().is_empty() {
                "Take".to_string()
            } else {
                take.tone.clone()
            };
            let label = format!("{tone} · {}", clip(&take.text, 72));
            row = row.child(button(
                SharedString::from(format!("desk-take-hist-{id}-{index}")),
                label,
                false,
                cx.listener(move |this, _, _, cx| {
                    this.restore_take(index, cx);
                    cx.stop_propagation();
                    cx.notify();
                }),
            ));
        }
        row.into_any_element()
    }

    pub(crate) fn restore_take(&mut self, index: usize, cx: &mut Context<Self>) {
        let Some(id) = self.popout.as_ref().map(|pop| pop.card_id.clone()) else {
            self.action_note = "Open a draft first.".into();
            return;
        };
        let current = self
            .reply_edit
            .as_ref()
            .map(|editor| editor.text.clone())
            .or_else(|| self.popout.as_ref().map(|pop| pop.draft.clone()))
            .unwrap_or_default();
        let tone = self
            .popout
            .as_ref()
            .map(|pop| pop.tone.clone())
            .unwrap_or_default();
        let provider = termy_x::trends_desk::provider_label(&self.settings.take_provider);
        if take_should_be_archived(&current) {
            begin_tone_replace(&mut self.pack, &id, &current, &provider, &tone);
            self.save_pack_file();
        }
        let Some(saved) = takes_for_card(&self.pack, &id).get(index).cloned() else {
            self.action_note = "That take is no longer in history.".into();
            return;
        };
        self.show_inline_editor(&id, &saved.text);
        if !saved.tone.trim().is_empty() {
            self.set_pop_tone(&saved.tone);
        }
        self.action_note = format!(
            "Restored a {} take from history. Nothing was posted.",
            if saved.tone.trim().is_empty() {
                "saved"
            } else {
                saved.tone.as_str()
            }
        );
        cx.notify();
    }
}

impl TrendsDesk {
    pub(crate) fn hook_cycle_chip(&self, cx: &mut Context<Self>) -> gpui_kit::AnyElement {
        button(
            SharedString::from("desk-hook-cycle"),
            self.hook_button_label(),
            false,
            cx.listener(move |this, _, _, cx| {
                this.cycle_selected_hook(cx);
                cx.stop_propagation();
                cx.notify();
            }),
        )
        .into_any_element()
    }

    pub(crate) fn hook_button_label(&self) -> String {
        match self.hook_choice.and_then(termy_x::trends_desk::hook_by_id) {
            Some(hook) => format!("Hook {:02} ▸ {}", hook.id, hook.name),
            None => "Hook ▸".into(),
        }
    }

    pub(crate) fn ensure_hook_choice(&mut self, post_id: &str) {
        let preferred = termy_x::trends_desk::cached_analysis(post_id)
            .map(|analysis| analysis.hooks)
            .unwrap_or_default();
        let text = self
            .columns
            .iter()
            .flat_map(|column| column.view.posts.iter())
            .find(|post| post.id == post_id)
            .map(|post| post.text.clone())
            .unwrap_or_default();
        let tone = termy_x::trends_desk::panel_tone_name(termy_x::trends_desk::TakeTone::parse(
            &self.pack.tone,
        ));
        self.hook_picks = termy_x::trends_desk::recommend_hooks(tone, &text, &preferred);
        self.hook_picks_for = post_id.to_string();
        let keep = self.hook_choice.filter(|id| self.hook_picks.contains(id));
        self.hook_choice = keep.or_else(|| self.hook_picks.first().copied());
    }

    pub(crate) fn cycle_selected_hook(&mut self, cx: &mut Context<Self>) {
        let Some(id) = self.popout.as_ref().map(|pop| pop.card_id.clone()) else {
            self.action_note = "Open a post first.".into();
            return;
        };
        self.ensure_hook_choice(&id);
        self.hook_choice = termy_x::trends_desk::next_hook_id(self.hook_choice, &self.hook_picks);
        if let Some(hook) = self.hook_choice.and_then(termy_x::trends_desk::hook_by_id) {
            self.action_note = format!("Hook {:02} {}: rewriting the take…", hook.id, hook.name);
        }
        self.request_another_take(cx);
    }

    pub(crate) fn take_note_msg(&mut self, card_id: String, msg: String) {
        if let Some((choice, picks)) = termy_x::trends_desk::parse_hook_picks_msg(&msg) {
            self.hook_choice = Some(choice);
            self.hook_picks = picks;
            self.hook_picks_for = card_id;
            return;
        }
        if msg.starts_with("Why it's popping:") {
            self.why_popping = msg.clone();
            self.why_popping_for = card_id;
        }
        self.action_note = msg;
    }
}
