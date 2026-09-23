//! Workset — Windows Workspace Manager backend entry point.

mod capture;
mod commands;
mod coords;
mod database;
mod discovery;
mod error;
mod import_export;
mod launcher;
mod matching;
mod models;
mod monitors;
mod processes;
mod sessions;
mod settings;
mod tray;
mod windows;

use tauri::Manager;

use crate::database::Db;

fn db_path() -> std::path::PathBuf {
    if let Some(base) = dirs::data_local_dir() {
        return base.join("Workset").join("workset.db");
    }
    std::path::PathBuf::from("workset.db")
}

fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_single_instance::init(|app, _args, _cwd| {
            if let Some(win) = app.get_webview_window("main") {
                let _ = win.show();
                let _ = win.unminimize();
                let _ = win.set_focus();
            }
        }))
        .plugin(tauri_plugin_autostart::init(
            tauri_plugin_autostart::MacosLauncher::LaunchAgent,
            None,
        ))
        .plugin(tauri_plugin_global_shortcut::Builder::new().build())
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .setup(|app| {
            let path = db_path();
            let db = Db::open(&path).expect("failed to open Workset database");
            app.manage(db);
            tray::init_tray(app);
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::get_monitors,
            commands::enum_windows,
            commands::capture_desktop,
            commands::discover_apps,
            commands::get_workspaces,
            commands::get_workspace,
            commands::create_workspace,
            commands::update_workspace,
            commands::delete_workspace,
            commands::duplicate_workspace,
            commands::toggle_favorite,
            commands::create_app,
            commands::update_app,
            commands::delete_app,
            commands::reorder_apps,
            commands::launch_workspace,
            commands::launch_workspace_wait,
            commands::cancel_launch,
            commands::get_sessions,
            commands::get_session,
            commands::restart_session,
            commands::close_session_app,
            commands::close_workspace_session,
            commands::recover_offscreen,
            commands::import_workspace,
            commands::export_workspace,
            commands::get_settings,
            commands::set_setting,
            commands::get_logs,
            commands::register_hotkey,
            commands::unregister_hotkey,
            commands::set_autostart,
        ])
        .run(tauri::generate_context!())
        .expect("error while running Workset");
}
