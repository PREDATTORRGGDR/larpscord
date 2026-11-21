//! Команды управления пользовательскими пресетами.

use crate::presets::Preset;
use crate::storage::Storage;
use tauri::State;

/// Создать/обновить пользовательский пресет (upsert по id).
#[tauri::command]
pub async fn save_preset(
    storage: State<'_, Storage>,
    preset: Preset,
) -> Result<serde_json::Value, String> {
    let (mut data, _) = storage.load_or_default();
    if preset.id.trim().is_empty() {
        return Err("идентификатор пресета пуст".into());
    }
    if preset.name.trim().is_empty() {
        return Err("имя пресета пустое".into());
    }
    let mut p = preset;
    p.builtin = false;
    let idx = data.user_presets.iter().position(|x| x.id == p.id);
    match idx {
        Some(i) => data.user_presets[i] = p,
        None => data.user_presets.push(p),
    }
    storage.save(&data).map_err(|e| e.to_string())?;
    Ok(serde_json::json!({ "ok": true }))
}

/// Удалить пользовательский пресет по id.
#[tauri::command]
pub async fn delete_preset(
    storage: State<'_, Storage>,
    id: String,
) -> Result<serde_json::Value, String> {
    let (mut data, _) = storage.load_or_default();
    let before = data.user_presets.len();
    data.user_presets.retain(|p| p.id != id);
    if data.user_presets.len() == before {
        return Err(format!("пресет {id} не найден"));
    }
    storage.save(&data).map_err(|e| e.to_string())?;
    Ok(serde_json::json!({ "ok": true }))
}

/// Все пресеты: встроенные + пользовательские.
#[tauri::command]
pub async fn list_presets(storage: State<'_, Storage>) -> Result<serde_json::Value, String> {
    let (data, _) = storage.load_or_default();
    let mut all = crate::presets::builtin_presets();
    all.extend(data.user_presets);
    serde_json::to_value(all).map_err(|e| e.to_string())
}

/// Экспорт всех пользовательских пресетов в JSON-строку.
#[tauri::command]
pub async fn export_presets(storage: State<'_, Storage>) -> Result<String, String> {
    let (data, _) = storage.load_or_default();
    serde_json::to_string_pretty(&data.user_presets).map_err(|e| e.to_string())
}

/// Импорт пресетов из JSON-строки (merge по id).
#[tauri::command]
pub async fn import_presets(
    storage: State<'_, Storage>,
    json: String,
) -> Result<serde_json::Value, String> {
    let imported: Vec<Preset> = serde_json::from_str(&json)
        .map_err(|e| format!("не удалось разобрать JSON: {e}"))?;
    if imported.is_empty() {
        return Err("в файле нет пресетов".into());
    }
    let (mut data, _) = storage.load_or_default();
    let mut added = 0;
    let mut updated = 0;
    for mut p in imported {
        if p.id.trim().is_empty() || p.name.trim().is_empty() {
            continue;
        }
        p.builtin = false;
        match data.user_presets.iter().position(|x| x.id == p.id) {
            Some(i) => {
                data.user_presets[i] = p;
                updated += 1;
            }
            None => {
                data.user_presets.push(p);
                added += 1;
            }
        }
    }
    storage.save(&data).map_err(|e| e.to_string())?;
    Ok(serde_json::json!({ "added": added, "updated": updated }))
}
