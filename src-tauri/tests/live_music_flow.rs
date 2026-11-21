//! Живой тест полного музыкального потока: SMTC → активность → Discord.
//! cargo test --test live_music_flow -- --ignored --nocapture

use larpscord_lib::music::smtc::snapshot_async;
use larpscord_lib::presence::{Activity, ActivityType};

#[tokio::test]
#[ignore]
async fn live_music_to_discord() {
    // 1. Снимок SMTC.
    let snap = snapshot_async().await;
    println!("SMTC: {} — {} ({})", snap.title.clone().unwrap_or_default(), snap.artist.clone().unwrap_or_default(), snap.source.clone().unwrap_or_default());
    if !snap.available {
        println!("Плеер не активен — частичная проверка (только соединение)");
    }

    // 2. Активность как её строит watcher.
    let now = larpscord_lib::discord::rpc::now_epoch();
    let act = Activity {
        name: None,
        details: snap.title.clone().or(Some("Нет трека".into())),
        state: snap.artist.clone(),
        activity_type: ActivityType::Listening,
        timestamps_start: snap.position_secs.map(|p| now - p as i64).or(Some(now)),
        timestamps_end: None,
        large_image: None,
        large_text: snap.album.clone(),
        small_image: None,
        small_text: snap.source.clone(),
        buttons: vec![],
    };
    act.validate().expect("валидация");

    // 3. Публикация в живой Discord.
    let client = larpscord_lib::discord::rpc::RpcClient::new();
    client
        .connect(larpscord_lib::discord::manager::BUILTIN_CLIENT_ID)
        .await
        .expect("Discord не найден");
    client.set_activity(std::process::id(), &act).await.expect("SET_ACTIVITY");
    println!("Музыкальная активность опубликована в Discord");

    // 4. Очистка.
    tokio::time::sleep(std::time::Duration::from_secs(2)).await;
    client.clear_activity(std::process::id()).await.expect("CLEAR");
    println!("Очищено");
}
