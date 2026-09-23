//! Process helpers: exe validation, detached launching, waiting for a
//! new window after spawn, and explicit termination.

use std::path::Path;

use crate::error::WorksetError;
use crate::models::{CapturedWindow, MatchRules};

const ALLOWED_EXTENSIONS: &[&str] = &[
    "exe",
    "com",
    "bat",
    "cmd",
    "lnk",
    "url",
    "msi",
    "ps1",
    "vbs",
    "wsf",
    "appref-ms",
];

/// Heuristic: is this target a URL/protocol rather than a file path?
/// Single-letter `C:` drive prefixes are files, not schemes.
pub fn is_url_target(s: &str) -> bool {
    let t = s.trim();
    if t.is_empty() {
        return false;
    }
    if t.contains("://") {
        return true;
    }
    if let Some((scheme, _)) = t.split_once(':') {
        if scheme.len() > 1
            && scheme.len() <= 32
            && scheme
                .chars()
                .all(|c| c.is_ascii_alphanumeric() || c == '+' || c == '-' || c == '.')
        {
            return true;
        }
    }
    false
}

/// Validate an executable path: non-empty, no shell-injection tokens, an
/// allowed file type that exists (URLs skip the existence check).
/// Launching never goes through a shell, so `Command` receives exe+args
/// directly with no string concatenation.
pub fn validate_exe(path: &str) -> Result<(), WorksetError> {
    let t = path.trim();
    if t.is_empty() {
        return Err(WorksetError::Validation(
            "executable path is empty".to_owned(),
        ));
    }
    if t.contains(['\0', '\n', '\r', '`', ';'])
        || t.contains("$(")
        || t.contains("${")
        || t.contains("&&")
        || t.contains("||")
    {
        return Err(WorksetError::Validation(format!(
            "executable path contains disallowed characters: {t}"
        )));
    }
    if is_url_target(t) {
        return Ok(());
    }
    let p = Path::new(t);
    if !p.exists() {
        return Err(WorksetError::NotFound(format!("executable not found: {t}")));
    }
    if p.is_dir() {
        return Err(WorksetError::Validation(format!(
            "launch target is a directory: {t}"
        )));
    }
    if let Some(ext) = p.extension().and_then(|e| e.to_str()) {
        let e = ext.to_ascii_lowercase();
        if !ALLOWED_EXTENSIONS.contains(&e.as_str()) {
            return Err(WorksetError::Validation(format!(
                "file type .{e} is not launchable"
            )));
        }
    }
    Ok(())
}

/// Launch an app detached and return its pid.
/// `.exe`/`.com` files spawn directly; anything else (`.lnk`, `.bat`,
/// URLs, ...) goes through `cmd /C start` so shell associations resolve.
/// An exe-less entry with `url` opens the URL.
pub fn launch_app(
    exe_path: &str,
    args: &[String],
    cwd: &str,
    url: &str,
) -> Result<u32, WorksetError> {
    #[cfg(windows)]
    {
        launch_app_windows(exe_path, args, cwd, url)
    }
    #[cfg(not(windows))]
    {
        let _ = (exe_path, args, cwd, url);
        Err(WorksetError::Windows(
            "launching apps is only supported on Windows".to_owned(),
        ))
    }
}

