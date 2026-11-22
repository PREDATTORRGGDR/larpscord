//! Discord RPC-клиент: рукопожатие, SET_ACTIVITY / CLEAR_ACTIVITY,
//! keepalive (PING/PONG) и чтение ответов по nonce.

use super::ipc::{IpcConnection, IpcError, Op};
use crate::presence::Activity;
use serde_json::{json, Value};
use std::time::Duration;
use tokio::sync::Mutex;
use tokio::time::sleep;

/// Пользователь Discord из ответа READY.
#[derive(Debug, Clone, serde::Serialize)]
pub struct UserInfo {
    pub id: String,
    pub username: String,
    #[serde(rename = "globalName")]
    pub global_name: Option<String>,
}

/// Сводное состояние подключения (передаётся во фронтенд).
#[derive(Debug, Clone, Default, serde::Serialize)]
pub struct ConnectionState {
    pub connected: bool,
    pub flavor: String,
    pub user: Option<UserInfo>,
    pub last_error: Option<String>,
    #[serde(rename = "lastActivitySentAt")]
    pub last_activity_sent_at: Option<i64>,
    #[serde(rename = "hasPendingActivity")]
    pub has_pending_activity: bool,
}

/// Минимальный wire-уровень: одно соединение + обмен командами.
pub struct RpcClient {
    conn: Mutex<Option<IpcConnection>>,
    state: Mutex<ConnectionState>,
    /// Последняя успешно отправленная активность (для повтора после реконнекта).
    last_activity: Mutex<Option<Activity>>,
}

impl RpcClient {
    pub fn new() -> Self {
        Self {
            conn: Mutex::new(None),
            state: Mutex::new(ConnectionState::default()),
            last_activity: Mutex::new(None),
        }
    }

    /// Подключается к пайпу и выполняет рукопожатие с данным client_id.
    pub async fn connect(&self, client_id: &str) -> Result<UserInfo, IpcError> {
        let mut conn = IpcConnection::connect().await?;
        let ready = conn.handshake(client_id).await?;
        let user = parse_ready_user(&ready);
        let flavor = conn.flavor();
        {
            let mut st = self.state.lock().await;
            st.connected = true;
            st.flavor = flavor.as_str().to_string();
            st.last_error = None;
        }
        *self.conn.lock().await = Some(conn);
        Ok(user.unwrap_or(UserInfo {
            id: String::new(),
            username: "unknown".into(),
            global_name: None,
        }))
    }

    /// Закрывает текущее соединение (без попыток переподключения).
    pub async fn disconnect(&self) {
        *self.conn.lock().await = None;
        let mut st = self.state.lock().await;
        st.connected = false;
        st.user = None;
    }

    pub async fn is_connected(&self) -> bool {
        self.conn.lock().await.is_some()
    }

    pub async fn state(&self) -> ConnectionState {
        let mut st = self.state.lock().await.clone();
        st.has_pending_activity = self.last_activity.lock().await.is_some();
        st
    }

    /// Отмечает потерю соединения (вызывается менеджером при ошибке).
    pub async fn mark_disconnected(&self, reason: String) {
        let mut st = self.state.lock().await;
        st.connected = false;
        st.last_error = Some(reason);
    }

    /// Отправляет команду и ждёт кадр-ответ с совпадающим nonce.
    async fn command(&self, cmd: &str, args: Value) -> Result<Value, IpcError> {
        let mut guard = self.conn.lock().await;
        let conn = guard.as_mut().ok_or(IpcError::NoPipe)?;
        let nonce = uuid::Uuid::new_v4().to_string();
        let payload = json!({ "cmd": cmd, "args": args, "nonce": nonce });
        conn.write_frame(Op::Frame, &payload).await?;

        // Ждём ответ: пропускаем чужие фреймы (DISPATCH-события и т.п.).
        loop {
            let (op, data) = conn.read_frame_timeout(5).await?;
            match op {
                Op::Close => {
                    return Err(IpcError::Closed);
                }
                Op::Pong => continue, // ответ на keepalive — не для нас
                Op::Frame => {
                    let same_nonce = data.get("nonce").and_then(Value::as_str) == Some(&nonce);
                    let is_error = data.get("evt").and_then(Value::as_str) == Some("ERROR");
                    if same_nonce && is_error {
                        let msg = data
                            .pointer("/data/message")
                            .and_then(Value::as_str)
                            .unwrap_or("неизвестная ошибка Discord");
                        let code = data
                            .pointer("/data/code")
                            .and_then(Value::as_i64)
                            .map(|c| format!(" (код {c})"))
                            .unwrap_or_default();
                        return Err(IpcError::Rpc(format!("{msg}{code}")));
                    }
                    if same_nonce {
                        return Ok(data);
                    }
                    // чужой nonce — игнорируем
                }
                other => {
                    return Err(IpcError::Protocol(format!("неожиданный opcode {other:?}")));
                }
            }
        }
    }

    /// Публикует активность; сохраняет её для повтора после реконнекта.
    pub async fn set_activity(&self, pid: u32, activity: &Activity) -> Result<(), IpcError> {
        let args = json!({ "pid": pid, "activity": activity_to_rpc(activity) });
        self.command("SET_ACTIVITY", args).await?;
        *self.last_activity.lock().await = Some(activity.clone());
        self.state.lock().await.last_activity_sent_at = Some(now_epoch());
        Ok(())
    }

    /// Очищает активность.
    ///
    /// ВАЖНО: команды CLEAR_ACTIVITY в протоколе нет (проверено эмпирически —
    /// Discord отвечает «Invalid command: CLEAR_ACTIVITY»). Очистка
    /// выполняется через SET_ACTIVITY с `"activity": null`.
    pub async fn clear_activity(&self, pid: u32) -> Result<(), IpcError> {
        self.command("SET_ACTIVITY", json!({ "pid": pid, "activity": null }))
            .await?;
        *self.last_activity.lock().await = None;
        self.state.lock().await.last_activity_sent_at = None;
        Ok(())
    }

