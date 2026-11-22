//! Менеджер Discord-соединения: автоподключение при старте, keepalive,
//! автоматическое переподключение после перезапуска Discord и рассылка
//! событий во фронтенд.

use super::ipc::{DiscordFlavor, IpcError};
use super::rpc::{ConnectionState, RpcClient, UserInfo};
use crate::presence::Activity;
use std::sync::Arc;
use std::time::Duration;
use tauri::{AppHandle, Emitter};
use tokio::sync::mpsc;

/// Встроенный Application ID (реально существующее приложение Discord,
//  проверено живым рукопожатием; заимствовано из открытого проекта
//  patryk-ku/music-discord-rpc, MIT — см. docs/RESEARCH.md).
/// Пользователь может указать свой ID в настройках.
pub const BUILTIN_CLIENT_ID: &str = "1129859263741837373";

/// Текущий выбранный client_id (встроенный или пользовательский).
#[derive(Debug, Clone, serde::Serialize)]
pub struct ClientIdConfig {
    pub value: String,
    pub is_custom: bool,
}

/// Событие, отправляемое в окно (фронтенд слушает "discord://status").
#[derive(Debug, Clone, serde::Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum StatusEvent {
    Connecting { attempt: u32 },
    Connected { flavor: String, user: Option<UserInfo> },
    Disconnected { reason: String },
    Reconnected { flavor: String, user: Option<UserInfo> },
}

pub struct DiscordManager {
    client: Arc<RpcClient>,
    /// Команды для рабочего цикла.
    tx: mpsc::Sender<ManagerCommand>,
    /// Приёмник команд; забирается рабочим циклом при spawn().
    rx: parking_lot::Mutex<Option<mpsc::Receiver<ManagerCommand>>>,
    client_id: Arc<parking_lot::RwLock<ClientIdConfig>>,
    state: Arc<parking_lot::RwLock<ManagerState>>,
}

enum ManagerCommand {
    SetActivity(Activity),
    ClearActivity,
    Stop,
}

#[derive(Default)]
struct ManagerState {
    /// Активность, которую нужно (пере)применить.
    pending: Option<Activity>,
    /// Активность применена в Discord.
    live: Option<Activity>,
    running: bool,
}

impl DiscordManager {
    pub fn new() -> Self {
        let (tx, rx) = mpsc::channel(32);
        Self {
            client: Arc::new(RpcClient::new()),
            tx,
            rx: parking_lot::Mutex::new(Some(rx)),
            client_id: Arc::new(parking_lot::RwLock::new(ClientIdConfig {
                value: BUILTIN_CLIENT_ID.to_string(),
                is_custom: false,
            })),
            state: Arc::new(parking_lot::RwLock::new(ManagerState::default())),
        }
    }

    /// Запускает фоновый цикл подключения/поддержки соединения.
    /// Вызывается один раз при старте приложения.
    pub fn spawn(self: &Arc<Self>, app: AppHandle) {
        let manager = Arc::clone(self);
        let rx = manager
            .rx
            .lock()
            .take()
            .expect("spawn() вызван повторно");
        tauri::async_runtime::spawn(async move {
            manager.run_loop(app, rx).await;
        });
    }

    pub fn set_client_id(&self, value: String) {
        let is_custom = value.trim() != BUILTIN_CLIENT_ID;
        *self.client_id.write() = ClientIdConfig {
            value: value.trim().to_string(),
            is_custom,
        };
    }

    pub fn current_client_id(&self) -> ClientIdConfig {
        self.client_id.read().clone()
    }

    /// Публикует активность (валидация выполняется здесь).
    pub async fn set_activity(&self, activity: Activity) -> Result<(), String> {
        activity
            .validate()
            .map_err(|e| e.join("; "))?;
        self.tx
            .send(ManagerCommand::SetActivity(activity))
            .await
            .map_err(|_| "менеджер Discord не запущен".to_string())
    }

    /// Очищает активность в Discord.
    pub async fn clear_activity(&self) -> Result<(), String> {
        self.tx
            .send(ManagerCommand::ClearActivity)
            .await
            .map_err(|_| "менеджер Discord не запущен".to_string())
    }

    /// Полная остановка (при выходе из приложения).
    pub async fn stop(&self) -> Result<(), String> {
        let _ = self.tx.send(ManagerCommand::Stop).await;
        Ok(())
    }

    pub async fn state(&self) -> ConnectionState {
        self.client.state().await
    }

    /// Активность, применённая в Discord (если есть).
    pub fn live_activity(&self) -> Option<Activity> {
        self.state.read().live.clone()
    }

