//! Larpscord — студия настройки Discord Rich Presence.
//! Библиотечный корень Tauri-приложения.

pub mod commands;
pub mod discord;
pub mod integrations;
pub mod music;
pub mod presence;
pub mod presets;
pub mod storage;

use discord::manager::DiscordManager;
use music::watcher::MusicWatcher;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use storage::Storage;
use tauri::{
    menu::{Menu, MenuItem},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    Manager, WindowEvent,
};

/// Приложение остаётся в фоне (трей) после закрытия окна.
static RUN_IN_BACKGROUND: AtomicBool = AtomicBool::new(true);

/// Изменить режим «сворачивать в трей» на лету.
pub fn set_run_in_background(on: bool) {
    RUN_IN_BACKGROUND.store(on, Ordering::Relaxed);
}

/// Диагностический лог (файл в каталоге данных, для отладки трея).
fn log_line(msg: &str) {
    use std::io::Write;
    if let Ok(dir) = std::env::var("APPDATA") {
        let d = std::path::PathBuf::from(dir).join("Larpscord");
        let _ = std::fs::create_dir_all(&d);
        if let Ok(mut f) = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(d.join("tray-debug.log"))
        {
            let _ = writeln!(f, "{msg}");
        }
    }
}

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
            // Фоновой режим берём из настроек (по умолчанию включён).
            RUN_IN_BACKGROUND.store(data.settings.minimize_to_tray, Ordering::Relaxed);

            // Менеджер Discord.
            let manager = Arc::new(DiscordManager::new());
            if let Some(custom) = &data.settings.custom_client_id {
                let cid = custom.trim();
                if !cid.is_empty() {
                    manager.set_client_id(cid.to_string());
                }
            }
            manager.spawn(app.handle().clone());

            // Слежение за музыкой (SMTC).
            let watcher = Arc::new(MusicWatcher::new(Arc::clone(&manager)));
            watcher.spawn(app.handle().clone());

            // Трей: меню + иконка.
            let show = MenuItem::with_id(app, "show", "Открыть Larpscord", true, None::<&str>)?;
            let quit = MenuItem::with_id(app, "quit", "Выход", true, None::<&str>)?;
            let menu = Menu::with_items(app, &[&show, &quit])?;

            let _tray = TrayIconBuilder::with_id("main-tray")
                .icon(app.default_window_icon().unwrap().clone())
                .tooltip("Larpscord — нажми, чтобы открыть")
                .menu(&menu)
                .show_menu_on_left_click(false)
                .on_menu_event(|app, event| match event.id.as_ref() {
                    "show" => {
                        if let Some(win) = app.get_webview_window("main") {
                            let _ = win.show();
                            let _ = win.set_focus();
                        }
                    }
                    "quit" => {
                        RUN_IN_BACKGROUND.store(false, Ordering::Relaxed);
                        app.exit(0);
                    }
                    _ => {}
                })
                .on_tray_icon_event(|tray, event| {
                    // ЛКМ — показать окно.
                    if let TrayIconEvent::Click {
                        button: MouseButton::Left,
                        button_state: MouseButtonState::Up,
                        ..
                    } = event
                    {
                        let app = tray.app_handle();
                        if let Some(win) = app.get_webview_window("main") {
                            let _ = win.show();
                            let _ = win.set_focus();
                        }
                    }
                })
                .build(app)?;

            // Обработчик закрытия на самом окне (надёжный путь в Tauri 2).
            if let Some(win) = app.get_webview_window("main") {
                let handle = win.clone();
                win.on_window_event(move |event| {
                    if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                        let bg = RUN_IN_BACKGROUND.load(Ordering::Relaxed);
                        log_line(&format!("win_close_requested bg={bg}"));
                        if bg {
                            api.prevent_close();
                            let _ = handle.hide();
                            log_line("hidden_to_tray");
                        }
                    }
                });
            }

            app.manage(storage);
            app.manage(manager);
            app.manage(watcher);
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
            commands::music::music_poll,
            commands::music::music_set_mode,
            commands::music::music_cover,
            commands::music::music_state,
            commands::music::upload_cover,
            commands::presets::save_preset,
            commands::presets::delete_preset,
            commands::presets::list_presets,
            commands::presets::export_presets,
            commands::presets::import_presets,
        ])
        .on_window_event(|window, event| {
            // Закрытие окна → скрыть в трей вместо выхода.
            match event {
                WindowEvent::CloseRequested { api, .. } => {
                    let bg = RUN_IN_BACKGROUND.load(Ordering::Relaxed);
                    log_line(&format!("close_requested bg={bg} window={}", window.label()));
                    if bg {
                        api.prevent_close();
                        let _ = window.hide();
                        log_line("hidden_to_tray");
                    }
                }
                WindowEvent::Destroyed => {
                    log_line(&format!("destroyed window={}", window.label()));
                }
                _ => {}
            }
        })
        .build(tauri::generate_context!())
        .expect("ошибка сборки Larpscord")
        .run(|_app, event| {
            // Выход из цикла событий: если окна просто скрыты — не выходим.
            if let tauri::RunEvent::ExitRequested { code, api, .. } = event {
                let bg = RUN_IN_BACKGROUND.load(Ordering::Relaxed);
                log_line(&format!("exit_requested code={code:?} bg={bg}"));
                if bg && code.is_none() {
                    api.prevent_exit();
                }
            }
        });
}
