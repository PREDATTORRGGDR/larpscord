//! Слежение за музыкой: опрос SMTC каждую секунду, публикация
//! музыкальной активности в Discord с обложкой и прогрессом.

use super::smtc::MediaSnapshot;
use crate::discord::manager::DiscordManager;
use crate::presence::{Activity, ActivityType};
use std::sync::Arc;
use std::time::Duration;
use tauri::AppHandle;

/// Состояние музыкального режима.
#[derive(Debug, Clone, PartialEq)]
pub enum MusicMode {
    Off,
    /// Ручной режим (пользователь сам заполнил поля).
    Manual,
    /// Авто: следим за SMTC.
    Auto,
}

pub struct MusicWatcher {
    mode: Arc<parking_lot::RwLock<MusicMode>>,
    /// Кэш последней обложки (base64 data URL) для превью.
    last_cover: Arc<parking_lot::RwLock<Option<String>>>,
    /// Кэш последнего снимка для UI.
    last_snapshot: Arc<parking_lot::RwLock<Option<MediaSnapshot>>>,
    manager: Arc<DiscordManager>,
}

impl MusicWatcher {
    pub fn new(manager: Arc<DiscordManager>) -> Self {
        Self {
            mode: Arc::new(parking_lot::RwLock::new(MusicMode::Off)),
            last_cover: Arc::new(parking_lot::RwLock::new(None)),
            last_snapshot: Arc::new(parking_lot::RwLock::new(None)),
            manager,
        }
    }

    pub fn set_mode(&self, mode: MusicMode) {
        *self.mode.write() = mode;
    }

    pub fn mode(&self) -> MusicMode {
        self.mode.read().clone()
    }

    pub fn snapshot(&self) -> Option<MediaSnapshot> {
        self.last_snapshot.read().clone()
    }

    pub fn cover_data_url(&self) -> Option<String> {
        self.last_cover.read().clone()
    }

    /// Разовый опрос SMTC (для кнопки «Обновить» в UI).
    pub async fn poll_once(&self) -> MediaSnapshot {
        let snap = super::smtc::snapshot_async().await;
        self.cache_snapshot(&snap);
        snap
    }

    fn cache_snapshot(&self, snap: &MediaSnapshot) {
        if let Some(bytes) = &snap.thumbnail {
            *self.last_cover.write() = Some(format!(
                "data:image/png;base64,{}",
                base64::Engine::encode(&base64::engine::general_purpose::STANDARD, bytes)
            ));
        }
        *self.last_snapshot.write() = Some(snap.clone());
    }

    /// Запускает цикл слежения (spawn на старте приложения).
    pub fn spawn(self: &Arc<Self>, _app: AppHandle) {
        let watcher = Arc::clone(self);
        tauri::async_runtime::spawn(async move {
            let mut tick = tokio::time::interval(Duration::from_secs(2));
            let mut last_signature = String::new();
            let mut last_push: Option<std::time::Instant> = None;
            // Позиция на момент последней публикации — чтобы ловить seek и паузу.
            let mut published_pos = 0.0f64;
            let mut published_playing = false;
            tick.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Delay);
            loop {
                tick.tick().await;
                if watcher.mode() != MusicMode::Auto {
                    continue;
                }
                let snap = super::smtc::snapshot_async().await;
                watcher.cache_snapshot(&snap);

                // Нечего показывать — пропускаем.
                if !snap.available {
                    continue;
                }

                // Подпись трека: меняем активность только при смене трека/статуса.
                let signature = format!(
                    "{}|{}|{}|{}",
                    snap.title.clone().unwrap_or_default(),
                    snap.artist.clone().unwrap_or_default(),
                    snap.playing,
                    snap.duration_secs.map(|d| d as u64).unwrap_or(0),
                );
                let now = std::time::Instant::now();
                let mut republish = signature != last_signature;

                // Позиция в Discord считается от start-timestamp, поэтому
                // переопубликовываем активность только когда реальная позиция
                // разошлась с ожидаемой (seek, перемотка, пауза/плей).
                if !republish {
                    if let Some(at) = last_push {
                        if snap.playing != published_playing {
                            republish = true;
                        } else if snap.playing {
                            let expected = published_pos + at.elapsed().as_secs_f64();
                            let actual = snap.position_secs.unwrap_or(expected);
                            if (expected - actual).abs() > 2.0 {
                                republish = true;
                            }
                        }
                        // Страховка от накопленного дрейфа раз в 10 минут.
                        if at.elapsed() > Duration::from_secs(600) {
                            republish = true;
                        }
                    } else {
                        republish = true;
                    }
                }

                if republish {
                    last_signature = signature;
                    if let Some(activity) = build_music_activity(&snap).await {
                        if watcher.manager.set_activity(activity).await.is_ok() {
                            last_push = Some(now);
                            published_pos = snap.position_secs.unwrap_or(0.0);
                            published_playing = snap.playing;
                        }
                    }
                }
            }
        });
    }
}

async fn build_music_activity(snap: &MediaSnapshot) -> Option<Activity> {
    let title = snap.title.clone()?;
    let now = crate::discord::rpc::now_epoch();

    let mut act = Activity {
        name: None,
        details: Some(title),
        state: snap.artist.clone().or_else(|| Some("Неизвестный исполнитель".into())),
        activity_type: ActivityType::Listening,
        timestamps_start: None,
        timestamps_end: None,
        large_image: None,
        large_text: snap.album.clone(),
        small_image: None,
        small_text: snap.source.clone(),
        buttons: vec![],
    };

    // Прогресс: start = now - позиция, end = start + длительность.
    // На паузе таймеры не ставим: Discord всегда «тикает» от start, заморозить
    // прогресс протоколом нельзя — честнее показать трек без времени.
    if snap.playing {
        if let Some(pos) = snap.position_secs {
            let start = now - pos.round() as i64;
            act.timestamps_start = Some(start);
            if let Some(dur) = snap.duration_secs {
                if dur > pos {
                    act.timestamps_end = Some(start + dur.round() as i64);
                }
            }
        } else {
            act.timestamps_start = Some(now);
        }
    }

    // Обложка: байты SMTC → публичный URL (catbox.moe, кэш по треку).
    if let Some(bytes) = &snap.thumbnail {
        let sig = format!("{}|{}", snap.title.clone().unwrap_or_default(), snap.artist.clone().unwrap_or_default());
        if let Ok(Some(url)) = tokio::time::timeout(
            Duration::from_secs(25),
            super::cover::cover_url(&sig, bytes),
        )
        .await
        {
            act.large_image = Some(url);
        }
    }

    // Кнопка «Слушать» — поиск трека на SoundCloud (работает для любого плеера).
    if let Some(title) = &act.details {
        if let Some(artist) = &act.state {
            let q = format!("{title} {artist}");
            let url = format!("https://soundcloud.com/search?q={}", urlencode(&q));
            act.buttons.push(crate::presence::ActivityButton {
                label: "Слушать".into(),
                url,
            });
        }
    }
    Some(act)
}

/// Минимальный percent-encoding для query-параметра.
fn urlencode(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for b in s.as_bytes() {
        match b {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                out.push(*b as char)
            }
            b' ' => out.push('+'),
            _ => out.push_str(&format!("%{b:02X}")),
        }
    }
    out
}
