//! Tauri-команды: мост между React-фронтендом и Rust-бэкендом.
//! Все команды async — выполняются на tokio-рантайме Tauri.

pub mod music;
pub mod presets;

use crate::discord::manager::DiscordManager;
use crate::presence::Activity;
use crate::storage::{Settings, Storage};
use serde_json::Value;
use std::sync::Arc;
use tauri::{AppHandle, Emitter, Manager, State};

#[tauri::command]
pub async fn get_app_state(
    storage: State<'_, Storage>,
    manager: State<'_, Arc<DiscordManager>>,
) -> Result<Value, String> {
    let (data, corruption) = storage.load_or_default();
    let state = manager.state().await;
    Ok(serde_json::json!({
        "settings": data.settings,
        "lastActivity": data.last_activity,
        "activeActivity": data.active_activity,
        "connection": state,
        "builtinClientId": crate::discord::manager::BUILTIN_CLIENT_ID,
        "customClientId": manager.current_client_id(),
        "storageDir": storage.dir_path().display().to_string(),
        "corruptionWarning": corruption,
        "appVersion": env!("CARGO_PKG_VERSION"),
    }))
}

#[tauri::command]
pub async fn apply_activity(
    app: AppHandle,
    storage: State<'_, Storage>,
    manager: State<'_, Arc<DiscordManager>>,
    activity: Activity,
) -> Result<Value, String> {
    // Валидация уже внутри manager.set_activity.
    manager.set_activity(activity.clone()).await?;

    // Сохраняем как последнюю и активную.
    let (mut data, _) = storage.load_or_default();
    data.last_activity = Some(activity.clone());
    data.active_activity = Some(activity);
    storage.save(&data).map_err(|e| e.to_string())?;

    // Уведомляем фронтенд об изменении статуса.
    if let Some(win) = app.get_webview_window("main") {
        let _ = win.emit("activity://changed", ());
    }
    Ok(serde_json::json!({ "ok": true }))
}

#[tauri::command]
pub async fn clear_presence(
    storage: State<'_, Storage>,
    manager: State<'_, Arc<DiscordManager>>,
) -> Result<Value, String> {
    manager.clear_activity().await?;
    let (mut data, _) = storage.load_or_default();
    data.active_activity = None;
    storage.save(&data).map_err(|e| e.to_string())?;
    Ok(serde_json::json!({ "ok": true }))
}

#[tauri::command]
pub async fn get_settings(storage: State<'_, Storage>) -> Result<Settings, String> {
    let (data, _) = storage.load_or_default();
    Ok(data.settings)
}

#[tauri::command]
pub async fn save_settings(
    storage: State<'_, Storage>,
    manager: State<'_, Arc<DiscordManager>>,
    settings: Settings,
) -> Result<Value, String> {
    let (mut data, _) = storage.load_or_default();
    // Если сменился client_id — менеджер должен использовать новый.
    let new_cid = settings.custom_client_id.clone();
    data.settings = settings;
    storage.save(&data).map_err(|e| e.to_string())?;

    if let Some(cid) = new_cid {
        let cid = cid.trim().to_string();
        let builtin = crate::discord::manager::BUILTIN_CLIENT_ID;
        let effective = if cid.is_empty() { builtin.to_string() } else { cid };
        manager.set_client_id(effective);
    }
    Ok(serde_json::json!({ "ok": true }))
}

#[tauri::command]
pub async fn get_connection_status(
    manager: State<'_, Arc<DiscordManager>>,
) -> Result<Value, String> {
    let state = manager.state().await;
    serde_json::to_value(state).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn reconnect_discord(manager: State<'_, Arc<DiscordManager>>) -> Result<Value, String> {
    // Менеджер сам переподключится по keepalive-циклу; форсируем через
    // повторное применение активной активности.
    if let Some(a) = manager.live_activity() {
        manager.set_activity(a).await?;
    }
    Ok(serde_json::json!({ "ok": true }))
}

#[tauri::command]
pub async fn save_last_activity(
    storage: State<'_, Storage>,
    activity: Activity,
) -> Result<Value, String> {
    let (mut data, _) = storage.load_or_default();
    data.last_activity = Some(activity);
    storage.save(&data).map_err(|e| e.to_string())?;
    Ok(serde_json::json!({ "ok": true }))
}
