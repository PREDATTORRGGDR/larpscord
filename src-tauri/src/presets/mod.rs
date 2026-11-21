//! Пресеты Rich Presence: встроенные + пользовательские, JSON-хранилище.
//! (Полная система — Этап 2; для MVP — встроенные пресеты в редакторе.)

use crate::presence::{Activity, ActivityButton, ActivityType};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Preset {
    pub id: String,
    pub name: String,
    pub category: String,
    pub activity: Activity,
    pub builtin: bool,
}

/// Встроенные пресеты Larpscord.
pub fn builtin_presets() -> Vec<Preset> {
    let mk = |id: &str, name: &str, cat: &str, a: Activity| Preset {
        id: id.into(),
        name: name.into(),
        category: cat.into(),
        activity: a,
        builtin: true,
    };
    vec![
        mk("gaming", "Игры", "Игры", Activity {
            name: Some("Игра".into()),
            details: Some("Играю в любимую игру".into()),
            state: Some("Ранкед, победа близко".into()),
            activity_type: ActivityType::Playing,
            timestamps_start: Some(chrono_now()),
            ..Activity::default()
        }),
        mk("music", "Музыка", "Музыка", Activity {
            name: Some("Музыка".into()),
            details: Some("Трек — Исполнитель".into()),
            state: Some("Альбом".into()),
            activity_type: ActivityType::Listening,
            timestamps_start: Some(chrono_now()),
            ..Activity::default()
        }),
        mk("dev", "Разработка", "Работа", Activity {
            name: Some("VS Code".into()),
            details: Some("Пишу код на Rust".into()),
            state: Some("В ветке feature/larpscord".into()),
            activity_type: ActivityType::Playing,
            timestamps_start: Some(chrono_now()),
            ..Activity::default()
        }),
        mk("movie", "Кино", "Развлечения", Activity {
            name: Some("Кино".into()),
            details: Some("Смотрю фильм".into()),
            state: Some("Ужин и плед".into()),
            activity_type: ActivityType::Watching,
            timestamps_start: Some(chrono_now()),
            ..Activity::default()
        }),
        mk("work", "Работа", "Работа", Activity {
            name: Some("Работа".into()),
            details: Some("Работаю над проектом".into()),
            state: Some("Сфокусирован".into()),
            activity_type: ActivityType::Playing,
            timestamps_start: Some(chrono_now()),
            ..Activity::default()
        }),
        mk("minimal", "Минимал", "Другое", Activity {
            name: None,
            details: None,
            state: None,
            activity_type: ActivityType::Playing,
            ..Activity::default()
        }),
    ]
}

fn chrono_now() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0)
}

/// Готовая активность «Слушает музыку» с кнопкой (используется Music-студией).
pub fn music_activity_with_button(track: &str, artist: &str, url: Option<String>) -> Activity {
    Activity {
        name: None,
        details: Some(format!("{track} — {artist}")),
        state: None,
        activity_type: ActivityType::Listening,
        timestamps_start: Some(chrono_now()),
        timestamps_end: None,
        large_image: None,
        large_text: None,
        small_image: None,
        small_text: None,
        buttons: match url {
            Some(u) if u.starts_with("https://") => vec![ActivityButton { label: "Слушать".into(), url: u }],
            _ => vec![],
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn builtin_presets_valid() {
        for p in builtin_presets() {
            assert!(p.activity.validate().is_ok(), "пресет {} невалиден", p.name);
        }
    }
}
