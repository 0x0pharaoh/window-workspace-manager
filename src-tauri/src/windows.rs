//! Window enumeration and manipulation.
//!
//! Real implementation on Windows (via the `windows` crate); on other
//! platforms every function degrades to an empty result / descriptive error
//! so `cargo test` passes anywhere.

#[cfg(windows)]
mod platform {
    use windows::Win32::Foundation::{
        CloseHandle, BOOL, FALSE, HANDLE, HMODULE, HWND, LPARAM, RECT, TRUE, WPARAM,
    };
    use windows::Win32::Graphics::Dwm::{DwmGetWindowAttribute, DWMWA_CLOAKED};
    use windows::Win32::System::ProcessStatus::GetModuleFileNameExW;
    use windows::Win32::System::Threading::{
        GetCurrentProcessId, OpenProcess, PROCESS_QUERY_INFORMATION, PROCESS_VM_READ,
    };
    use windows::Win32::UI::WindowsAndMessaging::{
        EnumWindows, GetClassNameW, GetWindowRect, GetWindowTextW, GetWindowThreadProcessId,
        IsIconic, IsWindow, IsWindowVisible, IsZoomed, PostMessageW, SetForegroundWindow,
        SetWindowPos, ShowWindow, HWND_TOP, SWP_NOACTIVATE, SWP_NOZORDER, SWP_SHOWWINDOW,
        SW_MAXIMIZE, SW_MINIMIZE, SW_RESTORE, WM_CLOSE,
    };

    use crate::error::WorksetError;
    use crate::models::CapturedWindow;

    /// Window classes that are never user apps.
    const SYSTEM_CLASSES: &[&str] = &[
        "Shell_TrayWnd",
        "DV2ControlHost",
        "MsgrIMEWindowClass",
        "SysShadow",
        "Progman",
        "WorkerW",
        "Windows.UI.Core.CoreWindow",
        "TaskListThumbnailWnd",
        "CiceroUIWndFrame",
        "Xaml_WindowedPopupClass",
        "ApplicationFrameWindow",
    ];

    struct EnumCtx {
        hwnds: Vec<HWND>,
    }

    unsafe extern "system" fn enum_cb(hwnd: HWND, lp: LPARAM) -> BOOL {
        if let Some(ctx) = (lp.0 as *mut EnumCtx).as_mut() {
            ctx.hwnds.push(hwnd);
        }
        TRUE
    }

    fn window_text(hwnd: HWND) -> String {
        let mut buf = [0u16; 512];
        let len = unsafe { GetWindowTextW(hwnd, &mut buf) };
        if len <= 0 {
            String::new()
        } else {
            String::from_utf16_lossy(&buf[..len as usize])
        }
    }

    fn window_class(hwnd: HWND) -> String {
        let mut buf = [0u16; 256];
        let len = unsafe { GetClassNameW(hwnd, &mut buf) };
        if len <= 0 {
            String::new()
        } else {
            String::from_utf16_lossy(&buf[..len as usize])
        }
    }

    fn is_cloaked(hwnd: HWND) -> bool {
        let mut cloaked: u32 = 0;
        let ok = unsafe {
            DwmGetWindowAttribute(
                hwnd,
                DWMWA_CLOAKED,
                &mut cloaked as *mut u32 as *mut core::ffi::c_void,
                std::mem::size_of::<u32>() as u32,
            )
        };
        ok.is_ok() && cloaked != 0
    }

    fn exe_path_for_pid(pid: u32) -> String {
        if pid == 0 {
            return String::new();
        }
        unsafe {
            let handle: HANDLE =
                match OpenProcess(PROCESS_QUERY_INFORMATION | PROCESS_VM_READ, FALSE, pid) {
                    Ok(h) => h,
                    Err(_) => return String::new(),
                };
            let mut buf = [0u16; 1024];
            let len = GetModuleFileNameExW(handle, HMODULE(std::ptr::null_mut()), &mut buf);
            let _ = CloseHandle(handle);
            if len == 0 {
                String::new()
            } else {
                String::from_utf16_lossy(&buf[..len as usize])
            }
        }
    }

    fn describe_window(hwnd: HWND, own_pid: u32) -> Option<CapturedWindow> {
        unsafe {
            if !IsWindowVisible(hwnd).as_bool() {
                return None;
            }
            let title = window_text(hwnd);
            if title.trim().is_empty() {
                return None;
            }
            let class = window_class(hwnd);
            if SYSTEM_CLASSES.iter().any(|s| *s == class) {
                return None;
            }
            let mut rect = RECT {
                left: 0,
                top: 0,
                right: 0,
                bottom: 0,
            };
            if GetWindowRect(hwnd, &mut rect).is_err() {
                return None;
            }
            let w = rect.right - rect.left;
            let h = rect.bottom - rect.top;
            if w <= 0 || h <= 0 {
                return None;
            }
            let mut pid: u32 = 0;
            GetWindowThreadProcessId(hwnd, Some(&mut pid as *mut u32));
            if pid == own_pid {
                return None;
            }
            if is_cloaked(hwnd) {
                return None;
            }
            let state = if IsIconic(hwnd).as_bool() {
                "minimized"
            } else if IsZoomed(hwnd).as_bool() {
                "maximized"
            } else {
                "normal"
            }
            .to_owned();
            Some(CapturedWindow {
                hwnd: hwnd.0 as i64,
                pid,
                exe_path: exe_path_for_pid(pid),
                title,
                class,
                x: rect.left,
                y: rect.top,
                w,
                h,
                monitor_id: String::new(),
                state,
            })
        }
    }

