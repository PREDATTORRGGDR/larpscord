# Архитектура Larpscord

Larpscord — настольное приложение (Tauri 2): Rust-бэкенд для Discord RPC и
данных, React-фронтенд для интерфейса.

```
┌─────────────────────────────────────────────┐
│  Фронтенд (React + TypeScript + Vite)       │
│  src/                                       │
│    components/  — UI-компоненты             │
│    features/    — страницы (editor, music…) │
│    stores/      — zustand-состояние         │
│    lib/api.ts   — invoke-мост к Rust        │
└──────────────────────┬──────────────────────┘
                       │ Tauri IPC (invoke / events)
┌──────────────────────┴──────────────────────┐
│  Бэкенд (Rust, src-tauri/src)               │
│                                             │
│  discord/                                   │
│    ipc.rs      — named pipe, фреймы, опкоды │
│    rpc.rs      — рукопожатие, SET_ACTIVITY, │
│                  keepalive PING/PONG        │
│    manager.rs  — авто-подключение,          │
│                  реконнект, события статуса │
│  presence/     — модели Activity + лимиты   │
│  storage/      — JSON-хранилище (%APPDATA%) │
│  presets/      — встроенные пресеты         │
│  music/         — SMTC-снапшоты, слежение,  │
│                  загрузка обложек (catbox)  │
│  integrations/ — список процессов Windows   │
│  commands/     — Tauri-команды для фронтенда│
└──────────────────────┬──────────────────────┘
                       │ \\.\pipe\discord-ipc-N
              ┌────────┴────────┐
              │  Discord (RPC)  │
              └─────────────────┘
```

## Поток данных

1. **Запуск**: `lib.rs` инициализирует `Storage` и `DiscordManager`;
   менеджер в фоне подключается к первому доступному пайпу `discord-ipc-0..9`,
   выполняет рукопожатие (`{"v":1,"client_id":...}`) и рассылает статус
   через событие `discord://status`.
2. **Редактирование**: черновик живёт в zustand (`stores/app.ts`),
   автосохранение — debounce 1.2 с в `config.json` (поле `last_activity`).
3. **Применение**: фронтенд вызывает `apply_activity` → валидация лимитов →
   `SET_ACTIVITY` по пайпу → сохранение `active_activity`.
4. **Сброс**: `SET_ACTIVITY` с `activity: null` (команды CLEAR_ACTIVITY
   в протоколе нет — проверено эмпирически).
5. **Обрыв связи**: keepalive каждые 15 с; при ошибке — статус `disconnected`,
   цикл менеджера переподключается с бэкоффом (3с → 10с → 30с) и после
   успеха повторно применяет последнюю активность (pending).

## Хранилище

`%APPDATA%/Larpscord/config.json`:

```json
{
  "version": 1,
  "settings": { "autoconnect": true, "...": "..." },
  "last_activity": { "...Activity...": "" },
  "active_activity": null
}
```

- Атомарная запись: `config.json.tmp` → rename.
- Повреждённый файл: бэкап `config.corrupt.bak` + старт с дефолтов.
- `version` — для будущих миграций (`storage::migrate`).

## Музыкальный авто-режим (SMTC)

`music/smtc.rs` читает Windows Media Session (GlobalSystemMediaTransportControls)
в отдельном потоке (WinRT-объекты не Send — используется current_thread runtime
внутри spawn_blocking). `music/watcher.rs` опрашивает сессию каждые 2 секунды,
при смене трека/статуса публикует активность «Слушает» с прогрессом
(start = now − position). Обложка (байты из SMTC) грузится на catbox.moe
(`music/cover.rs`, кэш 6 ч по подписи трека) — Discord принимает только URL.
Кнопка «Слушать» ведёт на soundcloud.com/search?q=трек.

## Встроенный Application ID

По умолчанию используется ID `1129859263741837373` (реальное приложение
Discord, заимствовано из music-discord-rpc, MIT; см. docs/RESEARCH.md).
Пользователь может указать свой ID в настройках — тогда активность
публикуется от его приложения.

## Решения и компромиссы

- **interprocess + tokio** вместо синхронных файлов: нужно для keepalive и
  реконнекта без блокировки UI.
- **Внешние URL картинок**: Discord проксирует `https://...` в
  `mp:external/...` — проверено живьём; это заменяет загрузку ассетов.
- **zustand** вместо Redux: минимум шаблонного кода, подписки по селекторам.
- **CSS без Tailwind**: точный контроль стеклянных эффектов и типографики.
