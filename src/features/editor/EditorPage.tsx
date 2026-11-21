// Редактор Rich Presence: слева — управление, справа — живой предпросмотр.

import { useApp } from "@/stores/app";
import { ActivityPreviewCard } from "@/components/discord-preview/ActivityCard";
import { Section, Field, TextInput, Button, Badge } from "@/components/ui/primitives";
import { ACTIVITY_TYPES } from "@/types";
import type { Activity, ActivityType } from "@/types";
import { clsx } from "clsx";

export function EditorPage() {
  const draft = useApp((s) => s.draft);
  const setDraft = useApp((s) => s.setDraft);
  const applyDraft = useApp((s) => s.applyDraft);
  const clearPresence = useApp((s) => s.clearPresence);
  const appState = useApp((s) => s.appState);

  const update = (patch: Partial<Activity>) =>
    setDraft({ ...draft, ...patch });

  const isLive =
    !!appState?.activeActivity &&
    JSON.stringify(normalizeForCompare(appState.activeActivity)) ===
      JSON.stringify(normalizeForCompare(draft));

  return (
    <div className="page editor-page">
      {/* ЛЕВАЯ ПАНЕЛЬ */}
      <div className="editor-left glass">
        <div className="editor-left__head">
          <h1 className="page-title">Редактор активности</h1>
          <p className="page-sub">Настройте, как вас видят в Discord</p>
        </div>

        <div className="editor-scroll">
          <Section title="Основное" desc="Текст активности и её тип">
            <Field label="Тип активности">
              <div className="seg">
                {ACTIVITY_TYPES.map((t) => (
                  <button
                    key={t.value}
                    className={clsx("seg__item", draft.type === t.value && "seg__item--on")}
                    onClick={() => update({ type: t.value as ActivityType })}
                  >
                    {t.label}
                  </button>
                ))}
              </div>
            </Field>

            <Field label="Имя приложения" hint="Отображается сверху карточки. Если пусто — «Larpscord»" max={64} value={draft.name ?? ""}>
              <TextInput
                value={draft.name ?? ""}
                maxLength={64}
                placeholder="Например: Visual Studio Code"
                onChange={(e) => update({ name: e.target.value })}
              />
            </Field>

            <Field label="Детали" hint="Первая строка — крупным шрифтом" max={128} value={draft.details ?? ""}>
              <TextInput
                value={draft.details ?? ""}
                maxLength={128}
                placeholder="Что вы делаете"
                onChange={(e) => update({ details: e.target.value })}
              />
            </Field>

            <Field label="Состояние" hint="Вторая строка" max={128} value={draft.state ?? ""}>
              <TextInput
                value={draft.state ?? ""}
                maxLength={128}
                placeholder="Дополнительная информация"
                onChange={(e) => update({ state: e.target.value })}
              />
            </Field>
          </Section>

          <Section title="Время" desc="Отсчёт «прошло» или «осталось»" defaultOpen={false}>
            <Field label="Начало отсчёта">
              <TextInput
                type="datetime-local"
                value={toLocalInput(draft.timestamps_start)}
                onChange={(e) =>
                  update({
                    timestamps_start: e.target.value
                      ? Math.floor(new Date(e.target.value).getTime() / 1000)
                      : null,
                  })
                }
              />
            </Field>
            <Field label="Окончание (для «осталось»)">
              <TextInput
                type="datetime-local"
                value={toLocalInput(draft.timestamps_end)}
                onChange={(e) =>
                  update({
                    timestamps_end: e.target.value
                      ? Math.floor(new Date(e.target.value).getTime() / 1000)
                      : null,
                  })
                }
              />
            </Field>
            <div className="quick-row">
              <Button size="sm" onClick={() => update({ timestamps_start: nowSec(), timestamps_end: null })}>
                Считать с сейчас
              </Button>
              <Button
                size="sm"
                onClick={() =>
                  update({
                    timestamps_start: nowSec(),
                    timestamps_end: nowSec() + 3600,
                  })
                }
              >
                Часовой таймер
              </Button>
              <Button size="sm" onClick={() => update({ timestamps_start: null, timestamps_end: null })}>
                Убрать время
              </Button>
            </div>
          </Section>

          <Section title="Картинки" desc="Внешние URL (Discord проксирует их автоматически)" defaultOpen={false}>
            <Field label="Большая картинка (URL)" hint="PNG/JPEG/WebP по https-ссылке">
              <TextInput
                value={draft.large_image ?? ""}
                placeholder="https://example.com/cover.png"
                onChange={(e) => update({ large_image: e.target.value || null })}
              />
            </Field>
            <Field label="Подсказка (tooltip) большой" max={128} value={draft.large_text ?? ""}>
              <TextInput
                value={draft.large_text ?? ""}
                maxLength={128}
                placeholder="Текст при наведении"
                onChange={(e) => update({ large_text: e.target.value || null })}
              />
            </Field>
            <Field label="Маленькая картинка (URL)">
              <TextInput
                value={draft.small_image ?? ""}
                placeholder="https://example.com/icon.png"
                onChange={(e) => update({ small_image: e.target.value || null })}
              />
            </Field>
            <Field label="Подсказка маленькой" max={128} value={draft.small_text ?? ""}>
              <TextInput
                value={draft.small_text ?? ""}
                maxLength={128}
                placeholder="Текст при наведении"
                onChange={(e) => update({ small_text: e.target.value || null })}
              />
            </Field>
          </Section>

          <Section title="Кнопки" desc="До двух ссылок под активностью" defaultOpen={false}>
            {(draft.buttons ?? []).map((b, i) => (
              <div className="btn-editor" key={i}>
                <div className="btn-editor__row">
                  <Field label={`Метка ${i + 1}`} max={32} value={b.label}>
                    <TextInput
                      value={b.label}
                      maxLength={32}
                      placeholder="GitHub"
                      onChange={(e) => {
                        const buttons = [...draft.buttons];
                        buttons[i] = { ...b, label: e.target.value };
                        update({ buttons });
                      }}
                    />
                  </Field>
                  <Field label="URL" value={b.url}>
                    <TextInput
                      value={b.url}
                      placeholder="https://github.com"
                      onChange={(e) => {
                        const buttons = [...draft.buttons];
                        buttons[i] = { ...b, url: e.target.value };
                        update({ buttons });
                      }}
                    />
                  </Field>
                  <button
                    className="btn-editor__del"
                    title="Удалить кнопку"
                    onClick={() => update({ buttons: draft.buttons.filter((_, j) => j !== i) })}
                  >
                    ✕
                  </button>
                </div>
                {!b.url.startsWith("http") && b.label.trim() !== "" && (
                  <div className="field__hint field__hint--err">
                    URL должен начинаться с http:// или https:// — иначе Discord не покажет кнопку
                  </div>
                )}
              </div>
            ))}
            {(draft.buttons?.length ?? 0) < 2 && (
              <Button
                variant="subtle"
                size="sm"
                onClick={() => update({ buttons: [...(draft.buttons ?? []), { label: "", url: "" }] })}
              >
                + Добавить кнопку
              </Button>
            )}
          </Section>
        </div>

        {/* Действия */}
        <div className="editor-actions">
          <Button variant="primary" onClick={() => void applyDraft()} disabled={useApp.getState().loading}>
            Применить в Discord
          </Button>
          <Button variant="ghost" onClick={() => void clearPresence()}>
            Сбросить
          </Button>
        </div>
      </div>

      {/* ПРАВАЯ ПАНЕЛЬ — предпросмотр */}
      <div className="editor-right">
        <div className="preview-stack">
          <div className="preview-label">
            <span>Предпросмотр карточки</span>
            {isLive ? (
              <Badge tone="ok">применено</Badge>
            ) : (
              <Badge tone="muted">локальный черновик</Badge>
            )}
          </div>

          {/* Профиль пользователя (статичная имитация шапки) */}
          <div className="profile-card glass">
            <div className="profile-card__banner" />
            <div className="profile-card__inner">
              <div className="profile-card__avatar">
                <svg width="44" height="44" viewBox="0 0 44 44" fill="none">
                  <circle cx="22" cy="22" r="21" fill="#1b1c21" stroke="rgba(255,255,255,0.18)" />
                  <circle cx="22" cy="17" r="6.5" fill="#85858e" />
                  <path d="M10 37c1.8-6.5 6.6-9.8 12-9.8S38.2 30.5 40 37" fill="#85858e" />
                </svg>
              </div>
              <div className="profile-card__name">
                {appState?.connection?.user?.globalName ??
                  appState?.connection?.user?.username ??
                  "Ваш профиль"}
              </div>
              <div className="dc-card__wrap">
                <ActivityPreviewCard activity={draft} live={isLive} />
              </div>
            </div>
          </div>

          <div className="preview-note">
            {isLive
              ? "Эта активность сейчас отображается в вашем Discord."
              : "Черновик ещё не применён — нажмите «Применить в Discord»."}
          </div>
        </div>
      </div>
    </div>
  );
}

function nowSec() {
  return Math.floor(Date.now() / 1000);
}

function toLocalInput(sec: number | null | undefined): string {
  if (!sec) return "";
  const d = new Date(sec * 1000);
  const pad = (n: number) => String(n).padStart(2, "0");
  return `${d.getFullYear()}-${pad(d.getMonth() + 1)}-${pad(d.getDate())}T${pad(d.getHours())}:${pad(d.getMinutes())}`;
}

function normalizeForCompare(a: Activity): Activity {
  return {
    ...a,
    name: a.name || null,
    details: a.details || null,
    state: a.state || null,
    large_image: a.large_image || null,
    large_text: a.large_text || null,
    small_image: a.small_image || null,
    small_text: a.small_text || null,
  };
}
