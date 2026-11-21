//! Discord IPC: обнаружение пайпов, подключение, фрейминг.
//!
//! Протокол Discord RPC поверх локального именованного пайпа:
//! каждый фрейм = `[op: u32 LE][len: u32 LE][JSON-тело]`.
//! Пайпы: `discord-ipc-0` … `discord-ipc-9` (Stable, PTB и Canary используют
//! одинаковые имена; кто первым занял пайп — тот его и обслуживает).

use interprocess::local_socket::tokio::prelude::*;
use interprocess::local_socket::GenericNamespaced;
use serde_json::Value;
use std::time::Duration;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::time::timeout;

/// Максимальная длина JSON-тела фрейма (защита от мусора в пайпе).
const MAX_FRAME_LEN: usize = 4 * 1024 * 1024;

/// Вариант клиента Discord, к которому подключились.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum DiscordFlavor {
    Stable,
    Ptb,
    Canary,
}

impl DiscordFlavor {
    pub fn as_str(&self) -> &'static str {
        match self {
            DiscordFlavor::Stable => "stable",
            DiscordFlavor::Ptb => "ptb",
            DiscordFlavor::Canary => "canary",
        }
    }

    pub fn display_name(&self) -> &'static str {
        match self {
            DiscordFlavor::Stable => "Discord",
            DiscordFlavor::Ptb => "Discord PTB",
            DiscordFlavor::Canary => "Discord Canary",
        }
    }
}

/// Опкоды протокола Discord RPC.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum Op {
    Handshake = 0,
    Frame = 1,
    Close = 2,
    Ping = 3,
    Pong = 4,
}

impl Op {
    pub fn from_u32(v: u32) -> Option<Op> {
        match v {
            0 => Some(Op::Handshake),
            1 => Some(Op::Frame),
            2 => Some(Op::Close),
            3 => Some(Op::Ping),
            4 => Some(Op::Pong),
            _ => None,
        }
    }
}

/// Ошибки IPC-уровня.
#[derive(Debug, thiserror::Error)]
pub enum IpcError {
    #[error("Discord не запущен: пайпы discord-ipc-0..9 не найдены")]
    NoPipe,
    #[error("Таймаут обмена с Discord")]
    Timeout,
    #[error("Discord закрыл соединение (opcode CLOSE)")]
    Closed,
    #[error("Discord ответил ошибкой: {0}")]
    Rpc(String),
    #[error("Ошибка ввода-вывода: {0}")]
    Io(#[from] std::io::Error),
    #[error("Некорректный ответ Discord: {0}")]
    Protocol(String),
}

impl IpcError {
    /// true — стоит переподключаться (Discord перезапущен и т.п.).
    pub fn is_reconnectable(&self) -> bool {
        matches!(
            self,
            IpcError::NoPipe | IpcError::Timeout | IpcError::Closed | IpcError::Io(_)
        )
    }
}

/// Активное соединение с Discord по именованному пайпу.
pub struct IpcConnection {
    stream: LocalSocketStream,
    flavor: DiscordFlavor,
}

impl IpcConnection {
    /// Подключается к первому отвечающему пайпу `discord-ipc-N`.
    pub async fn connect() -> Result<Self, IpcError> {
        for i in 0..10u32 {
            let name = format!("discord-ipc-{i}");
            let ns = match name.to_ns_name::<GenericNamespaced>() {
                Ok(ns) => ns,
                Err(_) => continue,
            };
            match LocalSocketStream::connect(ns).await {
                Ok(stream) => {
                    let flavor = detect_flavor();
                    return Ok(Self { stream, flavor });
                }
                Err(_) => continue,
            }
        }
        Err(IpcError::NoPipe)
    }

    pub fn flavor(&self) -> DiscordFlavor {
        self.flavor
    }

    /// Рукопожатие RPC: `{"v":1,"client_id":"..."}` — ждём ответ с `READY`.
    pub async fn handshake(&mut self, client_id: &str) -> Result<Value, IpcError> {
        let payload = serde_json::json!({ "v": 1, "client_id": client_id });
        self.write_frame(Op::Handshake, &payload).await?;
        let deadline = Duration::from_secs(5);
        let (op, data) = match timeout(deadline, self.read_frame()).await {
            Ok(r) => r?,
            Err(_) => return Err(IpcError::Timeout),
        };
        match op {
            Op::Frame => Ok(data),
            Op::Close => Err(IpcError::Closed),
            other => Err(IpcError::Protocol(format!(
                "неожиданный opcode {other:?} на рукопожатии"
            ))),
        }
    }

    pub async fn write_frame(&mut self, op: Op, payload: &Value) -> Result<(), IpcError> {
        let body = serde_json::to_vec(payload)
            .map_err(|e| IpcError::Protocol(format!("ошибка сериализации: {e}")))?;
        let mut buf = Vec::with_capacity(8 + body.len());
        buf.extend_from_slice(&(op as u32).to_le_bytes());
        buf.extend_from_slice(&(body.len() as u32).to_le_bytes());
        buf.extend_from_slice(&body);
        self.stream.write_all(&buf).await?;
        self.stream.flush().await?;
        Ok(())
    }

    pub async fn read_frame(&mut self) -> Result<(Op, Value), IpcError> {
        let mut header = [0u8; 8];
        self.stream.read_exact(&mut header).await?;
        let op_raw = u32::from_le_bytes(header[0..4].try_into().unwrap());
        let len = u32::from_le_bytes(header[4..8].try_into().unwrap()) as usize;
        if len > MAX_FRAME_LEN {
            return Err(IpcError::Protocol(format!(
                "слишком большой фрейм: {len} байт"
            )));
        }
        let mut body = vec![0u8; len];
        self.stream.read_exact(&mut body).await?;
        let op = Op::from_u32(op_raw)
            .ok_or_else(|| IpcError::Protocol(format!("неизвестный opcode {op_raw}")))?;
        let data: Value = serde_json::from_slice(&body)
            .map_err(|e| IpcError::Protocol(format!("битый JSON во фрейме: {e}")))?;
        Ok((op, data))
    }

    /// Читает фрейм с таймаутом (для чтения ответа на команду).
    pub async fn read_frame_timeout(&mut self, secs: u64) -> Result<(Op, Value), IpcError> {
        match timeout(Duration::from_secs(secs), self.read_frame()).await {
            Ok(r) => r,
            Err(_) => Err(IpcError::Timeout),
        }
    }
}

/// Определяет, какой вариант Discord запущен, по списку процессов.
/// Приоритет: Canary > PTB > Stable (реже встречаются — важнее показать).
#[cfg(windows)]
pub fn detect_flavor() -> DiscordFlavor {
    let mut has_stable = false;
    let mut has_ptb = false;
    let mut has_canary = false;
    for name in crate::integrations::list_process_names() {
        let lower = name.to_ascii_lowercase();
        if lower.contains("discordcanary") {
            has_canary = true;
        } else if lower.contains("discordptb") {
            has_ptb = true;
        } else if lower == "discord.exe" {
            has_stable = true;
        }
    }
    if has_canary {
        DiscordFlavor::Canary
    } else if has_ptb {
        DiscordFlavor::Ptb
    } else if has_stable {
        DiscordFlavor::Stable
    } else {
        DiscordFlavor::Stable
    }
}

#[cfg(not(windows))]
pub fn detect_flavor() -> DiscordFlavor {
    DiscordFlavor::Stable
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn op_roundtrip() {
        for raw in 0..5u32 {
            let op = Op::from_u32(raw).unwrap();
            assert_eq!(op as u32, raw);
        }
        assert!(Op::from_u32(99).is_none());
    }
}
