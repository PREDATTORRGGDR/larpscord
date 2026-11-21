// Пресеты: встроенные шаблоны активностей. (Пользовательские — Этап 2.)

import { useApp } from "@/stores/app";
import { ActivityPreviewCard } from "@/components/discord-preview/ActivityCard";
import { Badge, Button } from "@/components/ui/primitives";
import { useEffect } from "react";

export function PresetsPage() {
  const presets = useApp((s) => s.presets);
  const loadPresets = useApp((s) => s.loadPresets);
  const setDraft = useApp((s) => s.setDraft);
  const setPage = useApp((s) => s.setPage);

  useEffect(() => {
    void loadPresets();
  }, [loadPresets]);

  const grouped = groupBy(presets, (p) => p.category);

  return (
    <div className="page presets-page">
      <div className="page-head">
        <div>
          <h1 className="page-title">Пресеты</h1>
          <p className="page-sub">Готовые шаблоны — нажмите, чтобы открыть в редакторе</p>
        </div>
      </div>

      <div className="presets-scroll">
        {Object.entries(grouped).map(([cat, items]) => (
          <div key={cat} className="preset-group">
            <div className="preset-group__title">{cat}</div>
            <div className="preset-grid">
              {items.map((p) => (
                <div key={p.id} className="preset-card glass">
                  <div className="preset-card__top">
                    <div className="preset-card__name">{p.name}</div>
                    {p.builtin ? <Badge tone="muted">встроенный</Badge> : null}
                  </div>
                  <div className="dc-card__wrap">
                    <ActivityPreviewCard activity={p.activity} live={false} />
                  </div>
                  <div className="preset-card__actions">
                    <Button
                      variant="subtle"
                      size="sm"
                      onClick={() => {
                        setDraft({ ...p.activity });
                        setPage("editor");
                      }}
                    >
                      Открыть в редакторе
                    </Button>
                  </div>
                </div>
              ))}
            </div>
          </div>
        ))}
        {presets.length === 0 && <div className="empty-state">Пресеты загружаются…</div>}
      </div>
    </div>
  );
}

function groupBy<T>(arr: T[], key: (t: T) => string): Record<string, T[]> {
  const out: Record<string, T[]> = {};
  for (const item of arr) {
    const k = key(item);
    (out[k] ??= []).push(item);
  }
  return out;
}
