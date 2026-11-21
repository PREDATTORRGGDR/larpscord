// Корневой компонент Larpscord.

import { useEffect } from "react";
import { Sidebar } from "@/components/layout/Sidebar";
import { EditorPage } from "@/features/editor/EditorPage";
import { MusicPage } from "@/features/music/MusicPage";
import { PresetsPage } from "@/features/presets/PresetsPage";
import { ProfilesPage } from "@/features/profiles/ProfilesPage";
import { SettingsPage } from "@/features/settings/SettingsPage";
import { useApp } from "@/stores/app";
import { api } from "@/lib/api";
import { listen } from "@tauri-apps/api/event";
import { clsx } from "clsx";

export default function App() {
  const page = useApp((s) => s.page);
  const refresh = useApp((s) => s.refresh);
  const loaded = useApp((s) => s.loaded);

  useEffect(() => {
    void refresh();
  }, [refresh]);

  // События статуса Discord из Rust.
  useEffect(() => {
    let unlisten: (() => void) | undefined;
    void (async () => {
      try {
        unlisten = await listen("discord://status", () => {
          void refresh();
        });
      } catch {
        /* в dev без Tauri — молча */
      }
    })();
    return () => unlisten?.();
  }, [refresh]);

  // Периодический поллинг статуса (лёгкий).
  useEffect(() => {
    const t = setInterval(() => {
      void api.getConnectionStatus().then((c) => {
        const st = useApp.getState().appState;
        if (st && (st.connection.connected !== c.connected || st.connection.last_error !== c.last_error)) {
          useApp.setState({
            appState: { ...st, connection: c },
          });
        }
      }).catch(() => {});
    }, 4000);
    return () => clearInterval(t);
  }, []);

  // Горячие клавиши: Ctrl+Z / Ctrl+Shift+Z / Ctrl+Enter.
  useEffect(() => {
    const onKey = (e: KeyboardEvent) => {
      if (!(e.ctrlKey || e.metaKey)) return;
      const key = e.key.toLowerCase();
      if (key === "z" && !e.shiftKey) {
        e.preventDefault();
        useApp.getState().undo();
      } else if ((key === "z" && e.shiftKey) || key === "y") {
        e.preventDefault();
        useApp.getState().redo();
      } else if (key === "enter") {
        e.preventDefault();
        void useApp.getState().applyDraft();
      }
    };
    window.addEventListener("keydown", onKey);
    return () => window.removeEventListener("keydown", onKey);
  }, []);

  // Автосохранение черновика (debounce).
  useEffect(() => {
    const unsub = useApp.subscribe((state, prev) => {
      if (state.draft !== prev.draft) {
        clearTimeout((saveTimer as any)._t);
        (saveTimer as any)._t = setTimeout(() => {
          void api.saveLastActivity(state.draft).catch(() => {});
        }, 1200);
      }
    });
    return unsub;
  }, []);

  return (
    <>
      <div className="bg-atmosphere" />
      <div className="app-shell">
        <Sidebar />
        <main className={clsx("workspace", !loaded && "workspace--loading")}>
          {page === "editor" && <EditorPage />}
          {page === "music" && <MusicPage />}
          {page === "presets" && <PresetsPage />}
          {page === "profiles" && <ProfilesPage />}
          {page === "settings" && <SettingsPage />}
        </main>
      </div>
      <Toasts />
    </>
  );
}

const saveTimer = { _t: undefined as undefined | ReturnType<typeof setTimeout> } as any;

function Toasts() {
  const toasts = useApp((s) => s.toasts);
  const dismiss = useApp((s) => s.dismissToast);
  return (
    <div className="toasts">
      {toasts.map((t) => (
        <div key={t.id} className={clsx("toast", `toast--${t.kind}`)} onClick={() => dismiss(t.id)}>
          {t.text}
        </div>
      ))}
    </div>
  );
}
