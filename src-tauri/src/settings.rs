//! App settings persisted in the `user_settings` table.

use crate::database::Db;
use crate::error::WorksetError;
use crate::models::AppSettings;

pub fn defaults() -> AppSettings {
    AppSettings {
        theme: "system".to_owned(),
        default_timeout_ms: 15000,
        default_match_policy: "exe_and_title".to_owned(),
        confirm_before_launch: false,
        notifications_enabled: true,
    }
}

fn parse_bool(v: &str) -> Option<bool> {
    match v.trim().to_ascii_lowercase().as_str() {
        "1" | "true" | "yes" | "on" => Some(true),
        "0" | "false" | "no" | "off" => Some(false),
        _ => None,
    }
}

/// Load typed settings, falling back to defaults for missing keys.
pub fn load(db: &Db) -> AppSettings {
    let d = defaults();
    let get = |k: &str| db.get_setting(k).unwrap_or(None);
    AppSettings {
        theme: get("theme").unwrap_or(d.theme),
        default_timeout_ms: get("default_timeout_ms")
            .and_then(|v| v.parse().ok())
            .unwrap_or(d.default_timeout_ms),
        default_match_policy: get("default_match_policy").unwrap_or(d.default_match_policy),
        confirm_before_launch: get("confirm_before_launch")
            .and_then(|v| parse_bool(&v))
            .unwrap_or(d.confirm_before_launch),
        notifications_enabled: get("notifications_enabled")
            .and_then(|v| parse_bool(&v))
            .unwrap_or(d.notifications_enabled),
    }
}

/// Persist typed settings.
pub fn save(db: &Db, s: &AppSettings) -> Result<(), WorksetError> {
    db.set_setting("theme", &s.theme)?;
    db.set_setting("default_timeout_ms", &s.default_timeout_ms.to_string())?;
    db.set_setting("default_match_policy", &s.default_match_policy)?;
    db.set_setting(
        "confirm_before_launch",
        if s.confirm_before_launch {
            "true"
        } else {
            "false"
        },
    )?;
    db.set_setting(
        "notifications_enabled",
        if s.notifications_enabled {
            "true"
        } else {
            "false"
        },
    )?;
    Ok(())
}
