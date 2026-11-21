// Модели, зеркалящие Rust-структуры (serde rename в camelCase через JSON).

export type ActivityType = "Playing" | "Streaming" | "Listening" | "Watching" | "Competing";

export interface ActivityButton {
  label: string;
  url: string;
}

export interface Activity {
  name?: string | null;
  details?: string | null;
  state?: string | null;
  type: ActivityType;
  timestamps_start?: number | null;
  timestamps_end?: number | null;
  large_image?: string | null;
  large_text?: string | null;
  small_image?: string | null;
  small_text?: string | null;
  buttons: ActivityButton[];
}

export interface UserInfo {
  id: string;
  username: string;
  globalName: string | null;
}

export interface ConnectionState {
  connected: boolean;
  flavor: string;
  user: UserInfo | null;
  last_error: string | null;
  last_activity_sent_at: number | null;
  has_pending_activity: boolean;
}

export interface Settings {
  autoconnect: boolean;
  restore_activity: boolean;
  auto_reapply: boolean;
  minimize_to_tray: boolean;
  start_minimized: boolean;
  launch_on_startup: boolean;
  custom_client_id: string | null;
  language: string;
  theme: string;
}

export interface Preset {
  id: string;
  name: string;
  category: string;
  activity: Activity;
  builtin: boolean;
}

export interface AppData {
  settings: Settings;
  lastActivity: Activity | null;
  activeActivity: Activity | null;
  connection: ConnectionState;
  builtinClientId: string;
  customClientId: { value: string; is_custom: boolean };
  storageDir: string;
  corruptionWarning: string | null;
  appVersion: string;
}

export const ACTIVITY_TYPES: { value: ActivityType; label: string; verb: string }[] = [
  { value: "Playing", label: "Играет", verb: "Играет в" },
  { value: "Streaming", label: "Транслирует", verb: "Транслирует" },
  { value: "Listening", label: "Слушает", verb: "Слушает" },
  { value: "Watching", label: "Смотрит", verb: "Смотрит" },
  { value: "Competing", label: "Соревнуется", verb: "Соревнуется в" },
];

// Числовые значения для Rust (serde). «type» сериализуется как имя варианта.
export const ACTIVITY_TYPE_TO_NUM: Record<ActivityType, number> = {
  Playing: 0,
  Streaming: 1,
  Listening: 2,
  Watching: 3,
  Competing: 5,
};

export function emptyActivity(): Activity {
  return {
    name: null,
    details: "",
    state: "",
    type: "Playing",
    timestamps_start: null,
    timestamps_end: null,
    large_image: null,
    large_text: null,
    small_image: null,
    small_text: null,
    buttons: [],
  };
}

export function defaultActivity(): Activity {
  return {
    name: "Larpscord",
    details: "Настраиваю присутствие",
    state: "",
    type: "Playing",
    timestamps_start: null,
    timestamps_end: null,
    large_image: null,
    large_text: null,
    small_image: null,
    small_text: null,
    buttons: [],
  };
}
