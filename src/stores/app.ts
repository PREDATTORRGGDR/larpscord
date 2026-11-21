// Центральное состояние приложения (zustand).

import { create } from "zustand";
import { subscribeWithSelector } from "zustand/middleware";
import type { Activity, AppData, Preset, Settings } from "@/types";
import { api } from "@/lib/api";
import { defaultActivity } from "@/types";

export type Page = "editor" | "music" | "presets" | "profiles" | "settings";

interface Toast {
  id: number;
  kind: "success" | "error" | "info";
  text: string;
}

interface AppState {
  page: Page;
  appState: AppData | null;
  /** Черновик активности в редакторе. */
  draft: Activity;
  /** Пресеты. */
  presets: Preset[];
  /** Загружено ли всё. */
  loaded: boolean;
  loading: boolean;
  toasts: Toast[];
  /** Undo/redo стеки черновика. */
  undoStack: Activity[];
  redoStack: Activity[];

  setPage: (p: Page) => void;
  refresh: () => Promise<void>;
  setDraft: (d: Activity, pushHistory?: boolean) => void;
  applyDraft: () => Promise<boolean>;
  clearPresence: () => Promise<void>;
  loadPresets: () => Promise<void>;
  saveSettings: (s: Settings) => Promise<boolean>;
  toast: (kind: Toast["kind"], text: string) => void;
  dismissToast: (id: number) => void;
  undo: () => void;
  redo: () => void;
}

let toastId = 0;

export const useApp = create<AppState>()(
  subscribeWithSelector((set, get) => ({
    page: "editor",
    appState: null,
    draft: defaultActivity(),
    presets: [],
    loaded: false,
    loading: true,
    toasts: [],
    undoStack: [],
    redoStack: [],

    setPage: (p) => set({ page: p }),

    refresh: async () => {
      try {
        const st = await api.getAppState();
        set({
          appState: st,
          draft: st.lastActivity ?? get().draft,
          loaded: true,
          loading: false,
        });
      } catch (e) {
        console.error("getAppState", e);
        set({ loading: false });
        get().toast("error", String(e));
      }
    },

    setDraft: (d, pushHistory = true) => {
      const prev = get().draft;
      const undoStack = pushHistory ? [...get().undoStack.slice(-40), prev] : get().undoStack;
      set({ draft: d, undoStack, redoStack: [] });
    },

    applyDraft: async () => {
      const { draft } = get();
      try {
        await api.applyActivity(draft);
        await get().refresh();
        get().toast("success", "Активность применена — смотрите Discord");
        return true;
      } catch (e) {
        get().toast("error", humanError(e));
        return false;
      }
    },

    clearPresence: async () => {
      try {
        await api.clearPresence();
        await get().refresh();
        get().toast("success", "Активность сброшена");
      } catch (e) {
        get().toast("error", humanError(e));
      }
    },

    loadPresets: async () => {
      try {
        const presets = await api.getBuiltinPresets();
        set({ presets });
      } catch (e) {
        console.error("loadPresets", e);
      }
    },

    saveSettings: async (s) => {
      try {
        await api.saveSettings(s);
        await get().refresh();
        get().toast("success", "Настройки сохранены");
        return true;
      } catch (e) {
        get().toast("error", humanError(e));
        return false;
      }
    },

    toast: (kind, text) => {
      const id = ++toastId;
      set((st) => ({ toasts: [...st.toasts, { id, kind, text }] }));
      setTimeout(() => get().dismissToast(id), 4200);
    },

    dismissToast: (id) =>
      set((st) => ({ toasts: st.toasts.filter((t) => t.id !== id) })),

    undo: () => {
      const { undoStack, redoStack, draft } = get();
      if (undoStack.length === 0) return;
      const prev = undoStack[undoStack.length - 1];
      set({
        draft: prev,
        undoStack: undoStack.slice(0, -1),
        redoStack: [...redoStack, draft],
      });
    },

    redo: () => {
      const { undoStack, redoStack, draft } = get();
      if (redoStack.length === 0) return;
      const next = redoStack[redoStack.length - 1];
      set({
        draft: next,
        redoStack: redoStack.slice(0, -1),
        undoStack: [...undoStack, draft],
      });
    },
  }))
);

function humanError(e: unknown): string {
  const s = typeof e === "string" ? e : String((e as Error)?.message ?? e);
  return s;
}