#[cfg(windows)]
fn launch_app_windows(
    exe_path: &str,
    args: &[String],
    cwd: &str,
    url: &str,
) -> Result<u32, WorksetError> {
    use std::process::{Command, Stdio};

    let exe = exe_path.trim();
    if exe.is_empty() {
        let u = url.trim();
        if u.is_empty() {
            return Err(WorksetError::Validation(
                "nothing to launch: exe_path and url are both empty".to_owned(),
            ));
        }
        let child = Command::new("cmd")
            .args(["/C", "start", "", u])
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .map_err(|e| WorksetError::Windows(format!("failed to open url: {e}")))?;
        return Ok(child.id());
    }
    validate_exe(exe)?;

    let ext = Path::new(exe)
        .extension()
        .and_then(|e| e.to_str())
        .map(|e| e.to_ascii_lowercase())
        .unwrap_or_default();
    let direct = matches!(ext.as_str(), "exe" | "com") && !is_url_target(exe);

    let mut cmd = if direct {
        let mut c = Command::new(exe);
        c.args(args);
        if !cwd.trim().is_empty() {
            if !Path::new(cwd).is_dir() {
                return Err(WorksetError::Validation(format!(
                    "working directory does not exist: {cwd}"
                )));
            }
            c.current_dir(cwd);
        }
        c
    } else {
        let mut c = Command::new("cmd");
        c.arg("/C").arg("start").arg("");
        if !cwd.trim().is_empty() && Path::new(cwd).is_dir() {
            c.arg("/D").arg(cwd);
        }
        c.arg(exe);
        c.args(args);
        c
    };
    let child = cmd
        .stdin(std::process::Stdio::null())
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .spawn()
        .map_err(|e| WorksetError::Windows(format!("failed to launch '{exe}': {e}")))?;
    Ok(child.id())
}

/// Poll for a newly appeared window matching `rules` (every 250 ms).
/// Only windows absent from `snapshot` are considered.
pub async fn wait_for_new_window(
    snapshot: &[CapturedWindow],
    rules: &MatchRules,
    timeout_ms: u64,
) -> Result<CapturedWindow, WorksetError> {
    use std::collections::HashSet;
    use std::time::{Duration, Instant};

    let known: HashSet<i64> = snapshot.iter().map(|w| w.hwnd).collect();
    let deadline = Instant::now() + Duration::from_millis(timeout_ms.max(250));
    loop {
        let fresh: Vec<CapturedWindow> = crate::windows::enum_windows()
            .into_iter()
            .filter(|w| !known.contains(&w.hwnd))
            .collect();
        if let Some(best) = crate::matching::find_best(&fresh, rules) {
            return Ok(best);
        }
        if Instant::now() >= deadline {
            return Err(WorksetError::Timeout(format!(
                "no matching window appeared within {timeout_ms} ms"
            )));
        }
        tokio::time::sleep(Duration::from_millis(250)).await;
    }
}

/// Terminate a process by pid. Only called on explicit user request
/// (force-close), never as part of a normal launch.
pub fn kill_pid(pid: u32) -> Result<(), WorksetError> {
    #[cfg(windows)]
    {
        use windows::Win32::Foundation::{CloseHandle, FALSE};
        use windows::Win32::System::Threading::{OpenProcess, TerminateProcess, PROCESS_TERMINATE};
        unsafe {
            let handle = OpenProcess(PROCESS_TERMINATE, FALSE, pid)
                .map_err(|e| WorksetError::Windows(format!("cannot open pid {pid}: {e}")))?;
            let res = TerminateProcess(handle, 1)
                .map(|_| ())
                .map_err(|e| WorksetError::Windows(format!("cannot terminate pid {pid}: {e}")));
            let _ = CloseHandle(handle);
            res
        }
    }
    #[cfg(not(windows))]
    {
        let _ = pid;
        Err(WorksetError::Windows(
            "terminating processes is only supported on Windows".to_owned(),
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn url_detection() {
        assert!(is_url_target("https://example.com/x"));
        assert!(is_url_target("mailto:a@b.c"));
        assert!(is_url_target("vscode://file/x"));
        assert!(!is_url_target("C:\\Program Files\\app.exe"));
        assert!(!is_url_target("C:/x/y.exe"));
        assert!(!is_url_target(""));
        assert!(!is_url_target("my app: test"));
    }

    #[test]
    fn injection_blocked() {
        assert!(validate_exe("a.exe; rm -rf /").is_err());
        assert!(validate_exe("run $(whoami)").is_err());
        assert!(validate_exe("a && b").is_err());
        assert!(validate_exe("").is_err());
        // URLs pass the char filter (existence not required)
        assert!(validate_exe("https://example.com/").is_ok());
    }
}