    /// Тихо повторно публикует последнюю активность (после реконнекта).
    /// Возвращает true, если была что повторять.
    pub async fn replay_last_activity(&self, pid: u32) -> Result<bool, IpcError> {
        let last = self.last_activity.lock().await.clone();
        match last {
            Some(a) => {
                self.set_activity(pid, &a).await?;
                Ok(true)
            }
            None => Ok(false),
        }
    }

    /// Проверка живости соединения: PING → ждём PONG.
    pub async fn keepalive(&self) -> Result<(), IpcError> {
        let mut guard = self.conn.lock().await;
        let conn = guard.as_mut().ok_or(IpcError::NoPipe)?;
        conn.write_frame(Op::Ping, &json!({})).await?;
        let deadline = sleep(Duration::from_secs(5));
        tokio::select! {
            _ = deadline => Err(IpcError::Timeout),
            frame = conn.read_frame() => {
                let (op, _data) = frame?;
                match op {
                    Op::Pong => Ok(()),
                    Op::Close => Err(IpcError::Closed),
                    // Иногда вместо PONG может прийти другой фрейм — считаем живым,
                    // раз пайп отвечает.
                    _ => Ok(()),
                }
            }
        }
    }
}

fn parse_ready_user(ready: &Value) -> Option<UserInfo> {
    let user = ready.pointer("/data/user")?;
    Some(UserInfo {
        id: user.get("id")?.as_str()?.to_string(),
        username: user
            .get("username")
            .and_then(Value::as_str)
            .unwrap_or_default()
            .to_string(),
        global_name: user
            .get("global_name")
            .or_else(|| user.get("username"))
            .and_then(Value::as_str)
            .map(str::to_string),
    })
}

/// Текущее время в секундах от эпохи Unix.
pub fn now_epoch() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0)
}

/// Преобразует модель активности Larpscord в JSON для SET_ACTIVITY.
///
/// Пустые поля не отправляются: Discord удаляет только те поля, что пришли
/// в последнем SET_ACTIVITY, поэтому отсутствие поля = очистка.
pub fn activity_to_rpc(a: &Activity) -> Value {
    let mut act = serde_json::Map::new();
    act.insert("type".into(), json!(a.activity_type as u8));
    if let Some(name) = &a.name {
        if !name.trim().is_empty() {
            act.insert("name".into(), json!(name));
        }
    }
    if let Some(d) = &a.details {
        if !d.trim().is_empty() {
            act.insert("details".into(), json!(d));
        }
    }
    if let Some(s) = &a.state {
        if !s.trim().is_empty() {
            act.insert("state".into(), json!(s));
        }
    }
    let mut ts = serde_json::Map::new();
    if let Some(start) = a.timestamps_start {
        ts.insert("start".into(), json!(start));
    }
    if let Some(end) = a.timestamps_end {
        ts.insert("end".into(), json!(end));
    }
    if !ts.is_empty() {
        act.insert("timestamps".into(), Value::Object(ts));
    }
    let mut assets = serde_json::Map::new();
    if let Some(li) = &a.large_image {
        if !li.trim().is_empty() {
            assets.insert("large_image".into(), json!(li));
        }
    }
    if let Some(lt) = &a.large_text {
        if !lt.trim().is_empty() {
            assets.insert("large_text".into(), json!(lt));
        }
    }
    if let Some(si) = &a.small_image {
        if !si.trim().is_empty() {
            assets.insert("small_image".into(), json!(si));
        }
    }
    if let Some(st) = &a.small_text {
        if !st.trim().is_empty() {
            assets.insert("small_text".into(), json!(st));
        }
    }
    if !assets.is_empty() {
        act.insert("assets".into(), Value::Object(assets));
    }
    let buttons: Vec<Value> = a
        .buttons
        .iter()
        .filter(|b| !b.label.trim().is_empty() && b.is_url_valid())
        .map(|b| json!({ "label": b.label.trim(), "url": b.url.trim() }))
        .collect();
    if !buttons.is_empty() {
        act.insert("buttons".into(), json!(buttons));
    }
    Value::Object(act)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::presence::ActivityButton;

    fn sample() -> Activity {
        Activity {
            name: Some("Larpscord".into()),
            details: Some("Настраиваю присутствие".into()),
            state: Some("Black Liquid Glass".into()),
            activity_type: crate::presence::ActivityType::Playing,
            timestamps_start: Some(1700000000),
            timestamps_end: None,
            large_image: Some("https://example.com/a.png".into()),
            large_text: Some("Подсказка".into()),
            small_image: None,
            small_text: None,
            buttons: vec![ActivityButton {
                label: "Сайт".into(),
                url: "https://example.com".into(),
            }],
        }
    }

    #[test]
    fn activity_serializes_fields() {
        let v = activity_to_rpc(&sample());
        assert_eq!(v["type"], 0);
        assert_eq!(v["details"], "Настраиваю присутствие");
        assert_eq!(v["timestamps"]["start"], 1700000000);
        assert_eq!(v["assets"]["large_image"], "https://example.com/a.png");
        assert_eq!(v["buttons"][0]["label"], "Сайт");
    }

    #[test]
    fn empty_fields_omitted() {
        let mut a = sample();
        a.details = Some("   ".into());
        a.buttons.clear();
        let v = activity_to_rpc(&a);
        assert!(v.get("details").is_none());
        assert!(v.get("buttons").is_none());
    }
}
