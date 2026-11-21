// Пресеты: встроенные шаблоны + пользовательские (создать/удалить/импорт/экспорт).

import { useApp } from "@/stores/app";
import { ActivityPreviewCard } from "@/components/discord-preview/ActivityCard";
import { Badge, Button } from "@/components/ui/primitives";
import { api } from "@/lib/api";
import type { Preset } from "@/types";
import { useEffect, useState } from "react";

export function PresetsPage() {
  const toast = useApp((s) => s.toast);
  const setDraft = useApp((s) => s.setDraft);
  const setPage = useApp((s) => s.setPage);
  const [presets, setPresets] = useState<Preset[]>([]);
  const [busy, setBusy] = useState(false);

  const reload = async () => {
    try {
      setPresets(await api.listPresets());
    } catch (e) {
      toast("error", String(e));
    }
  };

  useEffect(() => {
    void reload();
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, []);

  const createFromDraft = async () => {
    const draft = useApp.getState().draft;
    const name = prompt("Имя нового пресета:", "Мой пресет");
    if (!name?.trim()) return;
    setBusy(true);
    try {
      await api.savePreset({
        id: `user-${Date.now()}`,
        name: name.trim(),
        category: "Мои пресеты",
        activity: draft,
        builtin: false,
      });
      await reload();
      toast("success", `Пресет «${name}» создан`);
    } catch (e) {
      toast("error", String(e));
    } finally {
      setBusy(false);
    }
  };

  const doExport = async () => {
    try {
      const json = await api.exportPresets();
      const blob = new Blob([json], { type: "application/json" });
      const url = URL.createObjectURL(blob);
      const a = document.createElement("a");
      a.href = url;
      a.download = "larpscord-presets.json";
      a.click();
      URL.revokeObjectURL(url);
      toast("success", "Пресеты экспортированы");
    } catch (e) {
      toast("error", String(e));
    }
  };

  const doImport = async () => {
    const input = document.createElement("input");
    input.type = "file";
    input.accept = ".json,application/json";
    input.onchange = async () => {
      const file = input.files?.[0];
      if (!file) return;
      try {
        const json = await file.text();
        const r = await api.importPresets(json);
        await reload();
        toast("success", `Импортировано: добавлено ${r.added}, обновлено ${r.updated}`);
      } catch (e) {
        toast("error", String(e));
      }
    };
    input.click();
  };

  const grouped = groupBy(presets, (p) => p.category);

  return (
    <div className="page presets-page">
      <div className="page-head">
        <div>
          <h1 className="page-title">Пресеты</h1>
          <p className="page-sub">Готовые шаблоны и ваши сохранённые активности</p>
        </div>
        <div className="page-head__actions">
          <Button variant="primary" size="sm" onClick={() => void createFromDraft()} disabled={busy}>
            Сохранить черновик как пресет
          </Button>
          <Button variant="ghost" size="sm" onClick={() => void doImport()}>
            Импорт
          </Button>
          <Button variant="ghost" size="sm" onClick={() => void doExport()}>
            Экспорт
          </Button>
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
                    {p.builtin ? <Badge tone="muted">встроенный</Badge> : <Badge tone="ok">мой</Badge>}
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
                      Открыть
                    </Button>
                    {!p.builtin && (
                      <Button
                        variant="danger"
                        size="sm"
                        onClick={async () => {
                          if (!confirm(`Удалить пресет «${p.name}»?`)) return;
                          try {
                            await api.deletePreset(p.id);
                            await reload();
                            toast("success", "Пресет удалён");
                          } catch (e) {
                            toast("error", String(e));
                          }
                        }}
                      >
                        Удалить
                      </Button>
                    )}
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
