//! Monitor enumeration (position, work area, DPI scale, primary flag).
//!
//! Real implementation on Windows; a single 1920x1080 mock primary on
//! other platforms so layout logic stays testable everywhere.

#[cfg(windows)]
mod platform {
    use windows::Win32::Foundation::{BOOL, LPARAM, RECT, TRUE};
    use windows::Win32::Graphics::Gdi::{
        EnumDisplayMonitors, GetMonitorInfoW, HDC, HMONITOR, MONITORINFO, MONITORINFOEXW,
    };
    use windows::Win32::UI::HiDpi::{GetDpiForMonitor, MDT_EFFECTIVE_DPI};
    use windows::Win32::UI::WindowsAndMessaging::MONITORINFOF_PRIMARY;

    use crate::models::MonitorInfo;

    struct Ctx {
        mons: Vec<HMONITOR>,
    }

    unsafe extern "system" fn mon_cb(hm: HMONITOR, _hdc: HDC, _rc: *mut RECT, lp: LPARAM) -> BOOL {
        if let Some(ctx) = (lp.0 as *mut Ctx).as_mut() {
            ctx.mons.push(hm);
        }
        TRUE
    }

    pub fn mock_primary() -> MonitorInfo {
        MonitorInfo {
            id: "primary".to_owned(),
            x: 0,
            y: 0,
            w: 1920,
            h: 1080,
            work_x: 0,
            work_y: 0,
            work_w: 1920,
            work_h: 1040,
            scale: 1.0,
            is_primary: true,
            name: "Primary".to_owned(),
        }
    }

    fn device_name(raw: &[u16; 32]) -> String {
        let len = raw.iter().position(|&c| c == 0).unwrap_or(raw.len());
        String::from_utf16_lossy(&raw[..len])
    }

    fn describe_monitor(hm: HMONITOR, index: usize) -> Option<MonitorInfo> {
        let zero = RECT {
            left: 0,
            top: 0,
            right: 0,
            bottom: 0,
        };
        let mut ex = MONITORINFOEXW {
            monitorInfo: MONITORINFO {
                cbSize: std::mem::size_of::<MONITORINFOEXW>() as u32,
                rcMonitor: zero,
                rcWork: zero,
                dwFlags: 0,
            },
            szDevice: [0u16; 32],
        };
        let ok = unsafe { GetMonitorInfoW(hm, &mut ex as *mut MONITORINFOEXW as *mut MONITORINFO) }
            .as_bool();
        if !ok {
            return None;
        }
        let full = ex.monitorInfo.rcMonitor;
        let work = ex.monitorInfo.rcWork;
        let mut dpi_x = 0u32;
        let mut dpi_y = 0u32;
        let scale = unsafe { GetDpiForMonitor(hm, MDT_EFFECTIVE_DPI, &mut dpi_x, &mut dpi_y) }
            .map(|_| dpi_x as f64 / 96.0)
            .unwrap_or(1.0);
        let scale = if scale > 0.0 { scale } else { 1.0 };
        let name = device_name(&ex.szDevice);
        let id = if name.trim().is_empty() {
            format!("monitor-{index}")
        } else {
            name.clone()
        };
        Some(MonitorInfo {
            id,
            x: full.left,
            y: full.top,
            w: (full.right - full.left).max(0),
            h: (full.bottom - full.top).max(0),
            work_x: work.left,
            work_y: work.top,
            work_w: (work.right - work.left).max(0),
            work_h: (work.bottom - work.top).max(0),
            scale,
            is_primary: (ex.monitorInfo.dwFlags & MONITORINFOF_PRIMARY) != 0,
            name,
        })
    }

    pub fn get_monitors() -> Vec<MonitorInfo> {
        let mut ctx = Ctx { mons: Vec::new() };
        let ok = unsafe {
            EnumDisplayMonitors(
                HDC(std::ptr::null_mut()),
                None,
                Some(mon_cb),
                LPARAM(&mut ctx as *mut Ctx as isize),
            )
        }
        .as_bool();
        if !ok {
            return vec![mock_primary()];
        }
        let mut out: Vec<MonitorInfo> = ctx
            .mons
            .iter()
            .enumerate()
            .filter_map(|(i, hm)| describe_monitor(*hm, i))
            .collect();
        if out.is_empty() {
            return vec![mock_primary()];
        }
        if !out.iter().any(|m| m.is_primary) {
            if let Some(first) = out.first_mut() {
                first.is_primary = true;
            }
        }
        out
    }
}

#[cfg(not(windows))]
mod platform {
    use crate::models::MonitorInfo;

    pub fn get_monitors() -> Vec<MonitorInfo> {
        vec![MonitorInfo {
            id: "primary".to_owned(),
            x: 0,
            y: 0,
            w: 1920,
            h: 1080,
            work_x: 0,
            work_y: 0,
            work_w: 1920,
            work_h: 1040,
            scale: 1.0,
            is_primary: true,
            name: "Primary".to_owned(),
        }]
    }
}

pub use platform::get_monitors;