    async fn run_loop(self: Arc<Self>, app: AppHandle, mut rx: mpsc::Receiver<ManagerCommand>) {

        let pid = std::process::id();
        let mut attempt: u32 = 0;
        let mut ever_connected = false;
        let mut stopped = false;
        let mut force_reconnect = false;

        'outer: loop {
            // --- Фаза подключения ---
            loop {
                if stopped { break 'outer; }
                attempt += 1;
                let _ = app.emit("discord://status", StatusEvent::Connecting { attempt });
                let cid = self.client_id.read().value.clone();
                match self.client.connect(&cid).await {
                    Ok(user) => {
                        ever_connected = true;
                        let flavor = self.client.state().await.flavor;
                        let evt = if attempt > 1 || force_reconnect {
                            StatusEvent::Reconnected { flavor, user: Some(user) }
                        } else {
                            StatusEvent::Connected { flavor, user: Some(user) }
                        };
                        let _ = app.emit("discord://status", evt);
                        force_reconnect = false;
                        attempt = 0;
                        break;
                    }
                    Err(e) => {
                        let msg = humanize_ipc_error(&e);
                        self.client.mark_disconnected(msg.clone()).await;
                        let _ = app.emit("discord://status", StatusEvent::Disconnected { reason: msg });
                        // Пауза перед повтором: сначала часто, потом реже.
                        let pause = if attempt < 5 { 3 } else if attempt < 20 { 10 } else { 30 };
                        tokio::select! {
                            _ = tokio::time::sleep(Duration::from_secs(pause)) => {},
                            Some(cmd) = rx.recv() => {
                                match cmd {
                                    ManagerCommand::SetActivity(a) => { self.state.write().pending = Some(a); },
                                    ManagerCommand::ClearActivity => { self.state.write().pending = None; self.state.write().live = None; },
                                    ManagerCommand::Stop => { stopped = true; break 'outer; }
                                }
                            }
                        }
                    }
                }
            }

            // --- Подключено: применяем отложенную активность ---
            let pending_now = self.state.read().pending.clone();
            if let Some(pending) = pending_now {
                if self.client.set_activity(pid, &pending).await.is_ok() {
                    let mut st = self.state.write();
                    st.live = Some(pending.clone());
                }
            }

            // --- Фаза поддержки соединения ---
            let mut keepalive = tokio::time::interval(Duration::from_secs(15));
            keepalive.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Delay);
            loop {
                tokio::select! {
                    _ = keepalive.tick() => {
                        if let Err(e) = self.client.keepalive().await {
                            let msg = humanize_ipc_error(&e);
                            self.client.mark_disconnected(msg.clone()).await;
                            let _ = app.emit("discord://status", StatusEvent::Disconnected { reason: msg });
                            force_reconnect = true;
                            break; // на переподключение
                        }
                    }
                    Some(cmd) = rx.recv() => {
                        match cmd {
                            ManagerCommand::SetActivity(a) => {
                                if let Err(e) = self.client.set_activity(pid, &a).await {
                                    let msg = humanize_ipc_error(&e);
                                    self.client.mark_disconnected(msg.clone()).await;
                                    let _ = app.emit("discord://status", StatusEvent::Disconnected { reason: msg });
                                    self.state.write().pending = Some(a);
                                    force_reconnect = true;
                                    break;
                                } else {
                                    let mut st = self.state.write();
                                    st.pending = Some(a.clone());
                                    st.live = Some(a);
                                }
                            }
                            ManagerCommand::ClearActivity => {
                                let _ = self.client.clear_activity(pid).await;
                                let mut st = self.state.write();
                                st.pending = None;
                                st.live = None;
                            }
                            ManagerCommand::Stop => { stopped = true; break 'outer; }
                        }
                    }
                }
            }
        }
        // Финальная очистка активности при остановке — не обязательна:
        // Discord сам снимает активность при закрытии пайпа.
        let _ = ever_connected;
    }
}

/// Человекочитаемое описание ошибки IPC.
pub fn humanize_ipc_error(e: &IpcError) -> String {
    match e {
        IpcError::NoPipe => "Discord не запущен".to_string(),
        IpcError::Timeout => "Discord не отвечает (таймаут)".to_string(),
        IpcError::Closed => "Discord закрыл соединение".to_string(),
        IpcError::Rpc(msg) => msg.clone(),
        IpcError::Io(io) => format!("Ошибка соединения: {io}"),
        IpcError::Protocol(p) => format!("Ошибка протокола: {p}"),
    }
}

/// Определяет flavor по имени процесса (переиспользуем из ipc).
pub fn current_flavor() -> DiscordFlavor {
    super::ipc::detect_flavor()
}
