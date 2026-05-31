#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod db;

use tauri::{
    Emitter, Manager, RunEvent,
    menu::{MenuBuilder, MenuItemBuilder},
};

#[tauri::command]
fn search_commands(query: String, limit: Option<usize>) -> Result<Vec<String>, String> {
    let conn = db::open().map_err(|e| e.to_string())?;
    let limit = limit.unwrap_or(50);
    if query.is_empty() {
        db::recent(&conn, limit).map_err(|e| e.to_string())
    } else {
        db::search(&conn, &query, limit).map_err(|e| e.to_string())
    }
}

#[tauri::command]
fn get_recent(limit: Option<usize>) -> Result<Vec<String>, String> {
    let conn = db::open().map_err(|e| e.to_string())?;
    db::recent(&conn, limit.unwrap_or(50)).map_err(|e| e.to_string())
}

fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .invoke_handler(tauri::generate_handler![search_commands, get_recent])
        .setup(|app| {
            // Build tray menu
            let quit = MenuItemBuilder::with_id("quit", "Quit Hindsight").build(app)?;
            let menu = MenuBuilder::new(app).items(&[&quit]).build()?;

            // Set up tray icon
            if let Some(tray) = app.tray_by_id("main") {
                let _ = tray.set_menu(Some(menu));
                let _ = tray.set_tooltip(Some("Hindsight - Shell History"));

                // Left click only → show window + show dock icon
                let app_handle = app.handle().clone();
                tray.on_tray_icon_event(move |_tray, event| {
                    if let tauri::tray::TrayIconEvent::Click { button, .. } = event {
                        if button == tauri::tray::MouseButton::Left {
                            #[cfg(target_os = "macos")]
                            app_handle.set_activation_policy(tauri::ActivationPolicy::Regular);
                            if let Some(window) = app_handle.get_webview_window("search") {
                                let _ = window.show();
                                let _ = window.set_focus();
                                // Notify frontend to refresh
                                let _ = app_handle.emit("window-shown", ());
                            }
                        }
                    }
                });

                // Menu item click → quit
                tray.on_menu_event(move |_app, event| {
                    if event.id().as_ref() == "quit" {
                        std::process::exit(0);
                    }
                });
            }

            // Hide window on startup (tray-only, no dock icon)
            #[cfg(target_os = "macos")]
            app.set_activation_policy(tauri::ActivationPolicy::Accessory);

            if let Some(window) = app.get_webview_window("search") {
                let _ = window.hide();

                // Close button → hide window + hide dock icon
                let app_handle = app.handle().clone();
                window.on_window_event(move |event| {
                    if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                        api.prevent_close();
                        #[cfg(target_os = "macos")]
                        app_handle.set_activation_policy(tauri::ActivationPolicy::Accessory);
                        if let Some(w) = app_handle.get_webview_window("search") {
                            let _ = w.hide();
                        }
                    }
                });
            }

            Ok(())
        })
        .build(tauri::generate_context!())
        .expect("error while building tauri application")
        .run(|_app_handle, event| {
            if let RunEvent::ExitRequested { api, .. } = event {
                api.prevent_exit();
            }
        });
}
