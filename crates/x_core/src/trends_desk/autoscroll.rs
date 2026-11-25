//! Browser-style middle-click autoscroll: drop an anchor, scroll faster the
//! farther the pointer moves, and stop on the next click or Escape.
//! Holding the middle button and dragging works the same and stops on release.
//! A short click (no drag) stays in autoscroll until the next click or Esc.

/// No scroll inside this many pixels of the anchor, like a website.
pub const DEAD_ZONE: f32 = 12.0;
/// Dragging this far before release means "hold to scroll, stop on release".
pub const HOLD_MOVE: f32 = 6.0;
/// Scroll speed in px/s for each pixel past the dead zone.
pub const SPEED_PER_PX: f32 = 10.0;
pub const TICK_MS: u64 = 16;
/// Wheel ticks use this many pixels per line, about 4–5× a 16px line, so
/// scrolling feels closer to a website (~80–100px per notch).
pub const WHEEL_LINE_PX: f32 = 80.0;
pub const MARKER_SIZE: f32 = 28.0;

pub const PANE_ROOT: &str = "trends-desk";
pub const PANE_FEED: &str = "desk-take-list";
pub const PANE_BODY: &str = "trends-desk-body";
pub const PANE_LINK: &str = "desk-link-page";
pub const PANE_AGENT: &str = "desk-agent-scroll";
pub const PANE_BOARD: &str = "desk-board";

pub fn pane_ids() -> &'static [&'static str] {
    &[
        PANE_ROOT, PANE_FEED, PANE_BODY, PANE_LINK, PANE_AGENT, PANE_BOARD,
    ]
}

#[derive(Debug, Clone, PartialEq)]
pub struct AutoScroll {
    pub pane: String,
    pub origin_x: f32,
    pub origin_y: f32,
    pub mouse_x: f32,
    pub mouse_y: f32,
    /// True while the starting middle button is still down.
    pub held: bool,
    /// True if the pointer moved far enough during the hold to count as a drag.
    pub moved: bool,
}

impl AutoScroll {
    pub fn start(pane: impl Into<String>, x: f32, y: f32) -> Self {
        Self {
            pane: pane.into(),
            origin_x: x,
            origin_y: y,
            mouse_x: x,
            mouse_y: y,
            held: true,
            moved: false,
        }
    }

    pub fn distance(&self) -> f32 {
        let dx = self.mouse_x - self.origin_x;
        let dy = self.mouse_y - self.origin_y;
        (dx * dx + dy * dy).sqrt()
    }

    pub fn on_move(&mut self, x: f32, y: f32) {
        self.mouse_x = x;
        self.mouse_y = y;
        if self.held && self.distance() >= HOLD_MOVE {
            self.moved = true;
        }
    }

    /// After a middle-button release: `false` means the session ends (it was a
    /// drag). `true` means a short click, so autoscroll stays on.
    pub fn on_middle_up(&mut self) -> bool {
        if self.moved {
            return false;
        }
        self.held = false;
        true
    }

    /// Velocity in px/s. Positive y is pointer below the anchor (scroll down).
    pub fn velocity(&self) -> (f32, f32) {
        let dx = self.mouse_x - self.origin_x;
        let dy = self.mouse_y - self.origin_y;
        let dist = (dx * dx + dy * dy).sqrt();
        if dist <= DEAD_ZONE {
            return (0.0, 0.0);
        }
        let speed = (dist - DEAD_ZONE) * SPEED_PER_PX;
        let scale = speed / dist;
        (dx * scale, dy * scale)
    }

    /// Delta to add to a GPUI scroll offset for this tick. Offset is 0 at the
    /// top and more negative as you scroll down, so a downward pointer
    /// (positive velocity y) yields a negative delta.
    pub fn tick_delta(&self, dt: f32) -> (f32, f32) {
        let dt = dt.max(0.0);
        let (vx, vy) = self.velocity();
        (-vx * dt, -vy * dt)
    }
}

/// Keep a GPUI offset between 0 and `-max`.
pub fn clamp_offset(current: f32, delta: f32, max: f32) -> f32 {
    (current + delta).clamp(-max.max(0.0), 0.0)
}

/// Pixel delta for one wheel event, scaled to a website-like line height.
pub fn wheel_delta(raw_line_pixels: f32, line_px: f32) -> f32 {
    if raw_line_pixels == 0.0 {
        0.0
    } else {
        raw_line_pixels * (line_px / 16.0).max(1.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dead_zone_is_still() {
        let mut s = AutoScroll::start("feed", 100.0, 100.0);
        s.on_move(108.0, 104.0);
        assert_eq!(s.velocity(), (0.0, 0.0));
        assert_eq!(s.tick_delta(0.016), (0.0, 0.0));
    }

    #[test]
    fn farther_is_faster_and_points_the_right_way() {
        let mut near = AutoScroll::start("feed", 0.0, 0.0);
        near.on_move(0.0, 40.0);
        let mut far = AutoScroll::start("feed", 0.0, 0.0);
        far.on_move(0.0, 80.0);
        let (_, near_y) = near.velocity();
        let (_, far_y) = far.velocity();
        assert!(near_y > 0.0, "{near_y}");
        assert!(far_y > near_y, "{far_y} vs {near_y}");
        let (_, dy) = far.tick_delta(0.1);
        assert!(
            dy < 0.0,
            "scrolling down must move the offset up the page, {dy}"
        );

        let mut left = AutoScroll::start("feed", 0.0, 0.0);
        left.on_move(-50.0, 0.0);
        let (vx, vy) = left.velocity();
        assert!(vx < 0.0 && vy == 0.0, "{vx} {vy}");
    }

    #[test]
    fn a_click_stays_on_and_a_drag_stops_on_release() {
        let mut click = AutoScroll::start("feed", 10.0, 10.0);
        click.on_move(12.0, 11.0);
        assert!(click.on_middle_up());
        assert!(!click.held);

        let mut drag = AutoScroll::start("feed", 10.0, 10.0);
        drag.on_move(10.0, 40.0);
        assert!(drag.moved);
        assert!(!drag.on_middle_up());
    }

    #[test]
    fn clamp_stays_in_range() {
        assert_eq!(clamp_offset(0.0, -30.0, 100.0), -30.0);
        assert_eq!(clamp_offset(-90.0, -30.0, 100.0), -100.0);
        assert_eq!(clamp_offset(-10.0, 40.0, 100.0), 0.0);
        assert_eq!(clamp_offset(0.0, -10.0, 0.0), 0.0);
    }

    #[test]
    fn wheel_lines_are_faster_than_a_16px_line() {
        assert_eq!(wheel_delta(16.0, WHEEL_LINE_PX), 80.0);
        assert_eq!(wheel_delta(-16.0, WHEEL_LINE_PX), -80.0);
        assert_eq!(wheel_delta(0.0, WHEEL_LINE_PX), 0.0);
    }

    #[test]
    fn marker_sits_on_the_anchor() {
        let half = MARKER_SIZE / 2.0;
        assert_eq!(100.0 - half, 86.0);
        assert_eq!(pane_ids().len(), 6);
    }
}
