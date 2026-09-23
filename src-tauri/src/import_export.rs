//! Workspace import/export.
//!
//! Exports wrap the workspace in a versioned envelope; imports require
//! `schema_version` (or legacy `version`) == 1 and an explicit confirmation
//! flag when shell-like tokens are found in exe paths or args.

use crate::database::{now_iso, Db};
use crate::error::WorksetError;
use crate::models::{contains_suspicious_tokens, validate_workspace, ExportFormat, Workspace};

/// Serialize a workspace to a versioned JSON envelope.
pub fn export_workspace(db: &Db, id: &str) -> Result<String, WorksetError> {
    let ws = db.get_workspace(id)?;
    let envelope = serde_json::json!({
        "schema_version": 1,
        "version": 1,
        "exported_at": now_iso(),
        "workspace": ws,
    });
    serde_json::to_string_pretty(&envelope).map_err(WorksetError::from)
}

/// Import a workspace envelope (or a bare workspace object).
/// Fresh ids are assigned so imports never collide with existing rows.
pub fn import_workspace(
    db: &Db,
    json_str: &str,
    user_confirmed: bool,
) -> Result<Workspace, WorksetError> {
    let v: serde_json::Value = serde_json::from_str(json_str)?;
    let version = v
        .get("schema_version")
        .or_else(|| v.get("version"))
        .and_then(|x| x.as_u64())
        .unwrap_or(0);
    if version != 1 {
        return Err(WorksetError::Validation(format!(
            "unsupported schema version {version} (expected 1)"
        )));
    }
    let ws_value = match v.get("workspace") {
        Some(w) => w.clone(),
        None => v.clone(),
    };
    // Accept both the versioned envelope and a bare workspace.
    let mut ws: Workspace = if ws_value.get("workspace").is_some() {
        serde_json::from_value::<ExportFormat>(ws_value)
            .map(|f| f.workspace)
            .map_err(|e| WorksetError::Validation(format!("invalid workspace: {e}")))?
    } else {
        serde_json::from_value(ws_value)
            .map_err(|e| WorksetError::Validation(format!("invalid workspace: {e}")))?
    };
    validate_workspace(&ws)?;

    let risky = ws
        .apps
        .iter()
        .any(|a| contains_suspicious_tokens(&a.exe_path, &a.args));
    if risky && !user_confirmed {
        return Err(WorksetError::Validation(
            "import contains shell-like tokens in exe paths or args; \
             re-import with explicit confirmation to proceed"
                .to_owned(),
        ));
    }

    ws.id = uuid::Uuid::new_v4().to_string();
    ws.hotkey.clear();
    ws.is_favorite = false;
    ws.created_at = now_iso();
    ws.updated_at = ws.created_at.clone();
    for app in ws.apps.iter_mut() {
        app.id = uuid::Uuid::new_v4().to_string();
        app.workspace_id = ws.id.clone();
    }
    db.create_workspace(ws)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::database::Db;

    fn envelope(version: u32, extra_arg: &str) -> String {
        serde_json::json!({
            "schema_version": version,
            "workspace": {
                "id": "old", "name": "Imported",
                "apps": [{
                    "id": "a", "workspace_id": "old", "name": "App",
                    "exe_path": "C:\\a.exe", "args": extra_arg,
                    "x": 0.0, "y": 0.0, "w": 0.5, "h": 0.5
                }]
            }
        })
        .to_string()
    }

    #[test]
    fn rejects_bad_schema_version() {
        let db = Db::open_in_memory().unwrap();
        let err = import_workspace(&db, &envelope(2, ""), true).unwrap_err();
        assert!(matches!(err, WorksetError::Validation(_)));
    }

    #[test]
    fn suspicious_needs_confirmation() {
        let db = Db::open_in_memory().unwrap();
        let json = envelope(1, "--x; rm -rf ~");
        assert!(import_workspace(&db, &json, false).is_err());
        let ws = import_workspace(&db, &json, true).unwrap();
        assert_eq!(ws.name, "Imported");
        assert_eq!(ws.apps.len(), 1);
        // ids are regenerated
        assert_ne!(ws.id, "old");
        assert_eq!(ws.apps[0].workspace_id, ws.id);
    }

    #[test]
    fn clean_import_no_confirm_needed() {
        let db = Db::open_in_memory().unwrap();
        let ws = import_workspace(&db, &envelope(1, "--clean"), false).unwrap();
        assert_eq!(ws.apps[0].args, vec!["--clean".to_owned()]);
    }
}
