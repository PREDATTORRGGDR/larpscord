// Реалистичная карточка активности Discord (как в профиле).
// Точная имитация вёрстки Discord: иконка приложения, глагол + имя,
// details (жирным), state, время, картинка с tooltip-ами, кнопки.

import type { Activity } from "@/types";
import { ACTIVITY_TYPES } from "@/types";
import { useEffect, useState } from "react";

export function ActivityPreviewCard({ activity, live }: { activity: Activity; live: boolean }) {
  const [, force] = useState(0);
  useEffect(() => {
    const t = setInterval(() => force((x) => x + 1), 1000);
    return () => clearInterval(t);
  }, []);

  const meta = ACTIVITY_TYPES.find((t) => t.value === activity.type) ?? ACTIVITY_TYPES[0];
  const hasContent =
    !!activity.details?.trim() ||
    !!activity.state?.trim() ||
    !!activity.large_image ||
    !!activity.small_image ||
    activity.timestamps_start != null ||
    activity.timestamps_end != null;

  const elapsed = formatElapsed(activity);
  const appName = activity.name?.trim() || "Larpscord";

  return (
    <div className="dc-card">
      <div className="dc-card__head">
        <div className="dc-card__icon">
          <svg width="22" height="22" viewBox="0 0 20 20" fill="none">
            <rect x="2" y="2" width="16" height="16" rx="4.5" fill="#15161b" stroke="rgba(255,255,255,0.16)" />
            <path d="M7.6 5.6h2.1v7.2h4.1v1.7H7.6V5.6z" fill="#f2f3f5" />
          </svg>
        </div>
        <div className="dc-card__heading">
          <div className="dc-card__verb">{meta.verb}</div>
          <div className="dc-card__app">{appName}</div>
        </div>
        {live && <span className="dc-card__live" title="Эта активность сейчас применена в Discord">● live</span>}
      </div>

      {hasContent ? (
        <div className="dc-card__body">
          {(activity.details?.trim() || activity.large_image) && (
            <div className="dc-card__row">
              {activity.large_image && (
                <div className="dc-asset" title={activity.large_text || undefined}>
                  <img src={normalizeImage(activity.large_image)} alt="" onError={(e) => ((e.target as HTMLImageElement).style.visibility = "hidden")} />
                  {activity.small_image && (
                    <img
                      className="dc-asset__small"
                      src={normalizeImage(activity.small_image)}
                      alt=""
                      title={activity.small_text || undefined}
                    />
                  )}
                </div>
              )}
              <div className="dc-card__lines">
                {activity.details?.trim() && <div className="dc-card__details">{activity.details}</div>}
                {activity.state?.trim() && <div className="dc-card__state">{activity.state}</div>}
                {elapsed && <div className="dc-card__time">{elapsed}</div>}
              </div>
            </div>
          )}
          {activity.buttons.filter((b) => b.label.trim()).length > 0 && (
            <div className="dc-card__buttons">
              {activity.buttons
                .filter((b) => b.label.trim())
                .slice(0, 2)
                .map((b, i) => (
                  <span key={i} className="dc-btn" title={b.url}>
                    {b.label}
                  </span>
                ))}
            </div>
          )}
        </div>
      ) : (
        <div className="dc-card__empty">
          Пустая активность — добавьте детали, статус или картинку
        </div>
      )}
    </div>
  );
}

function normalizeImage(src: string): string {
  // mp:external/... — уже проксировано Discord, во фронтенде не показываем.
  const s = src.trim();
  if (s.startsWith("http://") || s.startsWith("https://")) return s;
  return "";
}

function formatElapsed(a: Activity): string | null {
  const now = Math.floor(Date.now() / 1000);
  if (a.timestamps_start && a.timestamps_end) {
    const left = a.timestamps_end - now;
    if (left > 0) return `${formatDur(left)} осталось`;
    return `${formatDur(-left)} просрочено`;
  }
  if (a.timestamps_start) {
    const el = now - a.timestamps_start;
    return el >= 0 ? `${formatDur(el)} прошло` : null;
  }
  if (a.timestamps_end) {
    const left = a.timestamps_end - now;
    return left > 0 ? `${formatDur(left)} осталось` : null;
  }
  return null;
}

export function formatDur(sec: number): string {
  const d = Math.floor(sec / 86400);
  const h = Math.floor((sec % 86400) / 3600);
  const m = Math.floor((sec % 3600) / 60);
  const s = sec % 60;
  const pad = (n: number) => String(n).padStart(2, "0");
  if (d > 0) return `${d}д ${pad(h)}:${pad(m)}:${pad(s)}`;
  if (h > 0) return `${h}:${pad(m)}:${pad(s)}`;
  return `${pad(m)}:${pad(s)}`;
}
