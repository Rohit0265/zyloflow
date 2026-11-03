pub mod audio;
pub mod commands;
pub mod error;
pub mod hotkeys;
pub mod lifecycle;
pub mod popup;
pub mod refinement;
pub mod secure_storage;
pub mod settings;
pub mod text_injection;
pub mod tray;

use tauri::{Manager, WindowEvent};
use lifecycle::AppState;
use settings::SettingsManager;

pub fn run() {
    let app_state = AppState::new();

    tauri::Builder::default()
        .plugin(tauri_plugin_global_shortcut::Builder::new().build())
        .plugin(tauri_plugin_clipboard_manager::init())
        .plugin(tauri_plugin_autostart::init(
            tauri_plugin_autostart::MacosLauncher::LaunchAgent,
            Some(vec!["--minimized"]),
        ))
        .manage(app_state)
        .setup(|app| {
            let handle = app.handle().clone();

            // Setup System Tray
            if let Err(e) = tray::create_tray(&handle) {
                log::error!("Failed to create system tray: {}", e);
            }

            // Register initial global hotkeys
            if let Err(e) = hotkeys::HotkeyManager::register_all(&handle) {
                log::warn!("Global hotkey registration notice: {}", e);
            }

            // Check if launched with --minimized or settings has start_minimized
            let settings = SettingsManager::load_settings();
            let args: Vec<String> = std::env::args().collect();
            let launched_minimized = args.contains(&"--minimized".to_string()) || settings.start_minimized;

            if launched_minimized {
                if let Some(main_win) = app.get_webview_window("main") {
                    let _ = main_win.hide();
                }
            }

            Ok(())
        })
        .on_window_event(|window, event| {
            if let WindowEvent::CloseRequested { api, .. } = event {
                // If closing main window and background mode is enabled, hide instead of exiting
                if window.label() == "main" {
                    let settings = SettingsManager::load_settings();
                    if settings.background_mode {
                        api.prevent_close();
                        let _ = window.hide();
                    }
                }
            }
        })
        .invoke_handler(tauri::generate_handler![
            commands::get_settings,
            commands::save_settings,
            commands::get_api_key_configured,
            commands::save_api_key,
            commands::delete_api_key,
            commands::test_api_key,
            commands::get_system_status,
            commands::start_dictation,
            commands::stop_dictation,
            commands::cancel_dictation,
            commands::set_app_active,
            commands::get_history,
            commands::delete_history_item,
            commands::clear_history,
            commands::register_hotkeys,
            commands::check_microphone_permission,
            commands::complete_onboarding,
            commands::hide_main_window,
            commands::hide_popup_window,
        ])
        .run(tauri::generate_context!())
        .expect("error while running ZyloFlow application");
}
