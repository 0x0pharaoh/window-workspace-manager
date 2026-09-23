//! Shared data model: serde structs plus validation helpers.
//!
//! Rust field names follow the backend spec. Where the frontend uses
//! different JSON keys (`favorite` vs `is_favorite`, `width` vs `w`, ...),
//! `serde(rename = ..., alias = ...)` attributes accept both spellings on
//! input and emit the frontend spelling on output.

use serde::{Deserialize, Serialize};

use crate::error::WorksetError;

fn default_color() -> String {
    "#0078d4".to_owned()
}

fn default_half() -> f64 {
    0.5
}

fn default_scale() -> f64 {
    1.0
}

fn default_window_state() -> String {
    "normal".to_owned()
}

fn default_launch_policy() -> String {
    "if_not_running".to_owned()
}

fn default_pending() -> String {
    "pending".to_owned()
}

fn default_running() -> String {
    "running".to_owned()
}

fn default_info() -> String {
    "info".to_owned()
}

fn default_theme() -> String {
    "system".to_owned()
}

fn default_timeout_ms() -> i64 {
    15000
}

fn default_match_policy() -> String {
    "exe_and_title".to_owned()
}

fn default_true() -> bool {
    true
}

fn default_schema_version() -> u32 {
    1
}

fn default_object() -> serde_json::Value {
    serde_json::json!({})
}

/// Split a command line into words, honouring double quotes.
pub fn split_args(s: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut cur = String::new();
    let mut in_quotes = false;
    let mut in_word = false;
    for c in s.chars() {
        match c {
            '"' => {
                in_quotes = !in_quotes;
                in_word = true;
            }
            c if c.is_whitespace() && !in_quotes => {
                if in_word {
                    out.push(std::mem::take(&mut cur));
                    in_word = false;
                }
            }
            _ => {
                cur.push(c);
                in_word = true;
            }
        }
    }
    if in_word {
        out.push(cur);
    }
    out
}

