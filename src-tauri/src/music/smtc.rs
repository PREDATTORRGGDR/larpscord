//! Чтение Windows Media Session (SMTC): трек, исполнитель, альбом,
//! обложка, позиция и статус воспроизведения из любых плееров
//! (Spotify, браузеры, локальные плееры).

use serde::Serialize;

/// Снимок текущего медиа.
#[derive(Debug, Clone, Serialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct MediaSnapshot {
    pub available: bool,
    /// Имя источника (Spotify, Chrome и т.п.).
    pub source: Option<String>,
    pub title: Option<String>,
    pub artist: Option<String>,
    pub album: Option<String>,
    /// Обложка: PNG/JPEG байты, если плеер их отдаёт.
    pub thumbnail: Option<Vec<u8>>,
    /// Играет / пауза / остановлено.
    pub playing: bool,
    /// Позиция в секундах.
    pub position_secs: Option<f64>,
    /// Длительность в секундах.
    pub duration_secs: Option<f64>,
}

#[cfg(windows)]
pub mod win {
    use super::MediaSnapshot;
    use windows::core::HSTRING;
    use windows::Media::Control::GlobalSystemMediaTransportControlsSessionManager as SessionManager;
    use windows::Media::Control::GlobalSystemMediaTransportControlsSessionPlaybackStatus as PlaybackStatus;
    use windows::Storage::Streams::DataReader;

    /// Собирает снимок текущей медиа-сессии.
    /// Блокирующая: вызывать через `tauri::async_runtime::spawn_blocking`.
    pub async fn snapshot() -> windows::core::Result<MediaSnapshot> {
        let manager = SessionManager::RequestAsync()?.await?;

        // Берём сессию, которая реально играет: GetCurrentSession может указать
        // на свёрнутый/приостановленный плеер, пока музыка идёт в другом.
        let mut session = None;
        if let Ok(list) = manager.GetSessions() {
            for i in 0..list.Size().unwrap_or(0) {
                if let Ok(s) = list.GetAt(i) {
                    let playing = s
                        .GetPlaybackInfo()
                        .ok()
                        .and_then(|pi| pi.PlaybackStatus().ok())
                        .map(|st| st == PlaybackStatus::Playing)
                        .unwrap_or(false);
                    if playing {
                        session = Some(s);
                        break;
                    }
                }
            }
        }
        let session = match session.or_else(|| manager.GetCurrentSession().ok()) {
            Some(s) => s,
            None => return Ok(MediaSnapshot::default()),
        };
        // Пустая сессия (нет активного плеера): SourceAppUserModelId падает.
        if session.SourceAppUserModelId().is_err() {
            return Ok(MediaSnapshot::default());
        }

        let aumid = session.SourceAppUserModelId().unwrap_or_default().to_string();
        let props = session.TryGetMediaPropertiesAsync()?.await?;
        let title = hstr(&props.Title());
        let artist = hstr(&props.Artist());
        let album = hstr(&props.AlbumTitle());

        let thumbnail = read_thumbnail(&props).await;

        let playing = session
            .GetPlaybackInfo()
            .ok()
            .and_then(|pi| pi.PlaybackStatus().ok())
            .map(|st| {
                st == windows::Media::Control::GlobalSystemMediaTransportControlsSessionPlaybackStatus::Playing
            })
            .unwrap_or(false);

        let (position_secs, duration_secs) = session
            .GetTimelineProperties()
            .ok()
            .map(|tl| timeline(&tl, playing))
            .unwrap_or((None, None));

        Ok(MediaSnapshot {
            available: title.is_some() || artist.is_some(),
            source: Some(friendly_source(&aumid)),
            title,
            artist,
            album,
            thumbnail,
            playing,
            position_secs,
            duration_secs,
        })
    }

