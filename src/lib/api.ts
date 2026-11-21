// Мост к Rust-бэкенду (Tauri invoke).

import { invoke } from "@tauri-apps/api/core";
import type { Activity, AppData, ConnectionState, Preset, Settings } from "@/types";

export const api = {
  getAppState: () => invoke<AppData>("get_app_state"),
  applyActivity: (activity: Activity) =>
    invoke<{ ok: boolean }>("apply_activity", { activity }),
  clearPresence: () => invoke<{ ok: boolean }>("clear_presence"),
  getSettings: () => invoke<Settings>("get_settings"),
  saveSettings: (settings: Settings) =>
    invoke<{ ok: boolean }>("save_settings", { settings }),
  getConnectionStatus: () => invoke<ConnectionState>("get_connection_status"),
  reconnectDiscord: () => invoke<{ ok: boolean }>("reconnect_discord"),
  saveLastActivity: (activity: Activity) =>
    invoke<{ ok: boolean }>("save_last_activity", { activity }),
  getBuiltinPresets: () => invoke<Preset[]>("get_builtin_presets"),
};
