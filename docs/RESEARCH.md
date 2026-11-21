# Исследование существующих реализаций Discord Rich Presence

Документ фиксирует результаты исследования открытых проектов и протокола,
проведённого перед разработкой Larpscord. Все ключевые факты о протоколе
проверены **эмпирически** — живыми подключениями к работающему Discord
(`\\.\pipe\discord-ipc-0`) на машине разработчика, а не только по документации.

Дата исследования: 21–22 ноября 2026.

---

## 1. Проверка протокола Discord RPC (эмпирика)

Протокол исследован прямыми запросами к живому Discord через named pipe.
Результаты:

| Вопрос | Результат проверки |
|---|---|
| Формат фрейма | `[op: u32 LE][len: u32 LE][JSON-UTF8]` — подтверждено |
| Рукопожатие | `op=0, {"v":1,"client_id":"..."}` → ответ `op=1` c `evt=READY`, `data.user` (username, global_name, id) |
| Неверный client_id | `op=2 (CLOSE)` + `{"code":4000,"message":"Invalid Client ID"}` — подтверждено живьём |
| SET_ACTIVITY | `op=1, {"cmd":"SET_ACTIVITY","args":{"pid":N,"activity":{...}},"nonce":"uuid"}` — ответ `op=1` с тем же nonce |
| CLEAR_ACTIVITY | **Не существует!** Discord отвечает `Invalid command: CLEAR_ACTIVITY`. Очистка — `SET_ACTIVITY` с `"activity": null` |
| PING/PONG | `op=3 {}` → `op=4 {}` — работает, используется для keepalive |
| Внешние картинки | `large_image: "https://..."` — Discord **проксирует**: возвращает `mp:external/<hash>/https/...`. Значит, произвольные картинки по URL работают без загрузки ассетов |
| Поле `name` | Произвольное имя приложения передаётся и возвращается в ответе |
| Несуществующий asset key | `large_image: "nonexistent_key"` — принимается без ошибки (картинка просто не показывается) |
| Кнопки | `buttons: [{label, url}]`, максимум 2; в ответе возвращаются только метки + `metadata.button_urls` |
| Activity type | `PLAYING=0, STREAMING=1, LISTENING=2, WATCHING=3, CUSTOM=4, COMPETING=5` (по документации Discord; CUSTOM через RPC не устанавливается) |
| Готовый client_id | `1129859263741837373` (приложение music-discord-rpc) — рукопожатие проходит, SET_ACTIVITY принимается |

## 2. Исследованные проекты

### 2.1 vionya/discord-rich-presence (Rust, MIT)

**Что изучено:** полный исходный код — `src/ipc_windows.rs`, `src/discord_ipc.rs`,
`src/pack_unpack.rs`, `src/activity.rs`.

**Что взято в Larpscord:**
- Подход к подключению на Windows: перебор `\\?\pipe\discord-ipc-0..9` с
  `access_mode(0x3)` — мы используем crate `interprocess` (tokio), но логика
  перебора пайпов та же (`src-tauri/src/discord/ipc.rs`).
- Формирование payload SET_ACTIVITY с `pid` и `nonce` (`src-tauri/src/discord/rpc.rs`).
- Упаковка фреймов `[op u32 LE][len u32 LE][JSON]` (`pack_unpack.rs` → наша
  реализация `write_frame`/`read_frame`).

**Отличия нашего решения:** асинхронность (tokio), чтение ответов по nonce,
keepalive PING/PONG, автоматический реконнект — в оригинале этого нет.

### 2.2 patryk-ku/music-discord-rpc (Rust, MIT)

**Что изучено:** `config.yaml`, `src/main.rs`, `src/utils.rs`, `Cargo.toml`, `build.rs`.

**Что взято в Larpscord:**
- **Встроенный Application ID `1129859263741837373`** — реальное существующее
  приложение Discord, работоспособность подтверждена живым рукопожатием.
  Используется как встроенный по умолчанию, чтобы пользователю не нужно было
  создавать своё приложение в Discord Developer Portal.
- Подход к музыкальной активности: `type=2 (Listening)`, трек в `details`,
  исполнитель в `state`, обложка по внешнему URL, start-timestamp для прогресса.
- Подтверждение жизнеспособности схемы «внешние URL картинок вместо ассетов»
  (Last.fm/YouTube thumbnails как `large_image`).

### 2.3 discord-rpc (официальный архив, C++), discord.js/RPC, pypresence

Изучены как справочники по протоколу: таблица опкодов, команды, envelope
ошибок `evt: "ERROR"` с `data.code/message`. Частично использовано при
проектировании обработчика ответов в `rpc.rs`.

### 2.4 Crate `interprocess` (0BSD OR Apache-2.0)

Выбран для named pipes вместо ручной работы с WinAPI: поддерживает tokio,
даёт кроссплатформенный API local sockets. API проверено по docs.rs v2.4.9:
`interprocess::local_socket::tokio::prelude::*`, `to_ns_name::<GenericNamespaced>()`,
`LocalSocketStream::connect()`.

### 2.5 Прочее

- **EasyRP / DiscordRPCMaker / RPC-Reborn** — концепция «пользователь вводит
  свой client_id»; в Larpscord сделано наоборот: встроенный ID по умолчанию
  + опциональный свой ID в настройках (нулевая настройка для обычного пользователя).
- **Кандидатные ID 1077166212729997312 и др.** — проверены живьём, Discord
  отвечает `Invalid Client ID`; отброшены.

## 3. Ключевые технические ограничения (найдено в ходе исследования)

1. **Discord RPC не принимает локальные файлы картинок.** Решение Larpscord:
   внешние https-URL (Discord сам проксирует через `mp:external/...`) —
   проверено живьём. Для своих ассетов — свой Application ID в настройках.
2. **CLEAR_ACTIVITY отсутствует** в текущем протоколе — очищаем через
   `SET_ACTIVITY` с `activity: null`.
3. **CUSTOM-статус (type=4) не устанавливается через RPC** — это отдельная
   функция пользовательских настроек Discord.
4. **Shared Application ID**: имя приложения в профиле берётся из поля `name`
   активности (не жёстко из приложения), что проверено живьём — значит,
   встроенный ID не ограничивает кастомизацию имени.
5. **Активность живёт, пока жив процесс-клиент**: при закрытии Larpscord
   Discord снимает активность сам (пайп закрывается).

## 4. Лицензии и атрибуция

| Проект | Лицензия | Использование |
|---|---|---|
| vionya/discord-rich-presence | MIT | Идеи + фрагменты логики IPC (переписаны) |
| patryk-ku/music-discord-rpc | MIT | Встроенный Application ID, схема музыкальной активности |
| interprocess | 0BSD OR Apache-2.0 | Библиотека-зависимость |
| tauri | MIT OR Apache-2.0 | Библиотека-зависимость |

Копирование кода дословно не производилось; заимствованные фрагменты
переписаны под архитектуру Larpscord. Условия MIT соблюдены (см. LICENSE
и этот документ с атрибуцией).
