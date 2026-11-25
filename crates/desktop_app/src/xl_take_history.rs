//! Earlier takes from Funny / Smart / Spicy, shown under the pink box.

use gpui_kit::{
    Context, InteractiveElement, IntoElement, ParentElement, SharedString, Styled, div, px, rgb,
};
use termy_x::trends_desk::{begin_tone_replace, take_should_be_archived, takes_for_card};

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
    pub(crate) fn take_history_row(&self, cx: &mut Context<Self>) -> gpui_kit::AnyElement {
        let Some(id) = self.popout.as_ref().map(|pop| pop.card_id.clone()) else {
            return div().into_any_element();
        };
        let list = takes_for_card(&self.pack, &id);
        if list.is_empty() {
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
                    list.len()
                )),
        );
        for (index, take) in list.iter().enumerate().rev() {
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
