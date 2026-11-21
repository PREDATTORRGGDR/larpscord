//! Команды музыкального режима.

use crate::music::watcher::{MusicMode, MusicWatcher};
use crate::music::smtc::MediaSnapshot;
use std::sync::Arc;
use tauri::State;

#[tauri::command]
pub async fn music_poll(
    watcher: State<'_, Arc<MusicWatcher>>,
) -> Result<MediaSnapshot, String> {
    Ok(watcher.poll_once().await)
}

#[tauri::command]
pub async fn music_set_mode(
    watcher: State<'_, Arc<MusicWatcher>>,
    mode: String,
) -> Result<serde_json::Value, String> {
    let m = match mode.as_str() {
        "off" => MusicMode::Off,
        "manual" => MusicMode::Manual,
        "auto" => MusicMode::Auto,
        other => return Err(format!("неизвестный режим: {other}")),
    };
    watcher.set_mode(m);
    Ok(serde_json::json!({ "ok": true }))
}

#[tauri::command]
pub async fn music_cover(
    watcher: State<'_, Arc<MusicWatcher>>,
) -> Result<serde_json::Value, String> {
    Ok(serde_json::json!({ "dataUrl": watcher.cover_data_url() }))
}

#[tauri::command]
pub async fn music_state(
    watcher: State<'_, Arc<MusicWatcher>>,
) -> Result<serde_json::Value, String> {
    let mode = match watcher.mode() {
        MusicMode::Off => "off",
        MusicMode::Manual => "manual",
        MusicMode::Auto => "auto",
    };
    Ok(serde_json::json!({
        "mode": mode,
        "snapshot": watcher.snapshot(),
        "cover": watcher.cover_data_url(),
    }))
}
