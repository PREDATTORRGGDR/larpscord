// Профили: быстрые наборы активностей. Полное управление — Этап 2;
// сейчас профиль = последний черновик + активная активность.

import { useApp } from "@/stores/app";
import { ActivityPreviewCard } from "@/components/discord-preview/ActivityCard";
import { Badge, Button } from "@/components/ui/primitives";

export function ProfilesPage() {
  const appState = useApp((s) => s.appState);
  const draft = useApp((s) => s.draft);
  const setDraft = useApp((s) => s.setDraft);
  const setPage = useApp((s) => s.setPage);
  const applyDraft = useApp((s) => s.applyDraft);
  const clearPresence = useApp((s) => s.clearPresence);

  const last = appState?.lastActivity;
  const active = appState?.activeActivity;

  return (
    <div className="page profiles-page">
      <div className="page-head">
        <div>
          <h1 className="page-title">Профили</h1>
          <p className="page-sub">Быстрое переключение между активностями</p>
        </div>
      </div>

      <div className="presets-scroll">
        <div className="preset-group">
          <div className="preset-group__title">Текущий черновик</div>
          <div className="preset-grid">
            <div className="preset-card glass">
              <div className="preset-card__top">
                <div className="preset-card__name">Черновик</div>
                <Badge tone="muted">редактируется</Badge>
              </div>
              <div className="dc-card__wrap">
                <ActivityPreviewCard activity={draft} live={false} />
              </div>
              <div className="preset-card__actions">
                <Button variant="subtle" size="sm" onClick={() => setPage("editor")}>
                  Редактор
                </Button>
                <Button variant="primary" size="sm" onClick={() => void applyDraft()}>
                  Применить
                </Button>
              </div>
            </div>

            {last && (
              <div className="preset-card glass">
                <div className="preset-card__top">
                  <div className="preset-card__name">Последняя сохранённая</div>
                  <Badge tone="muted">автосохранение</Badge>
                </div>
                <div className="dc-card__wrap">
                  <ActivityPreviewCard activity={last} live={false} />
                </div>
                <div className="preset-card__actions">
                  <Button variant="subtle" size="sm" onClick={() => { setDraft({ ...last }); setPage("editor"); }}>
                    Загрузить
                  </Button>
                </div>
              </div>
            )}

            {active && (
              <div className="preset-card glass">
                <div className="preset-card__top">
                  <div className="preset-card__name">Активная сейчас</div>
                  <Badge tone="ok">в Discord</Badge>
                </div>
                <div className="dc-card__wrap">
                  <ActivityPreviewCard activity={active} live />
                </div>
                <div className="preset-card__actions">
                  <Button variant="danger" size="sm" onClick={() => void clearPresence()}>
                    Сбросить
                  </Button>
                </div>
              </div>
            )}
          </div>
        </div>

        <div className="music-note glass">
          <div className="music-note__title">Несколько профилей</div>
          <div className="music-note__text">
            Создание, дублирование и импорт/экспорт отдельных профилей с правилами
            автопереключения — в следующем обновлении. Сейчас доступны черновик,
            последняя сохранённая и активная активности.
          </div>
        </div>
      </div>
    </div>
  );
}
