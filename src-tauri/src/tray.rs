//! System tray: a minimal Show / Quit menu. The favorites launch list is
//! owned by the frontend, which invokes the regular launch commands.

use tauri::{App, Manager};

/// Build the tray icon + menu. Never fails startup: errors are ignored.
pub fn init_tray(app: &mut App) {
    use tauri::menu::{MenuBuilder, MenuItemBuilder};
    use tauri::tray::TrayIconBuilder;

    let menu = (|| {
        let show = MenuItemBuilder::with_id("show", "Show Workset").build(app)?;
        let quit = MenuItemBuilder::with_id("quit", "Quit").build(app)?;
        MenuBuilder::new(app).items(&[&show, &quit]).build()
    })();
    let menu = match menu {
        Ok(m) => m,
        Err(e) => {
            eprintln!("[workset] tray menu failed: {e}");
            return;
        }
    };

    let mut builder = TrayIconBuilder::with_id("main")
        .menu(&menu)
        .tooltip("Workset")
        .show_menu_on_left_click(true);
    if let Some(icon) = app.default_window_icon().cloned() {
        builder = builder.icon(icon);
    }
    let result = builder
        .on_menu_event(|app, event| match event.id.as_ref() {
            "show" => {
                if let Some(win) = app.get_webview_window("main") {
                    let _ = win.show();
                    let _ = win.unminimize();
                    let _ = win.set_focus();
                }
            }
            "quit" => app.exit(0),
            _ => {}
        })
        .build(app);
    if let Err(e) = result {
        eprintln!("[workset] tray icon failed: {e}");
    }
}
