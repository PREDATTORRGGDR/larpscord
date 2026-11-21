//! Larpscord — студия настройки Discord Rich Presence.
//! Библиотечный корень Tauri-приложения.

pub mod commands;
pub mod discord;
pub mod integrations;
pub mod presence;
pub mod presets;
pub mod storage;

use discord::manager::DiscordManager;
use std::sync::Arc;
use storage::Storage;
use tauri::Manager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_single_instance::init(|app, _args, _cwd| {
            // Второй запуск: показываем уже открытое окно.
            if let Some(win) = app.get_webview_window("main") {
                let _ = win.show();
                let _ = win.set_focus();
            }
        }))
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_autostart::init(
            tauri_plugin_autostart::MacosLauncher::LaunchAgent,
            None,
        ))
        .setup(|app| {
            // Хранилище.
            let storage = Storage::new().expect("не удалось создать каталог данных");
            let (data, corruption) = storage.load_or_default();
            if let Some(w) = &corruption {
                eprintln!("[larpscord] config повреждён, восстановлено: {w}");
            }

            // Менеджер Discord.
            let manager = Arc::new(DiscordManager::new());
            if let Some(custom) = &data.settings.custom_client_id {
                let cid = custom.trim();
                if !cid.is_empty() {
                    manager.set_client_id(cid.to_string());
                }
            }
            manager.spawn(app.handle().clone());

            app.manage(storage);
            app.manage(manager);
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::get_app_state,
            commands::apply_activity,
            commands::clear_presence,
            commands::get_settings,
            commands::save_settings,
            commands::get_connection_status,
            commands::reconnect_discord,
            commands::save_last_activity,
            commands::get_builtin_presets,
        ])
        .run(tauri::generate_context!())
        .expect("ошибка запуска Larpscord");
}
