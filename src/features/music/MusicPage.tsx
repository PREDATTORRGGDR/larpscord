// Музыка: ручной редактор + автоопределение через Windows Media Session (SMTC).

import { useApp } from "@/stores/app";
import { ActivityPreviewCard, formatDur } from "@/components/discord-preview/ActivityCard";
import { Field, TextInput, Button, Section, Badge, Toggle } from "@/components/ui/primitives";
import { api, type MediaSnapshot } from "@/lib/api";
import { useEffect, useState } from "react";
import { clsx } from "clsx";

export function MusicPage() {
  const draft = useApp((s) => s.draft);
  const setDraft = useApp((s) => s.setDraft);
  const applyDraft = useApp((s) => s.applyDraft);
  const appState = useApp((s) => s.appState);
  const toast = useApp((s) => s.toast);

  const [mode, setMode] = useState<"off" | "manual" | "auto">("off");
  const [snap, setSnap] = useState<MediaSnapshot | null>(null);
  const [cover, setCover] = useState<string | null>(null);
  const [busy, setBusy] = useState(false);

  const refreshState = async () => {
    try {
      const st = await api.musicState();
      setMode(st.mode as "off" | "manual" | "auto");
      setSnap(st.snapshot);
      setCover(st.cover);
    } catch {
      /* ignore */
    }
  };

  useEffect(() => {
    void refreshState();
    const t = setInterval(() => void refreshState(), 2000);
    return () => clearInterval(t);
  }, []);

  const enableAuto = async (on: boolean) => {
    setBusy(true);
    try {
      await api.musicSetMode(on ? "auto" : "off");
      setMode(on ? "auto" : "off");
      if (on) {
        const s = await api.musicPoll();
        setSnap(s);
        toast("success", s.available ? `Найден плеер: ${s.source}` : "Плеер не найден — включите музыку");
      }
    } catch (e) {
      toast("error", String(e));
    } finally {
      setBusy(false);
    }
  };

  const importSnapshot = () => {
    if (!snap?.title) return;
    setDraft({
      ...draft,
      type: "Listening",
      details: snap.title,
      state: snap.artist ?? "",
      large_text: snap.album ?? null,
      large_image: cover ?? draft.large_image ?? null,
      timestamps_start: Math.floor(Date.now() / 1000),
      timestamps_end: null,
    });
  };

  const track = draft.details ?? "";
  const artist = draft.state ?? "";
  const isLive = !!appState?.activeActivity && appState.activeActivity.type === "Listening";

  return (
    <div className="page editor-page">
      <div className="editor-left glass">
        <div className="editor-left__head">
          <h1 className="page-title">Музыкальная активность</h1>
          <p className="page-sub">Статус «Слушает» — вручную или автоматически из плеера</p>
        </div>

        <div className="editor-scroll">
          {/* Автоопределение */}
          <div className={clsx("auto-music glass", mode === "auto" && "auto-music--on")}>
            <div className="auto-music__head">
              <div className="auto-music__icon">
                <svg width="18" height="18" viewBox="0 0 20 20" fill="none">
                  <path d="M9 12.5V5.8a.6.6 0 01.47-.58l4.4-1.1a.6.6 0 01.73.58v7" stroke="currentColor" strokeWidth="1.4" strokeLinecap="round" />
                  <circle cx="6.9" cy="12.6" r="2.1" stroke="currentColor" strokeWidth="1.4" />
                  <circle cx="14.3" cy="11.8" r="2.1" stroke="currentColor" strokeWidth="1.4" />
                </svg>
              </div>
              <div className="auto-music__titles">
                <div className="auto-music__title">Автоопределение плеера</div>
                <div className="auto-music__sub">
                  {snap?.available
                    ? `${snap.source}: ${snap.playing ? "воспроизводится" : "на паузе"}`
                    : "Windows Media Session: Spotify, Chrome, Edge, VLC и другие"}
                </div>
              </div>
              <Toggle checked={mode === "auto"} onChange={(v) => void enableAuto(v)} disabled={busy} />
            </div>
            {mode === "auto" && (
              <div className="auto-music__track">
                {cover && <img className="auto-music__cover" src={cover} alt="" />}
                <div className="auto-music__meta">
                  <div className="auto-music__name">{snap?.title ?? "—"}</div>
                  <div className="auto-music__artist">{snap?.artist ?? ""}</div>
                  {snap?.positionSecs != null && snap?.durationSecs != null && (
                    <div className="auto-music__time">
                      {formatDur(Math.floor(snap.positionSecs))} / {formatDur(Math.floor(snap.durationSecs))}
                    </div>
                  )}
                </div>
                <Button size="sm" variant="subtle" onClick={importSnapshot} disabled={!snap?.title}>
                  В редактор
                </Button>
              </div>
            )}
          </div>

          {/* Ручной режим */}
          <Section title="Ручной ввод">
            <Field label="Название трека" max={128} value={track}>
              <TextInput value={track} maxLength={128} placeholder="Название" onChange={(e) => setDraft({ ...draft, details: e.target.value, type: "Listening" })} />
            </Field>
            <Field label="Исполнитель" max={128} value={artist}>
              <TextInput value={artist} maxLength={128} placeholder="Исполнитель" onChange={(e) => setDraft({ ...draft, state: e.target.value })} />
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

          <Section title="Время" defaultOpen={false}>
            <div className="quick-row">
              <Button size="sm" onClick={() => setDraft({ ...draft, timestamps_start: Math.floor(Date.now() / 1000), timestamps_end: null })}>
                Начать отсчёт
              </Button>
              <Button size="sm" onClick={() => setDraft({ ...draft, timestamps_start: null, timestamps_end: null })}>
                Убрать
              </Button>
            </div>
          </Section>
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
