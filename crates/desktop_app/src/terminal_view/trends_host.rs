//! TermyXL as an in-window docked pane of Termy X.
//!
//! The TrendsDesk entity is created in-process and mounted as a right-hand
//! sibling of the terminal pane in `render.rs`. It renders bare and at the
//! same width as the standalone window (`LOCKED_OPEN_WIDTH`), so it looks
//! exactly like standalone Termy XL — just docked inside Termy X.

use gpui_kit::{
    AppContext, Context, FontWeight, InteractiveElement, IntoElement, MouseButton, ParentElement,
    StatefulInteractiveElement, Styled, div, px, rgb,
};
use termy::trends_desk::{DeskOptions, TrendsDesk};

use super::TerminalView;

impl TerminalView {
    pub(super) fn toggle_trends_desk(&mut self, cx: &mut Context<Self>) {
        self.ensure_trends_desk(cx);
        self.trends_desk_open = !self.trends_desk_open;
        let width = self.trends_desk_width;
        if let Some(desk) = &self.trends_desk {
            desk.update(cx, |desk, _| {
                desk.hosted = true;
                desk.host_width = width;
            });
        }
        cx.notify();
    }

    /// Horizontal space the TermyXL rail + (optional) docked pane take from
    /// the terminal grid. The rail is always present, so it always reserves;
    /// the pane adds its width only while open. Must match the row layout,
    /// otherwise the PTY grid is computed too wide and the text mis-fits.
    pub(super) fn trends_desk_reserved_width(&self) -> f32 {
        const RAIL_WIDTH: f32 = 48.0;
        if self.trends_desk_open && self.trends_desk.is_some() {
            RAIL_WIDTH + self.trends_desk_width
        } else {
            RAIL_WIDTH
        }
    }

    fn ensure_trends_desk(&mut self, cx: &mut Context<Self>) {
        if self.trends_desk.is_none() {
            let desk = cx.new(|cx| TrendsDesk::new(DeskOptions::default(), cx));
            let width = self.trends_desk_width;
            desk.update(cx, |desk, _| {
                desk.hosted = true;
                desk.host_width = width;
            });
            self.trends_desk = Some(desk);
        }
    }

    /// The docked pane, mounted to the right of the terminal when open.
    /// Rendered bare (no extra frame) so it matches standalone Termy XL.
    pub(super) fn trends_desk_pane(
        &self,
        _cx: &mut Context<Self>,
    ) -> Option<gpui_kit::AnyElement> {
        if !self.trends_desk_open {
            return None;
        }
        let desk = self.trends_desk.clone()?;
        let width = self.trends_desk_width;
        Some(
            div()
                .id("trends-desk-pane")
                .flex_none()
                .h_full()
                .w(px(width))
                .overflow_hidden()
                .bg(rgb(0x070b14))
                .child(desk)
                .into_any_element(),
        )
    }

    /// The slim "XL" rail that toggles the pane. Always visible so it doubles
    /// as the close affordance (no overlay chrome over the desk).
    pub(super) fn trends_desk_rail(
        &self,
        cx: &mut Context<Self>,
    ) -> Option<gpui_kit::AnyElement> {
        let active = self.trends_desk_open;
        let mark = div()
            .id("trends-desk-rail-mark")
            .w(px(40.0))
            .h(px(40.0))
            .rounded(px(12.0))
            .flex()
            .items_center()
            .justify_center()
            .cursor_pointer()
            .bg(if active { rgb(0x2a1030) } else { rgb(0x10243a) })
            .text_color(if active { rgb(0xff4fa3) } else { rgb(0x7af6ff) })
            .border_1()
            .border_color(if active { rgb(0xff4fa3) } else { rgb(0x7af6ff) })
            .text_size(px(13.0))
            .font_weight(FontWeight::BOLD)
            .hover(|s| {
                s.bg(rgb(0x2a1030))
                    .text_color(rgb(0xff4fa3))
                    .border_color(rgb(0xff4fa3))
            })
            .child("XL")
            .on_mouse_down(
                MouseButton::Left,
                cx.listener(|this, _: &gpui_kit::MouseDownEvent, _window, cx| {
                    this.toggle_trends_desk(cx);
                    cx.stop_propagation();
                }),
            );

        Some(
            div()
                .id("trends-desk-rail")
                .flex_none()
                .h_full()
                .w(px(48.0))
                .flex()
                .flex_col()
                .items_center()
                .pt(px(10.0))
                .bg(rgb(0x0d1117))
                .border_l_1()
                .border_color(rgb(0x1a2535))
                .child(mark)
                .into_any_element(),
        )
    }
}
