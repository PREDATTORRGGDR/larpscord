// Мост к Rust-бэкенду (Tauri invoke).

import { invoke } from "@tauri-apps/api/core";
import type { Activity, AppData, ConnectionState, Preset, Settings } from "@/types";

export interface MediaSnapshot {
  available: boolean;
  source: string | null;
  title: string | null;
  artist: string | null;
  album: string | null;
  playing: boolean;
  positionSecs: number | null;
  durationSecs: number | null;
}

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
  listPresets: () => invoke<Preset[]>("list_presets"),
  savePreset: (preset: Preset) => invoke<{ ok: boolean }>("save_preset", { preset }),
  deletePreset: (id: string) => invoke<{ ok: boolean }>("delete_preset", { id }),
  exportPresets: () => invoke<string>("export_presets"),
  importPresets: (json: string) => invoke<{ added: number; updated: number }>("import_presets", { json }),
  musicPoll: () => invoke<MediaSnapshot>("music_poll"),
  musicSetMode: (mode: "off" | "manual" | "auto") => invoke<{ ok: boolean }>("music_set_mode", { mode }),
  musicCover: () => invoke<{ dataUrl: string | null }>("music_cover"),
  musicState: () => invoke<{ mode: string; snapshot: MediaSnapshot | null; cover: string | null }>("music_state"),
  uploadCover: () => invoke<{ url: string }>("upload_cover"),
};