    /// Позиция/длительность трека с поправкой на «возраст» снимка таймлайна.
    ///
    /// SMTC отдаёт `Position` на момент `LastUpdatedTime`, а не «сейчас»:
    /// плееры обновляют таймлайн только при seek/pause/смене трека. Пока
    /// играет — добавляем прошедшее с этого момента время, иначе позиция
    /// «замирает» и Discord показывает случайные секунды.
    fn timeline(
        tl: &windows::Media::Control::GlobalSystemMediaTransportControlsSessionTimelineProperties,
        playing: bool,
    ) -> (Option<f64>, Option<f64>) {
        let secs = |d: windows::Foundation::TimeSpan| d.Duration as f64 / 10_000_000.0;
        let start = tl.StartTime().map(secs).unwrap_or(0.0);
        let end = tl.EndTime().map(secs).unwrap_or(0.0);
        let mut pos = tl.Position().map(secs).unwrap_or(0.0) - start;

        if playing {
            if let Ok(updated) = tl.LastUpdatedTime() {
                if let Some(dt) = seconds_since(updated) {
                    // Разумный коридор: игнорируем мусорные/древние метки.
                    if (0.0..7200.0).contains(&dt) {
                        pos += dt;
                    }
                }
            }
        }

        let duration = end - start;
        if duration > 0.0 {
            pos = pos.clamp(0.0, duration);
            (Some(pos.max(0.0)), Some(duration))
        } else {
            (if pos > 0.0 { Some(pos) } else { None }, None)
        }
    }

    /// Секунды с момента WinRT `DateTime` (100-нс тики с 1601 года).
    fn seconds_since(when: windows::Foundation::DateTime) -> Option<f64> {
        const EPOCH_DIFF_100NS: i64 = 116_444_736_000_000_000;
        if when.UniversalTime <= EPOCH_DIFF_100NS {
            return None;
        }
        let then_ns = (when.UniversalTime - EPOCH_DIFF_100NS) as f64 * 100.0;
        let now_ns = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .ok()?
            .as_nanos() as f64;
        let delta = (now_ns - then_ns) / 1e9;
        delta.is_finite().then_some(delta)
    }

    async fn read_thumbnail(
        props: &windows::Media::Control::GlobalSystemMediaTransportControlsSessionMediaProperties,
    ) -> Option<Vec<u8>> {
        let stream_ref = props.Thumbnail().ok()?;
        let stream = stream_ref.OpenReadAsync().ok()?.await.ok()?;
        let size = stream.Size().ok()? as u32;
        if size == 0 || size > 8 * 1024 * 1024 {
            return None;
        }
        let reader = DataReader::CreateDataReader(&stream).ok()?;
        reader.LoadAsync(size).ok()?.await.ok()?;
        let mut buf = vec![0u8; size as usize];
        reader.ReadBytes(&mut buf).ok()?;
        Some(buf)
    }

    fn hstr(h: &windows::core::Result<HSTRING>) -> Option<String> {
        match h {
            Ok(s) => {
                let t = s.to_string();
                if t.trim().is_empty() { None } else { Some(t) }
            }
            Err(_) => None,
        }
    }

    /// AUMID → человекочитаемое имя.
    fn friendly_source(aumid: &str) -> String {
        let lower = aumid.to_lowercase();
        let known = [
            ("spotify", "Spotify"),
            ("chrome", "Chrome"),
            ("firefox", "Firefox"),
            ("msedge", "Edge"),
            ("yandex", "Yandex Music"),
            ("vlc", "VLC"),
            ("winamp", "Winamp"),
            ("foobar2000", "foobar2000"),
            ("mediaplayer", "Медиаплеер"),
        ];
        for (pat, name) in known {
            if lower.contains(pat) {
                return name.into();
            }
        }
        aumid.split('!').next().unwrap_or(aumid).to_string()
    }
}

#[cfg(windows)]
pub use win::snapshot;

/// Асинхронная обёртка: SMTC-вызовы выполняются в блокирующем пуле
/// (WinRT-объекты не Send, COM-апартамент).
pub async fn snapshot_async() -> MediaSnapshot {
    #[cfg(windows)]
    {
        tauri::async_runtime::spawn_blocking(|| {
            // Выделенный поток: создаём однопоточный runtime и блокируемся.
            tokio::runtime::Builder::new_current_thread()
                .build()
                .expect("runtime")
                .block_on(async { win::snapshot().await })
        })
        .await
        .ok()
        .and_then(|r| r.ok())
        .unwrap_or_default()
    }
    #[cfg(not(windows))]
    {
        MediaSnapshot::default()
    }
}
