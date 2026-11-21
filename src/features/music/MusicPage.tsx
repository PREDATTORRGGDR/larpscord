// Музыка: ручной редактор музыкальной активности (Этап 2 — автоинтеграции).

import { useApp } from "@/stores/app";
import { ActivityPreviewCard } from "@/components/discord-preview/ActivityCard";
import { Field, TextInput, Button, Section, Badge } from "@/components/ui/primitives";

export function MusicPage() {
  const draft = useApp((s) => s.draft);
  const setDraft = useApp((s) => s.setDraft);
  const applyDraft = useApp((s) => s.applyDraft);
  const appState = useApp((s) => s.appState);

  const track = draft.details ?? "";
  const artist = draft.state ?? "";
  const setTrack = (v: string) => setDraft({ ...draft, details: v, type: "Listening" });
  const setArtist = (v: string) => setDraft({ ...draft, state: v, type: "Listening" });

  const isLive =
    !!appState?.activeActivity &&
    appState.activeActivity.type === "Listening" &&
    appState.activeActivity.details === draft.details;

  return (
    <div className="page editor-page">
      <div className="editor-left glass">
        <div className="editor-left__head">
          <h1 className="page-title">Музыкальная активность</h1>
          <p className="page-sub">Статус «Слушает» с треком, исполнителем и обложкой</p>
        </div>

        <div className="editor-scroll">
          <Section title="Трек">
            <Field label="Название трека" max={128} value={track}>
              <TextInput value={track} maxLength={128} placeholder="Название" onChange={(e) => setTrack(e.target.value)} />
            </Field>
            <Field label="Исполнитель" max={128} value={artist}>
              <TextInput value={artist} maxLength={128} placeholder="Исполнитель" onChange={(e) => setArtist(e.target.value)} />
            </Field>
            <Field label="Обложка (URL)" hint="Discord сам скачает и проксирует картинку">
              <TextInput
                value={draft.large_image ?? ""}
                placeholder="https://example.com/cover.jpg"
                onChange={(e) => setDraft({ ...draft, large_image: e.target.value || null })}
              />
            </Field>
            <Field label="Альбом (tooltip обложки)" max={128} value={draft.large_text ?? ""}>
              <TextInput
                value={draft.large_text ?? ""}
                maxLength={128}
                placeholder="Название альбома"
                onChange={(e) => setDraft({ ...draft, large_text: e.target.value || null })}
              />
            </Field>
          </Section>

          <Section title="Время" desc="Сколько уже слушаете">
            <div className="quick-row">
              <Button
                size="sm"
                onClick={() => setDraft({ ...draft, timestamps_start: Math.floor(Date.now() / 1000), timestamps_end: null })}
              >
                Начать отсчёт
              </Button>
              <Button size="sm" onClick={() => setDraft({ ...draft, timestamps_start: null, timestamps_end: null })}>
                Убрать
              </Button>
            </div>
          </Section>

          <div className="music-note glass">
            <div className="music-note__title">Автоопределение плееров</div>
            <div className="music-note__text">
              Автоматическое определение играющей музыки (Spotify, браузеры, локальные плееры
              через Windows Media Session) появится в следующем обновлении Larpscord.
            </div>
          </div>
        </div>

        <div className="editor-actions">
          <Button variant="primary" onClick={() => void applyDraft()}>
            Применить в Discord
          </Button>
        </div>
      </div>

      <div className="editor-right">
        <div className="preview-stack">
          <div className="preview-label">
            <span>Предпросмотр</span>
            {isLive ? <Badge tone="ok">применено</Badge> : <Badge tone="muted">черновик</Badge>}
          </div>
          <div className="profile-card glass">
            <div className="profile-card__banner" />
            <div className="profile-card__inner">
              <div className="profile-card__name">Слушает сейчас</div>
              <div className="dc-card__wrap">
                <ActivityPreviewCard activity={{ ...draft, type: "Listening" }} live={isLive} />
              </div>
            </div>
          </div>
        </div>
      </div>
    </div>
  );
}
