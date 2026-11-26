#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use hindsight_core::{
    db::{self, HistoryItem, SearchOptions},
    settings,
};
use std::sync::atomic::{AtomicBool, Ordering};
use tauri::{
    menu::{MenuBuilder, MenuItemBuilder},
    AppHandle, Emitter, Manager, RunEvent,
};
use tauri_plugin_global_shortcut::{Code, GlobalShortcutExt, Modifiers, Shortcut, ShortcutState};

static ALLOW_EXIT: AtomicBool = AtomicBool::new(false);

fn show_search_window(app_handle: &AppHandle) {
    #[cfg(target_os = "macos")]
    let _ = app_handle.set_activation_policy(tauri::ActivationPolicy::Regular);

    if let Some(window) = app_handle.get_webview_window("search") {
        let _ = window.unminimize();
        let _ = window.show();
        let _ = window.set_focus();
        let _ = app_handle.emit("window-shown", ());
    }
}

fn hide_search_window(app_handle: &AppHandle) {
    #[cfg(target_os = "macos")]
    let _ = app_handle.set_activation_policy(tauri::ActivationPolicy::Accessory);

    if let Some(window) = app_handle.get_webview_window("search") {
        let _ = window.hide();
    }
}

fn toggle_search_window(app_handle: &AppHandle) {
    if let Some(window) = app_handle.get_webview_window("search") {
        let visible = window.is_visible().unwrap_or(false);
        let focused = window.is_focused().unwrap_or(false);
        if visible && focused {
            hide_search_window(app_handle);
        } else {
            show_search_window(app_handle);
        }
    }
}

#[tauri::command]
fn search_commands(
    query: String,
    failed_only: Option<bool>,
    limit: Option<usize>,
) -> Result<Vec<HistoryItem>, String> {
    let conn = db::open().map_err(|e| e.to_string())?;
    db::search(
        &conn,
        SearchOptions {
            query: &query,
            cwd: None,
            exit_code: None,
            failed_only: failed_only.unwrap_or(false),
            limit: limit.unwrap_or(50),
        },
    )
    .map_err(|e| e.to_string())
}

#[tauri::command]
fn get_recent(limit: Option<usize>) -> Result<Vec<HistoryItem>, String> {
    let conn = db::open().map_err(|e| e.to_string())?;
    db::recent(&conn, limit.unwrap_or(50)).map_err(|e| e.to_string())
}

#[tauri::command]
fn get_settings() -> Result<settings::Settings, String> {
    settings::load().map_err(|e| e.to_string())
}

#[tauri::command]
fn update_settings(settings: settings::Settings) -> Result<settings::Settings, String> {
    let saved = settings::save(&settings).map_err(|e| e.to_string())?;
    let conn = db::open().map_err(|e| e.to_string())?;
    db::prune(&conn, saved.retention_days).map_err(|e| e.to_string())?;
    Ok(saved)
}

#[tauri::command]
fn clear_history() -> Result<usize, String> {
    let conn = db::open().map_err(|e| e.to_string())?;
    db::clear_history(&conn).map_err(|e| e.to_string())
}

fn main() {
    let toggle_shortcut = Shortcut::new(Some(Modifiers::SUPER | Modifiers::ALT), Code::KeyH);

    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(
            tauri_plugin_global_shortcut::Builder::new()
                .with_handler(move |app, shortcut, event| {
                    if shortcut == &toggle_shortcut && event.state() == ShortcutState::Pressed {
                        toggle_search_window(app);
                    }
                })
                .build(),
        )
        .invoke_handler(tauri::generate_handler![
            search_commands,
            get_recent,
            get_settings,
            update_settings,
            clear_history
        ])
        .setup(move |app| {
            if let Err(err) = app.global_shortcut().register(toggle_shortcut) {
                eprintln!("hindsight: failed to register global shortcut: {err}");
            }

            // Build tray menu
            let quit = MenuItemBuilder::with_id("quit", "Quit Hindsight").build(app)?;
            let menu = MenuBuilder::new(app).items(&[&quit]).build()?;

            // Set up tray icon
            if let Some(tray) = app.tray_by_id("main") {
                let _ = tray.set_menu(Some(menu));
                let _ = tray.set_tooltip(Some("Hindsight - Shell History (Cmd+Option+H)"));

                // Left click only → show window + show dock icon
                let app_handle = app.handle().clone();
                tray.on_tray_icon_event(move |_tray, event| {
                    if let tauri::tray::TrayIconEvent::Click { button, .. } = event {
                        if button == tauri::tray::MouseButton::Left {
                            show_search_window(&app_handle);
                        }
                    }
                });

                // Menu item click → quit
                let quit_handle = app.handle().clone();
                tray.on_menu_event(move |_app, event| {
                    if event.id().as_ref() == "quit" {
                        ALLOW_EXIT.store(true, Ordering::SeqCst);
                        quit_handle.exit(0);
                    }
                });
            }

            // Show the search window on launch. Closing it still returns to the tray.
            show_search_window(app.handle());

            if let Some(window) = app.get_webview_window("search") {
                // Close button → hide window + hide dock icon
                let app_handle = app.handle().clone();
                window.on_window_event(move |event| {
                    if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                        api.prevent_close();
                        hide_search_window(&app_handle);
                    }
                });
            }

            Ok(())
        })
        .build(tauri::generate_context!())
        .expect("error while building tauri application")
        .run(|app_handle, event| match event {
            RunEvent::ExitRequested { api, .. } => {
                if !ALLOW_EXIT.load(Ordering::SeqCst) {
                    api.prevent_exit();
                }
            }
            #[cfg(target_os = "macos")]
            RunEvent::Reopen { .. } => {
                show_search_window(app_handle);
            }
            _ => {}
        });
}
