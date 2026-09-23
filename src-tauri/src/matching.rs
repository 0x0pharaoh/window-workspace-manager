//! Pure, testable window matching: score candidates against [`MatchRules`]
//! and pick the best one. Regex failures degrade gracefully to substring
//! matching instead of erroring.

use crate::models::{CapturedWindow, MatchRules};

/// Executable/path hints for OS-owned windows; matching them is penalized.
const SYSTEM_EXE_HINTS: &[&str] = &[
    "system32",
    "syswow64",
    "sihost",
    "shellexperiencehost",
    "textinputhost",
    "applicationframehost",
    "startmenuexperiencehost",
    "searchhost",
    "dwm.exe",
    "winlogon",
    "csrss",
    "smss",
    "services.exe",
    "lsass",
    "fontdrvhost",
];

fn non_empty(opt: &Option<String>) -> Option<String> {
    opt.as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(str::to_owned)
}

/// True when `title` satisfies the title half of `rules`.
///
/// A non-empty `title_regex` is tried first (an invalid pattern falls back
/// to substring matching); otherwise `title_contains` applies; with neither
/// set every title matches.
///
/// Test helper (production scoring inlines the same logic with weights).
#[cfg(test)]
pub fn is_title_match(title: &str, rules: &MatchRules) -> bool {
    if let Some(rx) = non_empty(&rules.title_regex) {
        if let Ok(re) = regex::Regex::new(&rx) {
            return re.is_match(title);
        }
    }
    match non_empty(&rules.title_contains) {
        None => true,
        Some(needle) => title.to_lowercase().contains(&needle.to_lowercase()),
    }
}

/// Score a candidate window. Weights: exe +50, class +30, title +20,
/// regex +25, fresh-after-launch +15, system-window penalty -50.
pub fn score_window(window: &CapturedWindow, rules: &MatchRules, launch_ts: Option<i64>) -> i32 {
    let mut score = 0i32;
    let exe = window.exe_path.to_lowercase();

    if let Some(needle) = non_empty(&rules.exe_contains) {
        if exe.contains(&needle.to_lowercase()) {
            score += 50;
        }
    }
    if let Some(class) = non_empty(&rules.window_class) {
        if window.class.eq_ignore_ascii_case(&class) {
            score += 30;
        }
    }
    if let Some(needle) = non_empty(&rules.title_contains) {
        if window.title.to_lowercase().contains(&needle.to_lowercase()) {
            score += 20;
        }
    }
    if let Some(rx) = non_empty(&rules.title_regex) {
        if let Ok(re) = regex::Regex::new(&rx) {
            if re.is_match(&window.title) {
                score += 25;
            }
        }
    }
    // Caller only scores freshly appeared windows when this is Some.
    if launch_ts.is_some() {
        score += 15;
    }
    if SYSTEM_EXE_HINTS.iter().any(|h| exe.contains(h)) {
        score -= 50;
    }
    score
}

/// Best candidate with a positive score, or `None`.
pub fn find_best(candidates: &[CapturedWindow], rules: &MatchRules) -> Option<CapturedWindow> {
    find_best_with(candidates, rules, None)
}

pub fn find_best_with(
    candidates: &[CapturedWindow],
    rules: &MatchRules,
    launch_ts: Option<i64>,
) -> Option<CapturedWindow> {
    let mut best: Option<(&CapturedWindow, i32)> = None;
    for w in candidates {
        let s = score_window(w, rules, launch_ts);
        if s > 0 && best.map(|(_, b)| s > b).unwrap_or(true) {
            best = Some((w, s));
        }
    }
    best.map(|(w, _)| w.clone())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn win(exe: &str, title: &str, class: &str) -> CapturedWindow {
        CapturedWindow {
            hwnd: 1,
            pid: 10,
            exe_path: exe.to_owned(),
            title: title.to_owned(),
            class: class.to_owned(),
            x: 0,
            y: 0,
            w: 100,
            h: 100,
            monitor_id: String::new(),
            state: "normal".to_owned(),
        }
    }

    fn rules(exe: &str, title: &str) -> MatchRules {
        MatchRules {
            exe_contains: if exe.is_empty() {
                None
            } else {
                Some(exe.to_owned())
            },
            title_contains: if title.is_empty() {
                None
            } else {
                Some(title.to_owned())
            },
            title_regex: None,
            window_class: None,
            use_regex: false,
        }
    }

    #[test]
    fn scoring_weights() {
        let w = win("C:\\App\\demo.exe", "Demo - Document", "DemoClass");
        let r = rules("demo.exe", "document");
        assert_eq!(score_window(&w, &r, None), 70);
        assert_eq!(score_window(&w, &r, Some(1)), 85);
    }

    #[test]
    fn class_exact_case_insensitive() {
        let w = win("C:\\a.exe", "t", "Notepad");
        let mut r = rules("", "");
        r.window_class = Some("notepad".to_owned());
        assert_eq!(score_window(&w, &r, None), 30);
    }

    #[test]
    fn regex_match_and_bad_pattern() {
        let w = win("C:\\a.exe", "Project - Editor", "");
        let mut r = rules("", "");
        r.title_regex = Some("^Project.*".to_owned());
        assert_eq!(score_window(&w, &r, None), 25);
        assert!(is_title_match("Project - Editor", &r));

        r.title_regex = Some("([bad".to_owned());
        // invalid regex degrades instead of panicking
        assert_eq!(score_window(&w, &r, None), 0);
        assert!(is_title_match("anything", &r)); // falls back to empty contains
    }

    #[test]
    fn system_penalty_and_best() {
        let sys = win("C:\\Windows\\System32\\sihost.exe", "sihost", "");
        let app = win("C:\\App\\demo.exe", "Demo", "");
        let r = rules("exe", "");
        assert!(score_window(&sys, &r, None) < score_window(&app, &r, None));
        let best = find_best(&[sys, app], &r).unwrap();
        assert_eq!(best.title, "Demo");
        assert!(find_best(&[], &r).is_none());
    }
}
