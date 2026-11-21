//! Загрузка обложек треков на публичные хостинги, чтобы Discord мог
//! показать картинку по http(s)-ссылке.
//!
//! Windows Media Session отдаёт обложку байтами, а Discord RPC принимает
//! только URL. Пробуем хостинги по цепочке (доступность зависит от сети):
//! 1. uguu.se  — проверен, работает; файлы живут ~3 часа (для обложек ок)
//! 2. tmpfiles.org — запасной (отдаёт ссылку через /dl/)

use std::collections::HashMap;
use std::sync::Mutex;
use std::time::{Duration, Instant};

/// Кэш: подпись трека → URL обложки.
static COVER_CACHE: Mutex<Option<CoverCache>> = Mutex::new(None);

struct CoverCache {
    map: HashMap<String, (String, Instant)>,
}

/// Возвращает URL обложки для трека: из кэша или загружает.
/// При недоступности всех хостингов — None (активность без обложки).
pub async fn cover_url(signature: &str, bytes: &[u8]) -> Option<String> {
    if bytes.is_empty() {
        return None;
    }
    // Кэш 2 часа (uguu хранит ~3 часа — берём запас).
    {
        let mut guard = cache().lock().ok()?;
        if let Some(c) = guard.as_mut() {
            if let Some((url, at)) = c.map.get(signature) {
                if at.elapsed() < Duration::from_secs(2 * 3600) {
                    return Some(url.clone());
                }
                c.map.remove(signature);
            }
        }
    }

    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(25))
        .user_agent("Larpscord/1.2")
        .build()
        .ok()?;

    // Определяем mime по магическим байтам (SMTC отдаёт PNG или JPEG).
    let (mime, ext) = if bytes.starts_with(&[0x89, b'P', b'N', b'G']) {
        ("image/png", "png")
    } else {
        ("image/jpeg", "jpg")
    };

    // 1. uguu.se (с одним ретраем — иногда отвечает 502)
    for _ in 0..2 {
        if let Some(url) = upload_uguu(&client, bytes, mime, ext).await {
            remember(signature, url.clone());
            return Some(url);
        }
    }
    // 2. tmpfiles.org
    if let Some(url) = upload_tmpfiles(&client, bytes, mime, ext).await {
        remember(signature, url.clone());
        return Some(url);
    }
    None
}

fn remember(signature: &str, url: String) {
    if let Ok(mut guard) = cache().lock() {
        let c = guard.get_or_insert_with(|| CoverCache {
            map: HashMap::new(),
        });
        if c.map.len() > 64 {
            c.map.clear();
        }
        c.map
            .insert(signature.to_string(), (url, Instant::now()));
    }
}

fn cache() -> &'static Mutex<Option<CoverCache>> {
    &COVER_CACHE
}

fn part(_name: &str, filename: &str, mime: &str, bytes: &[u8]) -> reqwest::multipart::Part {
    reqwest::multipart::Part::bytes(bytes.to_vec())
        .file_name(filename.to_string())
        .mime_str(mime)
        .unwrap()
}

/// uguu.se: POST /upload, поле files[] → JSON {"success":true,"files":[{"url":...}]}
async fn upload_uguu(client: &reqwest::Client, bytes: &[u8], mime: &str, ext: &str) -> Option<String> {
    let form = reqwest::multipart::Form::new().part(
        "files[]",
        part("files[]", &format!("cover.{ext}"), mime, bytes),
    );
    let resp = client
        .post("https://uguu.se/upload")
        .multipart(form)
        .send()
        .await
        .ok()?;
    if !resp.status().is_success() {
        return None;
    }
    let json: serde_json::Value = resp.json().await.ok()?;
    let url = json
        .pointer("/files/0/url")
        .and_then(|v| v.as_str())?
        .to_string();
    if url.starts_with("https://") {
        Some(url)
    } else {
        None
    }
}

/// tmpfiles.org: POST /api/v1/upload, поле file → JSON {"data":{"url":...}}
async fn upload_tmpfiles(
    client: &reqwest::Client,
    bytes: &[u8],
    mime: &str,
    ext: &str,
) -> Option<String> {
    let form = reqwest::multipart::Form::new().part(
        "file",
        part("file", &format!("cover.{ext}"), mime, bytes),
    );
    let resp = client
        .post("https://tmpfiles.org/api/v1/upload")
        .multipart(form)
        .send()
        .await
        .ok()?;
    if !resp.status().is_success() {
        return None;
    }
    let json: serde_json::Value = resp.json().await.ok()?;
    // tmpfiles отдаёт страницу вида /12345/file.png → нужен dl-домен.
    let page = json.pointer("/data/url").and_then(|v| v.as_str())?;
    let dl = page.replacen("tmpfiles.org/", "tmpfiles.org/dl/", 1);
    if dl.starts_with("https://") {
        Some(dl)
    } else {
        None
    }
}
