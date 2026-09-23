//! Desktop capture: enumerate windows and assign each to its monitor.

use crate::coords::{pick_monitor_for_rect, NativeRect};
use crate::models::CapturedWindow;

/// Enumerate visible windows with `monitor_id` assigned via max overlap.
pub fn capture_desktop() -> Vec<CapturedWindow> {
    let monitors = crate::monitors::get_monitors();
    let mut wins = crate::windows::enum_windows();
    for w in wins.iter_mut() {
        let rect = NativeRect {
            x: w.x,
            y: w.y,
            w: w.w,
            h: w.h,
        };
        w.monitor_id = pick_monitor_for_rect(&monitors, rect);
    }
    wins.sort_by(|a, b| {
        a.monitor_id
            .cmp(&b.monitor_id)
            .then_with(|| a.title.to_lowercase().cmp(&b.title.to_lowercase()))
    });
    wins
}
