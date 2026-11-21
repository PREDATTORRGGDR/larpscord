//! Живой интеграционный тест: реальное подключение к Discord и
//! публикация активности через библиотеку Larpscord.
//!
//! Запуск (только вручную, нужен запущенный Discord):
//!   cargo test --test live_rpc -- --ignored --nocapture

use larpscord_lib::discord::ipc::IpcConnection;
use larpscord_lib::discord::rpc::RpcClient;
use larpscord_lib::presence::{Activity, ActivityButton, ActivityType};

#[tokio::test]
#[ignore]
async fn live_handshake_and_activity() {
    // 1. Подключение к живому Discord.
    let client = RpcClient::new();
    let user = client
        .connect(larpscord_lib::discord::manager::BUILTIN_CLIENT_ID)
        .await
        .expect("Discord не найден — тест требует запущенный Discord");
    println!("READY: пользователь {}", user.username);
    assert!(!user.username.is_empty());

    // 2. Публикация активности.
    let activity = Activity {
        name: Some("Larpscord".into()),
        details: Some("Интеграционный тест".into()),
        state: Some("live_rpc.rs".into()),
        activity_type: ActivityType::Playing,
        timestamps_start: Some(larpscord_lib::discord::rpc::now_epoch()),
        timestamps_end: None,
        large_image: Some("https://picsum.photos/512".into()),
        large_text: Some("Тестовая обложка".into()),
        small_image: None,
        small_text: None,
        buttons: vec![ActivityButton {
            label: "GitHub".into(),
            url: "https://github.com".into(),
        }],
    };
    activity.validate().expect("валидация");
    client
        .set_activity(std::process::id(), &activity)
        .await
        .expect("SET_ACTIVITY не прошёл");
    println!("SET_ACTIVITY: OK");

    // 3. Соединение живо.
    assert!(client.is_connected().await);

    // 4. Очистка.
    client
        .clear_activity(std::process::id())
        .await
        .expect("очистка не прошла");
    println!("CLEAR: OK");
}

#[tokio::test]
#[ignore]
async fn live_keepalive() {
    let mut conn = IpcConnection::connect().await.expect("нет пайпа");
    let ready = conn
        .handshake(larpscord_lib::discord::manager::BUILTIN_CLIENT_ID)
        .await
        .expect("рукопожатие");
    assert_eq!(ready.get("evt").and_then(|v| v.as_str()), Some("READY"));
    conn.write_frame(larpscord_lib::discord::ipc::Op::Ping, &serde_json::json!({}))
        .await
        .unwrap();
    let (op, _) = conn.read_frame_timeout(5).await.unwrap();
    assert_eq!(op, larpscord_lib::discord::ipc::Op::Pong);
    println!("PING/PONG: OK");
}
