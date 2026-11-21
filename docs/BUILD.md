# Сборка Larpscord

## Требования

- **Rust** 1.77+ (rustup, таргет `x86_64-pc-windows-msvc`)
- **Node.js** 20+ и npm
- **WebView2 Runtime** (в Windows 10/11 предустановлен)
- Для инсталлятора NSIS ничего дополнительно ставить не нужно — Tauri скачает NSIS сам

## Разработка

```bash
npm install          # зависимости фронтенда
npm run tauri dev    # запуск в dev-режиме (hot reload)
```

## Проверки

```bash
npm run lint                # tsc --noEmit (типы фронтенда)
cd src-tauri && cargo test  # юнит-тесты бэкенда (IPC, модели, хранилище)
```

## Продакшн-сборка (Windows)

```bash
npm run tauri build
```

Результат:

- Исполняемый файл: `src-tauri/target/release/larpscord.exe`
- Установщик NSIS: `src-tauri/target/release/bundle/nsis/Larpscord_1.0.0_x64-setup.exe`

Установщик создаётся в режиме `currentUser` — установка без прав администратора,
в `%LOCALAPPDATA%`, ярлык на рабочий стол ставит NSIS автоматически.

## Иконки

Иконки генерируются скриптом `scripts/make-icon.py` (Pillow):

```bash
python scripts/make-icon.py
```

## Структура сборки

- Frontend: Vite + React + TypeScript → `dist/`
- Backend: Tauri 2 (Rust) → `src-tauri/`
- Инсталлятор: NSIS, конфигурация в `src-tauri/tauri.conf.json`
