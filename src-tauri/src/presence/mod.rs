//! Модели Discord Rich Presence (Activity) и их сериализация.
//!
//! Значения ActivityType проверены по документации Discord:
//! PLAYING=0, STREAMING=1, LISTENING=2, WATCHING=3, CUSTOM=4, COMPETING=5.
//! CUSTOM (=4) недоступен через RPC (только из настроек пользователя),
//! поэтому в редакторе не используется.

use serde::{Deserialize, Serialize};

/// Тип активности Discord (значения совпадают с протоколом).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[repr(u8)]
pub enum ActivityType {
    Playing = 0,
    Streaming = 1,
    Listening = 2,
    Watching = 3,
    Competing = 5,
}

impl ActivityType {
    pub fn display_name(&self) -> &'static str {
        match self {
            ActivityType::Playing => "Играет в",
            ActivityType::Streaming => "Транслирует",
            ActivityType::Listening => "Слушает",
            ActivityType::Watching => "Смотрит",
            ActivityType::Competing => "Соревнуется в",
        }
    }
}

impl Default for ActivityType {
    fn default() -> Self {
        ActivityType::Playing
    }
}

/// Кнопка в активности (максимум 2 по протоколу Discord).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ActivityButton {
    pub label: String,
    pub url: String,
}

impl ActivityButton {
    /// URL обязателен и должен быть http/https.
    pub fn is_url_valid(&self) -> bool {
        is_http_url(&self.url)
    }
}

/// Главная модель активности Larpscord — то, из чего собирается SET_ACTIVITY.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Activity {
    /// Имя приложения. Discord показывает его в профиле (сверху карточки).
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub details: Option<String>,
    #[serde(default)]
    pub state: Option<String>,
    /// Поле «type» протокола. Сериализуем числом.
    #[serde(rename = "type", default)]
    pub activity_type: ActivityType,
    /// Unix-время начала (сек).
    #[serde(default)]
    pub timestamps_start: Option<i64>,
    /// Unix-время окончания (сек).
    #[serde(default)]
    pub timestamps_end: Option<i64>,
    /// Большая картинка: внешний URL (Discord сам проксирует) или ключ ассета.
    #[serde(default)]
    pub large_image: Option<String>,
    #[serde(default)]
    pub large_text: Option<String>,
    #[serde(default)]
    pub small_image: Option<String>,
    #[serde(default)]
    pub small_text: Option<String>,
    #[serde(default)]
    pub buttons: Vec<ActivityButton>,
}

impl Default for Activity {
    fn default() -> Self {
        Self {
            name: None,
            details: Some("Настроено в Larpscord".into()),
            state: None,
            activity_type: ActivityType::Playing,
            timestamps_start: None,
            timestamps_end: None,
            large_image: None,
            large_text: None,
            small_image: None,
            small_text: None,
            buttons: vec![],
        }
    }
}

/// Лимиты Discord (проверены эмпирически и по документации).
pub const LIMIT_DETAILS: usize = 128;
pub const LIMIT_STATE: usize = 128;
pub const LIMIT_TOOLTIP: usize = 128;
pub const LIMIT_BUTTON_LABEL: usize = 32;
pub const LIMIT_BUTTON_URL: usize = 512;
pub const MAX_BUTTONS: usize = 2;

impl Activity {
    /// Проверяет лимиты Discord до отправки. Возвращает список ошибок.
    pub fn validate(&self) -> Result<(), Vec<String>> {
        let mut errors = Vec::new();
        let checks = [
            (self.details.as_deref(), LIMIT_DETAILS, "Детали"),
            (self.state.as_deref(), LIMIT_STATE, "Состояние"),
            (self.large_text.as_deref(), LIMIT_TOOLTIP, "Подсказка большой картинки"),
            (self.small_text.as_deref(), LIMIT_TOOLTIP, "Подсказка маленькой картинки"),
        ];
        for (field, limit, name) in checks {
            if let Some(s) = field {
                if s.chars().count() > limit {
                    errors.push(format!("{name}: лимит Discord — {limit} символов"));
                }
            }
        }
        for b in &self.buttons {
            if b.label.chars().count() > LIMIT_BUTTON_LABEL {
                errors.push(format!("Метка кнопки «{}»: лимит 32 символа", b.label));
            }
            if b.url.len() > LIMIT_BUTTON_URL {
                errors.push(format!("URL кнопки «{}»: лимит 512 символов", b.label));
            }
            if !b.label.trim().is_empty() && !b.is_url_valid() {
                errors.push(format!(
                    "URL кнопки «{label}» должен начинаться с http:// или https://",
                    label = b.label
                ));
            }
        }
        if self.buttons.len() > MAX_BUTTONS {
            errors.push("Discord поддерживает максимум 2 кнопки".into());
        }
        if let (Some(s), Some(e)) = (self.timestamps_start, self.timestamps_end) {
            if e <= s {
                errors.push("Время окончания должно быть позже начала".into());
            }
        }
        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }
}

/// Проверка, что строка — валидный http(s) URL.
pub fn is_http_url(s: &str) -> bool {
    let t = s.trim();
    if !(t.starts_with("http://") || t.starts_with("https://")) {
        return false;
    }
    // Схема есть — должен быть и непустой хост.
    let rest = &t[t.find("://").unwrap() + 3..];
    let host = rest.split(['/', '?', '#']).next().unwrap_or("");
    !host.is_empty() && host.chars().next().map_or(false, |c| c.is_ascii_alphanumeric())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn url_validation() {
        assert!(is_http_url("https://example.com"));
        assert!(is_http_url("http://example.com/a?b=c"));
        assert!(!is_http_url("ftp://example.com"));
        assert!(!is_http_url("javascript:alert(1)"));
        assert!(!is_http_url("https://"));
        assert!(!is_http_url(""));
        assert!(!is_http_url("https:///path"));
    }

    #[test]
    fn activity_type_values() {
        assert_eq!(ActivityType::Playing as u8, 0);
        assert_eq!(ActivityType::Streaming as u8, 1);
        assert_eq!(ActivityType::Listening as u8, 2);
        assert_eq!(ActivityType::Watching as u8, 3);
        assert_eq!(ActivityType::Competing as u8, 5);
    }

    #[test]
    fn activity_limits() {
        let mut a = Activity::default();
        assert!(a.validate().is_ok());
        a.details = Some("x".repeat(129));
        assert!(a.validate().is_err());
        a.details = Some("ok".into());
        a.buttons = vec![
            ActivityButton { label: "A".into(), url: "https://a.dev".into() },
            ActivityButton { label: "B".into(), url: "https://b.dev".into() },
            ActivityButton { label: "C".into(), url: "https://c.dev".into() },
        ];
        assert!(a.validate().is_err());
    }

    #[test]
    fn button_url_must_be_http() {
        let b = ActivityButton { label: "X".into(), url: "ftp://x".into() };
        assert!(!b.is_url_valid());
    }
}
