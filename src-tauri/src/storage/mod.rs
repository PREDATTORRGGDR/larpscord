//! Локальное хранилище: JSON-файлы в каталоге данных приложения.
//! Устойчивость: атомарная запись (tmp + rename), бэкап повреждённых
//! файлов, версионирование и миграции.

use serde::{Deserialize, Serialize};
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};

/// Текущая версия формата хранилища.
pub const STORAGE_VERSION: u32 = 1;

/// Все настройки и данные Larpscord.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppData {
    pub version: u32,
    #[serde(default)]
    pub settings: Settings,
    /// Последняя редактируемая активность (автосохранение редактора).
    #[serde(default)]
    pub last_activity: Option<crate::presence::Activity>,
    /// Активность, которую нужно восстановить при запуске (если включено).
    #[serde(default)]
    pub active_activity: Option<crate::presence::Activity>,
    /// Пользовательские пресеты.
    #[serde(default)]
    pub user_presets: Vec<crate::presets::Preset>,
}

impl Default for AppData {
    fn default() -> Self {
        Self {
            version: STORAGE_VERSION,
            settings: Settings::default(),
            last_activity: None,
            active_activity: None,
            user_presets: Vec::new(),
        }
    }
}

/// Настройки приложения.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct Settings {
    /// Подключаться к Discord автоматически при запуске.
    pub autoconnect: bool,
    /// Восстанавливать последнюю активность при запуске.
    pub restore_activity: bool,
    /// Автоматически применять активность после реконнекта Discord.
    pub auto_reapply: bool,
    /// Минимизировать в трей вместо закрытия.
    pub minimize_to_tray: bool,
    /// Запускать свёрнутым (в трей).
    pub start_minimized: bool,
    /// Запуск при старте Windows.
    pub launch_on_startup: bool,
    /// Пользовательский Application ID (если задан — используется вместо встроенного).
    pub custom_client_id: Option<String>,
    /// Язык интерфейса ("ru" по умолчанию).
    pub language: String,
    /// Тема оформления.
    pub theme: String,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            autoconnect: true,
            restore_activity: false,
            auto_reapply: true,
            minimize_to_tray: true,
            start_minimized: false,
            launch_on_startup: false,
            custom_client_id: None,
            language: "ru".into(),
            theme: "black-liquid-glass".into(),
        }
    }
}

/// Хранилище на диске.
pub struct Storage {
    dir: PathBuf,
}

#[derive(Debug, thiserror::Error)]
pub enum StorageError {
    #[error("не найден каталог данных приложения")]
    NoDataDir,
    #[error("ошибка ввода-вывода: {0}")]
    Io(#[from] std::io::Error),
    #[error("не удалось разобрать {file}: {reason}")]
    Corrupted { file: String, reason: String },
}

impl Storage {
    /// Создаёт хранилище в %APPDATA%/Larpscord (или эквиваленте).
    pub fn new() -> Result<Self, StorageError> {
        let base = dirs::data_dir().ok_or(StorageError::NoDataDir)?;
        let dir = base.join("Larpscord");
        fs::create_dir_all(&dir)?;
        Ok(Self { dir })
    }

    fn data_path(&self) -> PathBuf {
        self.dir.join("config.json")
    }

    fn backup_path(&self) -> PathBuf {
        self.dir.join("config.corrupt.bak")
    }

    /// Загружает данные; при повреждении сохраняет бэкап и начинает заново.
    pub fn load(&self) -> Result<AppData, StorageError> {
        let path = self.data_path();
        if !path.exists() {
            return Ok(AppData::default());
        }
        let raw = fs::read_to_string(&path)?;
        let mut data: AppData = match serde_json::from_str(&raw) {
            Ok(d) => d,
            Err(e) => {
                // Сохраняем повреждённый файл для диагностики и начинаем заново.
                let _ = fs::copy(&path, self.backup_path());
                return Err(StorageError::Corrupted {
                    file: path.display().to_string(),
                    reason: e.to_string(),
                });
            }
        };
        migrate(&mut data);
        Ok(data)
    }

    /// Загружает данные, при повреждении — бэкап + дефолт (не падаем).
    pub fn load_or_default(&self) -> (AppData, Option<String>) {
        match self.load() {
            Ok(d) => (d, None),
            Err(e) => (AppData::default(), Some(e.to_string())),
        }
    }

    /// Атомарно сохраняет данные.
    pub fn save(&self, data: &AppData) -> Result<(), StorageError> {
        let path = self.data_path();
        let tmp = self.dir.join("config.json.tmp");
        {
            let mut f = fs::File::create(&tmp)?;
            let body = serde_json::to_string_pretty(data)
                .map_err(|e| StorageError::Corrupted {
                    file: "serialize".into(),
                    reason: e.to_string(),
                })?;
            f.write_all(body.as_bytes())?;
            f.sync_all()?;
        }
        atomic_rename(&tmp, &path)?;
        Ok(())
    }

    /// Путь к каталогу хранилища (для страницы настроек).
    pub fn dir_path(&self) -> &Path {
        &self.dir
    }
}

fn atomic_rename(tmp: &Path, dest: &Path) -> Result<(), StorageError> {
    // Windows: rename поверх существующего файла — через remove + rename.
    if dest.exists() {
        let bak = dest.with_extension("json.prev");
        let _ = fs::copy(dest, &bak);
        fs::remove_file(dest)?;
    }
    fs::rename(tmp, dest)?;
    Ok(())
}

/// Миграции между версиями формата. Сейчас версия 1 — базовая.
fn migrate(data: &mut AppData) {
    if data.version == 0 {
        data.version = 1;
    }
    data.version = data.version.max(1).min(STORAGE_VERSION);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_roundtrip() {
        let d = AppData::default();
        let s = serde_json::to_string(&d).unwrap();
        let back: AppData = serde_json::from_str(&s).unwrap();
        assert_eq!(back.version, STORAGE_VERSION);
        assert!(back.settings.autoconnect);
    }

    #[test]
    fn corrupted_gives_error() {
        let json = r#"{"version": 1, "settings": {"autoconnect": "not-a-bool"}}"#;
        assert!(serde_json::from_str::<AppData>(json).is_err());
    }

    #[test]
    fn missing_fields_use_defaults() {
        let json = r#"{"version": 1}"#;
        let d: AppData = serde_json::from_str(json).unwrap();
        assert!(d.settings.autoconnect);
        assert_eq!(d.settings.language, "ru");
    }
}
