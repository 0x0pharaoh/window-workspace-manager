//! Pure coordinate math: normalized (0..1) rects <-> native pixels.
//!
//! No Windows API here — everything is unit-testable on any OS. Handles
//! negative monitor origins, portrait layouts and zero-size guards.

use serde::{Deserialize, Serialize};

use crate::models::MonitorInfo;

/// Native pixel rectangle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct NativeRect {
    pub x: i32,
    pub y: i32,
    pub w: i32,
    pub h: i32,
}

impl NativeRect {
    /// Overlap area with another rect (0 when disjoint or empty).
    /// Uses i64 math so negative origins cannot overflow.
    pub fn overlap(&self, other: &NativeRect) -> i64 {
        let x1 = self.x.max(other.x) as i64;
        let y1 = self.y.max(other.y) as i64;
        let x2 = (self.x as i64 + self.w as i64).min(other.x as i64 + other.w as i64);
        let y2 = (self.y as i64 + self.h as i64).min(other.y as i64 + other.h as i64);
        let w = x2 - x1;
        let h = y2 - y1;
        if w > 0 && h > 0 {
            w * h
        } else {
            0
        }
    }
}

pub fn clamp01(v: f64) -> f64 {
    if !v.is_finite() {
        0.0
    } else {
        v.clamp(0.0, 1.0)
    }
}

/// Clamp a normalized rect: NaN-safe, w/h in 0.05..1, x/y fitted inside.
pub fn clamp_normalized(nx: f64, ny: f64, nw: f64, nh: f64) -> (f64, f64, f64, f64) {
    let w = if !nw.is_finite() {
        0.5
    } else {
        nw.clamp(0.05, 1.0)
    };
    let h = if !nh.is_finite() {
        0.5
    } else {
        nh.clamp(0.05, 1.0)
    };
    let x = clamp01(nx).min(1.0 - w);
    let y = clamp01(ny).min(1.0 - h);
    (x, y, w, h)
}

/// Convert a normalized rect to native pixels inside a monitor work area.
/// Takes eight scalars so call sites stay readable; the lint threshold is
/// deliberately opted out for this coordinate tuple.
#[allow(clippy::too_many_arguments)]
pub fn normalized_to_native(
    nx: f64,
    ny: f64,
    nw: f64,
    nh: f64,
    work_x: i32,
    work_y: i32,
    work_w: i32,
    work_h: i32,
) -> NativeRect {
    if work_w <= 0 || work_h <= 0 {
        return NativeRect {
            x: work_x,
            y: work_y,
            w: 0,
            h: 0,
        };
    }
    let (x, y, w, h) = clamp_normalized(nx, ny, nw, nh);
    NativeRect {
        x: work_x + (x * work_w as f64).round() as i32,
        y: work_y + (y * work_h as f64).round() as i32,
        w: (w * work_w as f64).round() as i32,
        h: (h * work_h as f64).round() as i32,
    }
}

pub fn monitor_full_rect(m: &MonitorInfo) -> NativeRect {
    NativeRect {
        x: m.x,
        y: m.y,
        w: m.w,
        h: m.h,
    }
}

pub fn monitor_work_rect(m: &MonitorInfo) -> NativeRect {
    if m.work_w > 0 && m.work_h > 0 {
        NativeRect {
            x: m.work_x,
            y: m.work_y,
            w: m.work_w,
            h: m.work_h,
        }
    } else {
        monitor_full_rect(m)
    }
}

/// Pick the monitor with the largest overlap with `rect`.
/// Falls back to the primary (or first) monitor when there is no overlap.
pub fn pick_monitor_for_rect(monitors: &[MonitorInfo], rect: NativeRect) -> String {
    if monitors.is_empty() {
        return String::new();
    }
    if rect.w <= 0 || rect.h <= 0 {
        return fallback_monitor(monitors).unwrap_or_default();
    }
    let mut best_id = String::new();
    let mut best_area: i64 = 0;
    for m in monitors {
        let area = rect.overlap(&monitor_full_rect(m));
        if area > best_area {
            best_area = area;
            best_id = m.id.clone();
        }
    }
    if best_area > 0 {
        best_id
    } else {
        fallback_monitor(monitors).unwrap_or_default()
    }
}

