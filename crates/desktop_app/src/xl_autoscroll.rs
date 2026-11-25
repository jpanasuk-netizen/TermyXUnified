//! Middle-click autoscroll and faster wheel scrolling on Trends Desk panes.

use std::time::{Duration, Instant};

use gpui_kit::{
    Context, InteractiveElement, IntoElement, MouseButton, MouseDownEvent, MouseMoveEvent,
    MouseUpEvent, ParentElement, ScrollWheelEvent, StatefulInteractiveElement, Styled, Window, div,
    point, px, rgb,
};
use termy_x::trends_desk::{
    AutoScroll, MARKER_SIZE, PANE_ROOT, TICK_MS, WHEEL_LINE_PX, clamp_offset, pane_ids,
};

use crate::trends_desk::TrendsDesk;

impl TrendsDesk {
    pub(crate) fn ensure_pane_scrolls(&mut self) {
        for id in pane_ids() {
            self.scroll_handles.entry((*id).into()).or_default();
        }
    }

    pub(crate) fn pane_handle(&self, id: &str) -> gpui_kit::ScrollHandle {
        self.scroll_handles.get(id).cloned().unwrap_or_default()
    }

    /// A scrollable pane: middle-click starts autoscroll, the wheel moves faster.
    pub(crate) fn scroll_pane(
        &self,
        id: &'static str,
        cx: &mut Context<Self>,
    ) -> gpui_kit::Stateful<gpui_kit::Div> {
        let handle = self.pane_handle(id);
        div()
            .id(id)
            .track_scroll(&handle)
            .overflow_y_scroll()
            .on_mouse_down(
                MouseButton::Middle,
                cx.listener(move |this, event: &MouseDownEvent, _, cx| {
                    this.begin_autoscroll(
                        id,
                        f32::from(event.position.x),
                        f32::from(event.position.y),
                        cx,
                    );
                    cx.stop_propagation();
                    cx.notify();
                }),
            )
            .on_scroll_wheel(cx.listener(move |this, event: &ScrollWheelEvent, _, cx| {
                if this.boost_wheel(id, event) {
                    cx.stop_propagation();
                    cx.notify();
                }
            }))
    }

    pub(crate) fn wrap_scroll_page(
        &self,
        id: &'static str,
        child: gpui_kit::AnyElement,
        cx: &mut Context<Self>,
    ) -> gpui_kit::AnyElement {
        self.scroll_pane(id, cx)
            .size_full()
            .min_h(px(0.0))
            .child(child)
            .into_any_element()
    }

    pub(crate) fn begin_autoscroll(&mut self, pane: &str, x: f32, y: f32, cx: &mut Context<Self>) {
        if self.autoscroll.is_some() {
            self.autoscroll = None;
            return;
        }
        self.autoscroll = Some(AutoScroll::start(pane, x, y));
        self.autoscroll_tick_at = Some(Instant::now());
        self.start_autoscroll_loop(cx);
    }

    pub(crate) fn stop_autoscroll(&mut self) {
        self.autoscroll = None;
        self.autoscroll_tick_at = None;
    }

    /// Capture-phase click: any button stops an active session and is eaten,
    /// matching a website. A fresh middle-click still reaches the pane after
    /// this returns false.
    pub(crate) fn capture_autoscroll_down(&mut self, event: &MouseDownEvent) -> bool {
        if self.autoscroll.is_none() {
            return false;
        }
        self.stop_autoscroll();
        let _ = event;
        true
    }

    pub(crate) fn on_autoscroll_move(&mut self, event: &MouseMoveEvent) {
        if let Some(scroll) = self.autoscroll.as_mut() {
            scroll.on_move(f32::from(event.position.x), f32::from(event.position.y));
        }
    }

    pub(crate) fn on_autoscroll_up(&mut self, event: &MouseUpEvent) {
        if event.button != MouseButton::Middle {
            return;
        }
        let keep = self
            .autoscroll
            .as_mut()
            .is_some_and(|scroll| scroll.on_middle_up());
        if !keep {
            self.stop_autoscroll();
        }
    }

