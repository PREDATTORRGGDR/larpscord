//! Загрузка обложек треков на публичный хостинг (catbox.moe),
//! чтобы Discord мог показать картинку по URL.
//!
//! Windows Media Session отдаёт обложку байтами, а Discord RPC принимает
//! только http(s)-ссылки — поэтому кэшируем байты и грузим на хостинг.

use std::collections::HashMap;
use std::sync::Mutex;
use std::time::{Duration, Instant};

/// Кэш: подпись трека → URL обложки (чтобы не грузить повторно).
static COVER_CACHE: Mutex<Option<CoverCache>> = Mutex::new(None);

struct CoverCache {
    map: HashMap<String, (String, Instant)>,
}

fn cache() -> &'static Mutex<Option<CoverCache>> {
    &COVER_CACHE
}

/// Возвращает URL обложки для трека: из кэша или загружает.
/// При ошибке — None (активность уйдёт без обложки, честно).
pub async fn cover_url(signature: &str, bytes: &[u8]) -> Option<String> {
    if bytes.is_empty() {
        return None;
    }
    // Кэш на 6 часов.
    {
        let mut guard = cache().lock().ok()?;
        if let Some(c) = guard.as_mut() {
            if let Some((url, at)) = c.map.get(signature) {
                if at.elapsed() < Duration::from_secs(6 * 3600) {
                    return Some(url.clone());
                }
                c.map.remove(signature);
            }
        }
    }

    // Загрузка на catbox.moe (multipart/form-data, поле reqtype=fileupload).
    let part = reqwest::multipart::Part::bytes(bytes.to_vec())
        .file_name("cover.jpg")
        .mime_str("image/jpeg")
        .ok()?;
    let form = reqwest::multipart::Form::new()
        .text("reqtype", "fileupload")
        .part("fileToUpload", part);

    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(20))
        .build()
        .ok()?;
    let resp = client
        .post("https://catbox.moe/user/api.php")
        .multipart(form)
        .send()
        .await
        .ok()?;
    if !resp.status().is_success() {
        return None;
    }
    let url = resp.text().await.ok()?;
    let url = url.trim().to_string();
    if !url.starts_with("https://") {
        return None;
    }

    // В кэш.
    if let Ok(mut guard) = cache().lock() {
        let c = guard.get_or_insert_with(|| CoverCache {
            map: HashMap::new(),
        });
        // Не раздуваем кэш.
        if c.map.len() > 64 {
            c.map.clear();
        }
        c.map
            .insert(signature.to_string(), (url.clone(), Instant::now()));
    }
    Some(url)
}