    /// Enumerate visible, non-system top-level windows.
    pub fn enum_windows() -> Vec<CapturedWindow> {
        let own_pid = unsafe { GetCurrentProcessId() };
        let mut ctx = EnumCtx { hwnds: Vec::new() };
        unsafe {
            let _ = EnumWindows(Some(enum_cb), LPARAM(&mut ctx as *mut EnumCtx as isize));
        }
        let mut out = Vec::with_capacity(ctx.hwnds.len());
        for hwnd in ctx.hwnds {
            if let Some(w) = describe_window(hwnd, own_pid) {
                out.push(w);
            }
        }
        out
    }

    /// Snapshot of current windows, used to diff "new" windows after launch.
    pub fn snapshot_before_launch() -> Vec<CapturedWindow> {
        enum_windows()
    }

    /// Find one enumerated window by handle.
    pub fn find_window(hwnd: i64) -> Option<CapturedWindow> {
        enum_windows().into_iter().find(|w| w.hwnd == hwnd)
    }

    /// Move/resize a window and apply the requested state.
    pub fn move_window(
        hwnd: i64,
        x: i32,
        y: i32,
        w: i32,
        h: i32,
        state: &str,
    ) -> Result<(), WorksetError> {
        let hwnd = HWND(hwnd as *mut core::ffi::c_void);
        let w = w.max(1);
        let h = h.max(1);
        unsafe {
            if !IsWindow(hwnd).as_bool() {
                return Err(WorksetError::Windows("window no longer exists".to_owned()));
            }
            match state {
                "minimized" => {
                    let _ = ShowWindow(hwnd, SW_MINIMIZE);
                }
                "maximized" | "fullscreen" => {
                    // Best effort: true fullscreen (exclusive/borderless) needs
                    // app cooperation; maximize is the sane approximation.
                    let _ = ShowWindow(hwnd, SW_MAXIMIZE);
                }
                _ => {
                    let _ = ShowWindow(hwnd, SW_RESTORE);
                    SetWindowPos(
                        hwnd,
                        HWND_TOP,
                        x,
                        y,
                        w,
                        h,
                        SWP_NOZORDER | SWP_NOACTIVATE | SWP_SHOWWINDOW,
                    )
                    .map_err(|e| WorksetError::Windows(format!("SetWindowPos failed: {e}")))?;
                }
            }
        }
        Ok(())
    }

    pub fn is_window_valid(hwnd: i64) -> bool {
        unsafe { IsWindow(HWND(hwnd as *mut core::ffi::c_void)).as_bool() }
    }

    pub fn focus_window(hwnd: i64) -> Result<(), WorksetError> {
        let hwnd = HWND(hwnd as *mut core::ffi::c_void);
        unsafe {
            if !IsWindow(hwnd).as_bool() {
                return Err(WorksetError::Windows("window no longer exists".to_owned()));
            }
            if IsIconic(hwnd).as_bool() {
                let _ = ShowWindow(hwnd, SW_RESTORE);
            }
            if !SetForegroundWindow(hwnd).as_bool() {
                return Err(WorksetError::Windows(
                    "failed to bring window to the foreground".to_owned(),
                ));
            }
        }
        Ok(())
    }

    /// Ask a window to close itself (WM_CLOSE). Never kills the process.
    pub fn close_window(hwnd: i64) -> Result<(), WorksetError> {
        let hwnd = HWND(hwnd as *mut core::ffi::c_void);
        unsafe {
            if !IsWindow(hwnd).as_bool() {
                return Err(WorksetError::Windows("window no longer exists".to_owned()));
            }
            PostMessageW(hwnd, WM_CLOSE, WPARAM(0), LPARAM(0))
                .map_err(|e| WorksetError::Windows(format!("failed to close window: {e}")))?;
        }
        Ok(())
    }
}

#[cfg(not(windows))]
mod platform {
    use crate::error::WorksetError;
    use crate::models::CapturedWindow;

    const OFF: &str = "window management is only available on Windows";

    pub fn enum_windows() -> Vec<CapturedWindow> {
        Vec::new()
    }

    pub fn snapshot_before_launch() -> Vec<CapturedWindow> {
        Vec::new()
    }

    pub fn find_window(_hwnd: i64) -> Option<CapturedWindow> {
        None
    }

    pub fn move_window(
        _hwnd: i64,
        _x: i32,
        _y: i32,
        _w: i32,
        _h: i32,
        _state: &str,
    ) -> Result<(), WorksetError> {
        Err(WorksetError::Windows(OFF.to_owned()))
    }

    pub fn is_window_valid(_hwnd: i64) -> bool {
        false
    }

    pub fn focus_window(_hwnd: i64) -> Result<(), WorksetError> {
        Err(WorksetError::Windows(OFF.to_owned()))
    }

    pub fn close_window(_hwnd: i64) -> Result<(), WorksetError> {
        Err(WorksetError::Windows(OFF.to_owned()))
    }
}

pub use platform::{
    close_window, enum_windows, find_window, focus_window, is_window_valid, move_window,
    snapshot_before_launch,
};