    pub(crate) fn wire_autoscroll_root(
        &self,
        root: gpui_kit::Div,
        cx: &mut Context<Self>,
    ) -> gpui_kit::Stateful<gpui_kit::Div> {
        let handle = self.pane_handle(PANE_ROOT);
        root.id(PANE_ROOT)
            .track_scroll(&handle)
            .relative()
            .capture_any_mouse_down(cx.listener(|this, event: &MouseDownEvent, _, cx| {
                if this.capture_autoscroll_down(event) {
                    cx.stop_propagation();
                    cx.notify();
                }
            }))
            .on_mouse_move(cx.listener(|this, event: &MouseMoveEvent, _, cx| {
                if this.autoscroll.is_some() {
                    this.on_autoscroll_move(event);
                    cx.notify();
                }
            }))
            .on_mouse_up(
                MouseButton::Middle,
                cx.listener(|this, event: &MouseUpEvent, _, cx| {
                    if this.autoscroll.is_some() {
                        this.on_autoscroll_up(event);
                        cx.notify();
                    }
                }),
            )
    }

    pub(crate) fn autoscroll_marker(&self) -> Option<gpui_kit::AnyElement> {
        let scroll = self.autoscroll.as_ref()?;
        let (ox, oy) = self.desk_origin();
        let half = MARKER_SIZE / 2.0;
        let left = scroll.origin_x - ox - half;
        let top = scroll.origin_y - oy - half;
        Some(
            div()
                .id("desk-autoscroll-mark")
                .absolute()
                .left(px(left))
                .top(px(top))
                .w(px(MARKER_SIZE))
                .h(px(MARKER_SIZE))
                .rounded(px(MARKER_SIZE / 2.0))
                .border_1()
                .border_color(rgb(0xc7c7cc))
                .bg(rgb(0x1c1c1e))
                .flex()
                .items_center()
                .justify_center()
                .text_size(px(11.0))
                .text_color(rgb(0xe5e5ea))
                .child("▲▼")
                .into_any_element(),
        )
    }

    fn desk_origin(&self) -> (f32, f32) {
        let origin = self.pane_handle(PANE_ROOT).bounds().origin;
        (f32::from(origin.x), f32::from(origin.y))
    }

    fn boost_wheel(&self, id: &str, event: &ScrollWheelEvent) -> bool {
        let handle = self.pane_handle(id);
        let delta = event.delta.pixel_delta(px(WHEEL_LINE_PX));
        let cur = handle.offset();
        let max = handle.max_offset();
        let next = point(
            px(clamp_offset(
                f32::from(cur.x),
                f32::from(delta.x),
                f32::from(max.x),
            )),
            px(clamp_offset(
                f32::from(cur.y),
                f32::from(delta.y),
                f32::from(max.y),
            )),
        );
        if next == cur {
            return false;
        }
        handle.set_offset(next);
        true
    }

    fn start_autoscroll_loop(&mut self, cx: &mut Context<Self>) {
        if self.autoscroll_ticking {
            return;
        }
        self.autoscroll_ticking = true;
        cx.spawn(async move |this, cx| {
            loop {
                cx.background_executor()
                    .timer(Duration::from_millis(TICK_MS))
                    .await;
                let keep = this
                    .update(cx, |desk, cx| desk.tick_autoscroll(cx))
                    .ok()
                    .unwrap_or(false);
                if !keep {
                    break;
                }
            }
            let _ = this.update(cx, |desk, _cx| {
                desk.autoscroll_ticking = false;
            });
        })
        .detach();
    }

    fn tick_autoscroll(&mut self, cx: &mut Context<Self>) -> bool {
        let Some(scroll) = self.autoscroll.clone() else {
            return false;
        };
        let now = Instant::now();
        let dt = self
            .autoscroll_tick_at
            .map(|at| now.saturating_duration_since(at).as_secs_f32())
            .unwrap_or(TICK_MS as f32 / 1000.0)
            .clamp(0.0, 0.05);
        self.autoscroll_tick_at = Some(now);
        let (dx, dy) = scroll.tick_delta(dt);
        if dx != 0.0 || dy != 0.0 {
            let handle = self.pane_handle(&scroll.pane);
            let cur = handle.offset();
            let max = handle.max_offset();
            handle.set_offset(point(
                px(clamp_offset(f32::from(cur.x), dx, f32::from(max.x))),
                px(clamp_offset(f32::from(cur.y), dy, f32::from(max.y))),
            ));
            cx.notify();
        }
        true
    }
}