/// Join args back into a command line, quoting segments with whitespace.
pub fn join_args(args: &[String]) -> String {
    args.iter()
        .map(|a| {
            if a.is_empty() || a.chars().any(|c| c.is_whitespace() || c == '"') {
                format!("\"{}\"", a.replace('"', "\\\""))
            } else {
                a.clone()
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

/// `args` is `Vec<String>` in Rust but travels as a single command-line
/// string over JSON (frontend-friendly). Accepts a string or an array.
mod args_string {
    use serde::{Deserialize, Deserializer, Serializer};

    pub fn serialize<S>(args: &Vec<String>, ser: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        ser.serialize_str(&super::join_args(args))
    }

    pub fn deserialize<'de, D>(de: D) -> Result<Vec<String>, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        #[serde(untagged)]
        enum StringOrVec {
            S(String),
            V(Vec<String>),
        }
        match StringOrVec::deserialize(de)? {
            StringOrVec::S(s) => Ok(super::split_args(&s)),
            StringOrVec::V(v) => Ok(v),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Workspace {
    #[serde(default)]
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub icon: String,
    #[serde(default = "default_color")]
    pub color: String,
    #[serde(rename = "favorite", alias = "is_favorite", default)]
    pub is_favorite: bool,
    #[serde(default)]
    pub hotkey: String,
    #[serde(default = "default_object")]
    pub launch_settings: serde_json::Value,
    #[serde(default)]
    pub created_at: String,
    #[serde(default)]
    pub updated_at: String,
    #[serde(default)]
    pub apps: Vec<WorkspaceApp>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkspaceApp {
    #[serde(default)]
    pub id: String,
    #[serde(default)]
    pub workspace_id: String,
    #[serde(default)]
    pub sort_order: i64,
    pub name: String,
    #[serde(default)]
    pub exe_path: String,
    #[serde(default, with = "args_string")]
    pub args: Vec<String>,
    #[serde(default)]
    pub cwd: String,
    #[serde(default)]
    pub url: String,
    #[serde(default)]
    pub app_icon: String,
    #[serde(default)]
    pub match_rules: MatchRules,
    #[serde(rename = "delay_ms", alias = "launch_delay_ms", default)]
    pub launch_delay_ms: i64,
    #[serde(rename = "monitor_id", alias = "target_monitor", default)]
    pub target_monitor: String,
    #[serde(default)]
    pub x: f64,
    #[serde(default)]
    pub y: f64,
    #[serde(default = "default_half")]
    pub w: f64,
    #[serde(default = "default_half")]
    pub h: f64,
    #[serde(
        rename = "state",
        alias = "window_state",
        default = "default_window_state"
    )]
    pub window_state: String,
    #[serde(
        rename = "policy",
        alias = "launch_policy",
        default = "default_launch_policy"
    )]
    pub launch_policy: String,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct MatchRules {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub exe_contains: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub title_contains: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub title_regex: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub window_class: Option<String>,
    #[serde(default)]
    pub use_regex: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonitorInfo {
    pub id: String,
    pub x: i32,
    pub y: i32,
    #[serde(rename = "width", alias = "w")]
    pub w: i32,
    #[serde(rename = "height", alias = "h")]
    pub h: i32,
    pub work_x: i32,
    pub work_y: i32,
    #[serde(rename = "work_width", alias = "work_w")]
    pub work_w: i32,
    #[serde(rename = "work_height", alias = "work_h")]
    pub work_h: i32,
    #[serde(rename = "scale_factor", alias = "scale", default = "default_scale")]
    pub scale: f64,
    #[serde(default)]
    pub is_primary: bool,
    #[serde(default)]
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CapturedWindow {
    #[serde(default)]
    pub hwnd: i64,
    #[serde(default)]
    pub pid: u32,
    #[serde(default)]
    pub exe_path: String,
    #[serde(default)]
    pub title: String,
    #[serde(default, rename = "window_class", alias = "class")]
    pub class: String,
    #[serde(default)]
    pub x: i32,
    #[serde(default)]
    pub y: i32,
    #[serde(default, rename = "width", alias = "w")]
    pub w: i32,
    #[serde(default, rename = "height", alias = "h")]
    pub h: i32,
    #[serde(default)]
    pub monitor_id: String,
    #[serde(default = "default_window_state")]
    pub state: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiscoveredApp {
    pub name: String,
    pub exe_path: String,
    #[serde(default)]
    pub source: String,
    #[serde(default)]
    pub icon: String,
}

/// Progress event emitted on the `launch-progress` channel. Carries the
/// spec fields plus the frontend's display fields.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LaunchProgressEvent {
    pub session_id: String,
    #[serde(default)]
    pub workspace_id: String,
    pub app_id: String,
    #[serde(default)]
    pub app_name: String,
    pub status: String,
    #[serde(default)]
    pub message: String,
    #[serde(default)]
    pub progress: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionWindowRow {
    pub id: String,
    pub session_id: String,
    #[serde(default)]
    pub app_id: String,
    #[serde(default)]
    pub pid: Option<i64>,
    #[serde(default)]
    pub hwnd: Option<i64>,
    #[serde(default = "default_pending")]
    pub status: String,
    #[serde(default)]
    pub message: String,
    #[serde(default = "default_object")]
    pub last_rect: serde_json::Value,
    #[serde(default)]
    pub launched_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionInfo {
    pub id: String,
    pub workspace_id: String,
    #[serde(default = "default_running")]
    pub status: String,
    #[serde(default)]
    pub started_at: String,
    #[serde(default)]
    pub finished_at: Option<String>,
    #[serde(default)]
    pub windows: Vec<SessionWindowRow>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppSettings {
    #[serde(default = "default_theme")]
    pub theme: String,
    #[serde(default = "default_timeout_ms")]
    pub default_timeout_ms: i64,
    #[serde(default = "default_match_policy")]
    pub default_match_policy: String,
    #[serde(default)]
    pub confirm_before_launch: bool,
    #[serde(default = "default_true")]
    pub notifications_enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportFormat {
    #[serde(alias = "version", default = "default_schema_version")]
    pub schema_version: u32,
    pub workspace: Workspace,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LaunchLog {
    pub id: i64,
    #[serde(default)]
    pub session_id: String,
    #[serde(default)]
    pub app_id: String,
    #[serde(default = "default_info")]
    pub level: String,
    #[serde(default)]
    pub message: String,
    #[serde(default)]
    pub created_at: String,
}

/// Tokens that suggest shell interpretation. Imports containing them need
/// explicit user confirmation (see `import_export`).
pub fn contains_suspicious_tokens(exe_path: &str, args: &[String]) -> bool {
    const TOKENS: &[&str] = &[";", "`", "$(", "${", "&&", "||", "&", "|"];
    let haystacks = std::iter::once(exe_path.to_owned()).chain(args.iter().cloned());
    for h in haystacks {
        for t in TOKENS {
            if h.contains(t) {
                return true;
            }
        }
    }
    false
}

fn check_rect(name: &str, x: f64, y: f64, w: f64, h: f64) -> Result<(), WorksetError> {
    for (k, v) in [("x", x), ("y", y), ("w", w), ("h", h)] {
        if !v.is_finite() || v < 0.0 || v > 1.0 {
            return Err(WorksetError::Validation(format!(
                "app \"{name}\" has invalid {k}: must be within 0..1"
            )));
        }
    }
    if w < 0.05 || h < 0.05 {
        return Err(WorksetError::Validation(format!(
            "app \"{name}\" is too small: w/h must be >= 0.05"
        )));
    }
    Ok(())
}

/// Validate a workspace and all of its apps.
pub fn validate_workspace(ws: &Workspace) -> Result<(), WorksetError> {
    if ws.name.trim().is_empty() {
        return Err(WorksetError::Validation(
            "workspace name must not be empty".to_owned(),
        ));
    }
    for app in &ws.apps {
        validate_app(app)?;
    }
    Ok(())
}

/// Validate a single app entry.
pub fn validate_app(app: &WorkspaceApp) -> Result<(), WorksetError> {
    if app.name.trim().is_empty() {
        return Err(WorksetError::Validation(
            "app name must not be empty".to_owned(),
        ));
    }
    check_rect(&app.name, app.x, app.y, app.w, app.h)?;
    if app.exe_path.trim().is_empty() && app.url.trim().is_empty() {
        return Err(WorksetError::Validation(format!(
            "app \"{}\" needs either exe_path or url",
            app.name
        )));
    }
    if let Some(rx) = app.match_rules.title_regex.as_deref() {
        let rx = rx.trim();
        if !rx.is_empty() {
            regex::Regex::new(rx)
                .map(|_| ())
                .map_err(|e| WorksetError::Regex(format!("invalid title_regex: {e}")))?;
        }
    }
    if app.launch_delay_ms < 0 {
        return Err(WorksetError::Validation(format!(
            "app \"{}\" has a negative launch delay",
            app.name
        )));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_app() -> WorkspaceApp {
        WorkspaceApp {
            id: "a1".to_owned(),
            workspace_id: "w1".to_owned(),
            sort_order: 0,
            name: "Demo".to_owned(),
            exe_path: "C:\\App\\demo.exe".to_owned(),
            args: vec!["--new-window".to_owned()],
            cwd: String::new(),
            url: String::new(),
            app_icon: String::new(),
            match_rules: MatchRules::default(),
            launch_delay_ms: 0,
            target_monitor: String::new(),
            x: 0.0,
            y: 0.0,
            w: 0.5,
            h: 0.5,
            window_state: "normal".to_owned(),
            launch_policy: "if_not_running".to_owned(),
        }
    }

    #[test]
    fn valid_app_passes() {
        assert!(validate_app(&sample_app()).is_ok());
    }

    #[test]
    fn empty_name_fails() {
        let mut a = sample_app();
        a.name = "   ".to_owned();
        assert!(matches!(validate_app(&a), Err(WorksetError::Validation(_))));
    }

    #[test]
    fn rect_out_of_range_fails() {
        let mut a = sample_app();
        a.w = 1.5;
        assert!(validate_app(&a).is_err());
        let mut b = sample_app();
        b.w = 0.01;
        assert!(validate_app(&b).is_err());
    }

    #[test]
    fn exe_or_url_required() {
        let mut a = sample_app();
        a.exe_path.clear();
        assert!(validate_app(&a).is_err());
        a.url = "https://example.com".to_owned();
        assert!(validate_app(&a).is_ok());
    }

    #[test]
    fn bad_regex_fails() {
        let mut a = sample_app();
        a.match_rules.title_regex = Some("([unclosed".to_owned());
        assert!(matches!(validate_app(&a), Err(WorksetError::Regex(_))));
    }

    #[test]
    fn workspace_name_required() {
        let ws = Workspace {
            id: "w".to_owned(),
            name: String::new(),
            description: String::new(),
            icon: String::new(),
            color: default_color(),
            is_favorite: false,
            hotkey: String::new(),
            launch_settings: default_object(),
            created_at: String::new(),
            updated_at: String::new(),
            apps: vec![],
        };
        assert!(validate_workspace(&ws).is_err());
    }

    #[test]
    fn args_string_roundtrip() {
        let args = vec!["--flag".to_owned(), "C:\\My Dir\\file.txt".to_owned()];
        let joined = join_args(&args);
        assert_eq!(split_args(&joined), args);
        // plain string from frontend
        let v: serde_json::Value = serde_json::json!({ "args": "--a --b" });
        let parsed: Vec<String> = serde_json::from_value(v["args"].clone())
            .map(|s: String| split_args(&s))
            .unwrap();
        assert_eq!(parsed, vec!["--a", "--b"]);
    }

    #[test]
    fn frontend_keys_accepted() {
        let v = serde_json::json!({
            "id": "a1", "workspace_id": "w1", "name": "N",
            "exe_path": "C:\\x.exe", "args": "--a",
            "delay_ms": 500, "monitor_id": "m1",
            "x": 0.0, "y": 0.0, "w": 0.5, "h": 0.5,
            "state": "maximized", "policy": "always_new",
            "match_rules": { "title_contains": "hi" }
        });
        let app: WorkspaceApp = serde_json::from_value(v).unwrap();
        assert_eq!(app.launch_delay_ms, 500);
        assert_eq!(app.target_monitor, "m1");
        assert_eq!(app.window_state, "maximized");
        assert_eq!(app.launch_policy, "always_new");
        // serialized back with frontend names
        let out = serde_json::to_value(&app).unwrap();
        assert_eq!(out["delay_ms"], 500);
        assert_eq!(out["favorite"].is_null(), true); // unrelated struct key absent
    }

    #[test]
    fn suspicious_tokens_detected() {
        assert!(contains_suspicious_tokens("app.exe", &["a;rm".to_owned()]));
        assert!(contains_suspicious_tokens(
            "app.exe",
            &["$(whoami)".to_owned()]
        ));
        assert!(!contains_suspicious_tokens(
            "C:\\App\\app.exe",
            &["--url".to_owned(), "https://x.test/?a=1".to_owned()]
        ));
    }
}
