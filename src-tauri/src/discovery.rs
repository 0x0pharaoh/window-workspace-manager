//! App discovery: Start Menu shortcuts, installed programs and running
//! processes. Results are deduplicated by lowercase exe path; the frontend
//! does the search filtering.

use std::collections::HashMap;
use std::path::{Path, PathBuf};

use crate::models::DiscoveredApp;

/// Best-effort `.lnk` target resolution. Real resolution needs COM
/// (`IShellLink`), so the `.lnk` path itself is returned as the launch
/// target — the frontend opens it via the opener plugin.
pub fn parse_lnk_target(lnk_path: &Path) -> String {
    lnk_path.to_string_lossy().to_string()
}

fn walk_ext(dir: &Path, exts: &[&str], max_depth: usize, cap: usize, out: &mut Vec<PathBuf>) {
    if out.len() >= cap || max_depth == 0 {
        return;
    }
    let entries = match std::fs::read_dir(dir) {
        Ok(e) => e,
        Err(_) => return,
    };
    for entry in entries.flatten() {
        if out.len() >= cap {
            break;
        }
        let path = entry.path();
        if path.is_dir() {
            walk_ext(&path, exts, max_depth - 1, cap, out);
        } else if path
            .extension()
            .and_then(|e| e.to_str())
            .is_some_and(|e| exts.iter().any(|w| w.eq_ignore_ascii_case(e)))
        {
            out.push(path);
        }
    }
}

fn display_name(path: &Path) -> String {
    path.file_stem()
        .map(|s| s.to_string_lossy().to_string())
        .filter(|s| !s.trim().is_empty())
        .unwrap_or_else(|| "Unknown".to_owned())
}

fn start_menu_dirs() -> Vec<PathBuf> {
    let mut dirs = Vec::new();
    if let Some(data) = dirs::data_dir() {
        dirs.push(data.join("Microsoft/Windows/Start Menu/Programs"));
    }
    if let Ok(all) = std::env::var("PROGRAMDATA") {
        dirs.push(PathBuf::from(all).join("Microsoft/Windows/Start Menu/Programs"));
    } else {
        dirs.push(PathBuf::from(
            "C:/ProgramData/Microsoft/Windows/Start Menu/Programs",
        ));
    }
    dirs.into_iter().filter(|d| d.is_dir()).collect()
}

fn program_files_dirs() -> Vec<PathBuf> {
    let mut dirs = Vec::new();
    for key in ["PROGRAMFILES", "PROGRAMFILES(X86)", "PROGRAMW6432"] {
        if let Ok(v) = std::env::var(key) {
            dirs.push(PathBuf::from(v));
        }
    }
    dirs.push(PathBuf::from("C:/Program Files"));
    dirs.push(PathBuf::from("C:/Program Files (x86)"));
    dirs.into_iter().filter(|d| d.is_dir()).collect()
}

fn scan_start_menu() -> Vec<DiscoveredApp> {
    let mut found = Vec::new();
    for dir in start_menu_dirs() {
        let mut lnks = Vec::new();
        walk_ext(&dir, &["lnk"], 4, 2000, &mut lnks);
        for lnk in lnks {
            found.push(DiscoveredApp {
                name: display_name(&lnk),
                exe_path: parse_lnk_target(&lnk),
                source: "start_menu".to_owned(),
                icon: String::new(),
            });
        }
    }
    found
}

fn scan_program_files() -> Vec<DiscoveredApp> {
    let mut found = Vec::new();
    for dir in program_files_dirs() {
        let mut exes = Vec::new();
        walk_ext(&dir, &["exe"], 2, 300, &mut exes);
        for exe in exes {
            if found.len() >= 300 {
                break;
            }
            found.push(DiscoveredApp {
                name: display_name(&exe),
                exe_path: exe.to_string_lossy().to_string(),
                source: "installed".to_owned(),
                icon: String::new(),
            });
        }
        if found.len() >= 300 {
            break;
        }
    }
    found
}

fn os_to_string<S: AsRef<std::ffi::OsStr>>(s: S) -> String {
    s.as_ref().to_string_lossy().to_string()
}

fn scan_running_processes() -> Vec<DiscoveredApp> {
    let sys = sysinfo::System::new_all();
    let mut found = Vec::new();
    for proc_ in sys.processes().values() {
        let exe = proc_.exe().map(os_to_string).unwrap_or_default();
        if exe.trim().is_empty() {
            continue;
        }
        let name = os_to_string(proc_.name());
        let name = if name.trim().is_empty() {
            display_name(Path::new(&exe))
        } else {
            name
        };
        found.push(DiscoveredApp {
            name,
            exe_path: exe,
            source: "running".to_owned(),
            icon: String::new(),
        });
        if found.len() >= 500 {
            break;
        }
    }
    found
}

/// Discover launchable apps from all sources, deduplicated and sorted.
pub fn discover_apps() -> Vec<DiscoveredApp> {
    let mut by_exe: HashMap<String, DiscoveredApp> = HashMap::new();
    for app in scan_start_menu()
        .into_iter()
        .chain(scan_program_files())
        .chain(scan_running_processes())
    {
        let key = app.exe_path.to_lowercase();
        if key.trim().is_empty() {
            continue;
        }
        by_exe.entry(key).or_insert(app);
    }
    let mut out: Vec<DiscoveredApp> = by_exe.into_values().collect();
    out.sort_by(|a, b| {
        a.name
            .to_lowercase()
            .cmp(&b.name.to_lowercase())
            .then(a.exe_path.to_lowercase().cmp(&b.exe_path.to_lowercase()))
    });
    out
}