/// Primary monitor id, or the first monitor when none is marked primary.
pub fn fallback_monitor(monitors: &[MonitorInfo]) -> Option<String> {
    monitors
        .iter()
        .find(|m| m.is_primary)
        .or(monitors.first())
        .map(|m| m.id.clone())
}

/// True when any part of `rect` is visible on any monitor.
pub fn is_rect_visible_on_any_monitor(monitors: &[MonitorInfo], rect: NativeRect) -> bool {
    if rect.w <= 0 || rect.h <= 0 {
        return false;
    }
    monitors
        .iter()
        .any(|m| rect.overlap(&monitor_full_rect(m)) > 0)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn mon(id: &str, x: i32, y: i32, w: i32, h: i32, primary: bool) -> MonitorInfo {
        MonitorInfo {
            id: id.to_owned(),
            x,
            y,
            w,
            h,
            work_x: x,
            work_y: y,
            work_w: w,
            work_h: h,
            scale: 1.0,
            is_primary: primary,
            name: id.to_owned(),
        }
    }

    #[test]
    fn forward_center() {
        let r = normalized_to_native(0.25, 0.25, 0.5, 0.5, 0, 0, 1920, 1080);
        assert_eq!(
            r,
            NativeRect {
                x: 480,
                y: 270,
                w: 960,
                h: 540
            }
        );
    }

    #[test]
    fn negative_origin_monitor() {
        let mons = vec![
            mon("left", -1920, 0, 1920, 1080, false),
            mon("main", 0, 0, 1920, 1080, true),
        ];
        let r = normalized_to_native(0.0, 0.0, 1.0, 1.0, -1920, 0, 1920, 1080);
        assert_eq!(r.x, -1920);
        assert_eq!(pick_monitor_for_rect(&mons, r), "left");
    }

    #[test]
    fn portrait_monitor_math() {
        // 1080x1920 portrait panel as secondary
        let mons = vec![
            mon("main", 0, 0, 1920, 1080, true),
            mon("portrait", 1920, -420, 1080, 1920, false),
        ];
        let r = normalized_to_native(0.0, 0.0, 0.5, 0.5, 1920, -420, 1080, 1920);
        assert_eq!(
            r,
            NativeRect {
                x: 1920,
                y: -420,
                w: 540,
                h: 960
            }
        );
        assert_eq!(pick_monitor_for_rect(&mons, r), "portrait");
        assert!(is_rect_visible_on_any_monitor(&mons, r));
    }

    #[test]
    fn zero_size_guards() {
        let r = normalized_to_native(0.5, 0.5, 0.5, 0.5, 0, 0, 0, 0);
        assert_eq!(r.w, 0);
        assert!(!is_rect_visible_on_any_monitor(&[], r));
        assert_eq!(pick_monitor_for_rect(&[], r), "");
    }

    #[test]
    fn clamp_handles_garbage() {
        assert_eq!(
            clamp_normalized(f64::NAN, -2.0, 99.0, 0.0),
            (0.0, 0.0, 1.0, 0.05)
        );
        let r = normalized_to_native(-0.5, -0.5, 2.0, 2.0, 0, 0, 1000, 800);
        assert_eq!(
            r,
            NativeRect {
                x: 0,
                y: 0,
                w: 1000,
                h: 800
            }
        );
    }

    #[test]
    fn fallback_prefers_primary() {
        let mons = vec![
            mon("a", 0, 0, 800, 600, false),
            mon("b", 800, 0, 800, 600, true),
        ];
        assert_eq!(fallback_monitor(&mons).as_deref(), Some("b"));
        // rect far away from everything -> fallback
        let far = NativeRect {
            x: 9000,
            y: 9000,
            w: 100,
            h: 100,
        };
        assert_eq!(pick_monitor_for_rect(&mons, far), "b");
        assert!(!is_rect_visible_on_any_monitor(&mons, far));
    }

    #[test]
    fn overlap_picks_largest_area() {
        let mons = vec![
            mon("a", 0, 0, 1000, 1000, true),
            mon("b", 1000, 0, 1000, 1000, false),
        ];
        // straddles both, mostly on b
        let r = NativeRect {
            x: 900,
            y: 0,
            w: 500,
            h: 500,
        };
        assert_eq!(pick_monitor_for_rect(&mons, r), "b");
        assert!(is_rect_visible_on_any_monitor(&mons, r));
    }
}
