// Сайдбар Larpscord: навигация + индикатор Discord + версия.

import { clsx } from "clsx";
import { useApp, type Page } from "@/stores/app";
import type { ReactNode } from "react";
import logoIcon from "@/assets/icon.png";

const NAV: { id: Page; label: string; icon: ReactNode }[] = [
  {
    id: "editor",
    label: "Редактор",
    icon: (
      <path d="M4 3.5h12a1.5 1.5 0 011.5 1.5v10a1.5 1.5 0 01-1.5 1.5H4A1.5 1.5 0 012.5 15V5A1.5 1.5 0 014 3.5zM6 7.5h8M6 10.5h8M6 13.5h5" stroke="currentColor" strokeWidth="1.4" strokeLinecap="round" />
    ),
  },
  {
    id: "music",
    label: "Музыка",
    icon: (
      <>
        <path d="M9 12.5V5.8a.6.6 0 01.47-.58l4.4-1.1a.6.6 0 01.73.58v7" stroke="currentColor" strokeWidth="1.4" strokeLinecap="round" />
        <circle cx="6.9" cy="12.6" r="2.1" stroke="currentColor" strokeWidth="1.4" />
        <circle cx="14.3" cy="11.8" r="2.1" stroke="currentColor" strokeWidth="1.4" />
      </>
    ),
  },
  {
    id: "presets",
    label: "Пресеты",
    icon: (
      <>
        <path d="M3.5 6.2a1 1 0 011-1h4.2a1 1 0 011 1v10a1 1 0 01-1 1H4.5a1 1 0 01-1-1v-10zM10.9 3.9a1 1 0 011-1h3.6a1 1 0 011 1v12.3a1 1 0 01-1 1h-3.6a1 1 0 01-1-1V3.9z" stroke="currentColor" strokeWidth="1.4" />
      </>
    ),
  },
  {
    id: "profiles",
    label: "Профили",
    icon: (
      <>
        <circle cx="8.5" cy="8" r="2.7" stroke="currentColor" strokeWidth="1.4" />
        <path d="M3.6 15.5c.7-2.5 2.6-3.8 4.9-3.8s4.2 1.3 4.9 3.8" stroke="currentColor" strokeWidth="1.4" strokeLinecap="round" />
        <path d="M14.8 5.2v4M12.8 7.2h4" stroke="currentColor" strokeWidth="1.4" strokeLinecap="round" />
      </>
    ),
  },
  {
    id: "settings",
    label: "Настройки",
    icon: (
      <>
        <circle cx="10" cy="10" r="2.4" stroke="currentColor" strokeWidth="1.4" />
        <path d="M10 2.8v2M10 15.2v2M2.8 10h2M15.2 10h2M4.9 4.9l1.4 1.4M13.7 13.7l1.4 1.4M15.1 4.9l-1.4 1.4M6.3 13.7l-1.4 1.4" stroke="currentColor" strokeWidth="1.4" strokeLinecap="round" />
      </>
    ),
  },
];

export function Sidebar() {
  const page = useApp((s) => s.page);
  const setPage = useApp((s) => s.setPage);
  const appState = useApp((s) => s.appState);

  return (
    <aside className="sidebar">
      {/* Логотип */}
      <div className="sidebar__brand">
        <div className="sidebar__logo">
          {/* Та же иконка, что у ярлыка и в трее (v1.2). */}
          <img src={logoIcon} alt="Larpscord" width={22} height={22} draggable={false} />
        </div>
        <div className="sidebar__title">Larpscord</div>
      </div>

      {/* Навигация */}
      <nav className="sidebar__nav">
        {NAV.map((n) => (
          <button
            key={n.id}
            className={clsx("nav-item", page === n.id && "nav-item--active")}
            onClick={() => setPage(n.id)}
          >
            <svg width="18" height="18" viewBox="0 0 20 20" fill="none">
              {n.icon}
            </svg>
            <span>{n.label}</span>
          </button>
        ))}
      </nav>

      {/* Низ: статус Discord */}
      <div className="sidebar__footer">
        <ConnectionChip />
        <div className="sidebar__meta">
          <span>v{appState?.appVersion ?? "1.0.0"}</span>
          <button className="sidebar__gear" title="Настройки" onClick={() => setPage("settings")}>
            <svg width="14" height="14" viewBox="0 0 20 20" fill="none">
              <circle cx="10" cy="10" r="2.4" stroke="currentColor" strokeWidth="1.5" />
              <path d="M10 2.8v2M10 15.2v2M2.8 10h2M15.2 10h2M4.9 4.9l1.4 1.4M13.7 13.7l1.4 1.4M15.1 4.9l-1.4 1.4M6.3 13.7l-1.4 1.4" stroke="currentColor" strokeWidth="1.5" strokeLinecap="round" />
            </svg>
          </button>
        </div>
      </div>
    </aside>
  );
}

function ConnectionChip() {
  const conn = useApp((s) => s.appState?.connection);
  const connected = conn?.connected ?? false;
  const flavorName =
    conn?.flavor === "canary" ? "Canary" : conn?.flavor === "ptb" ? "PTB" : conn?.flavor === "stable" ? "" : "";
  return (
    <div className={clsx("conn-chip", connected && "conn-chip--on")} title={conn?.last_error ?? undefined}>
      <span className={clsx("conn-chip__dot", connected && "conn-chip__dot--on")} />
      <span className="conn-chip__text">
        {connected ? `Discord${flavorName ? " " + flavorName : ""} подключен` : "Discord не подключен"}
      </span>
    </div>
  );
}
