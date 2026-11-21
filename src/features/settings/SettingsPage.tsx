// Настройки: подключение, поведение, данные.

import { useApp } from "@/stores/app";
import { Section, Toggle, TextInput, Button, Field } from "@/components/ui/primitives";
import { useEffect, useState } from "react";
import type { Settings } from "@/types";

export function SettingsPage() {
  const appState = useApp((s) => s.appState);
  const saveSettings = useApp((s) => s.saveSettings);
  const toast = useApp((s) => s.toast);
  const [s, setS] = useState<Settings | null>(appState?.settings ?? null);
  const [cid, setCid] = useState(appState?.settings.custom_client_id ?? "");

  useEffect(() => {
    if (appState?.settings && !s) {
      setS(appState.settings);
      setCid(appState.settings.custom_client_id ?? "");
    }
  }, [appState, s]);

  if (!s || !appState) return <div className="page"><div className="empty-state">Загрузка…</div></div>;

  const dirty =
    JSON.stringify({ ...s, custom_client_id: cid }) !==
    JSON.stringify(appState.settings);

  return (
    <div className="page settings-page">
      <div className="page-head">
        <div>
          <h1 className="page-title">Настройки</h1>
          <p className="page-sub">Поведение приложения и подключения</p>
        </div>
        {dirty && (
          <Button variant="primary" onClick={() => { setS({ ...s, custom_client_id: cid || null }); void saveSettings({ ...s, custom_client_id: cid || null }); }}>
            Сохранить
          </Button>
        )}
      </div>

      <div className="settings-scroll">
        <Section title="Подключение">
          <div className="set-row">
            <div className="set-row__info">
              <div className="set-row__name">Подключаться к Discord автоматически</div>
              <div className="set-row__desc">Larpscord сам найдёт запущенный Discord при старте</div>
            </div>
            <Toggle checked={s.autoconnect} onChange={(v) => setS({ ...s, autoconnect: v })} />
          </div>
          <div className="set-row">
            <div className="set-row__info">
              <div className="set-row__name">Восстанавливать активность при запуске</div>
              <div className="set-row__desc">После старта приложения снова применить последнюю активность</div>
            </div>
            <Toggle checked={s.restore_activity} onChange={(v) => setS({ ...s, restore_activity: v })} />
          </div>
          <div className="set-row">
            <div className="set-row__info">
              <div className="set-row__name">Применять заново после переподключения</div>
              <div className="set-row__desc">Если Discord перезапустился — активность вернётся автоматически</div>
            </div>
            <Toggle checked={s.auto_reapply} onChange={(v) => setS({ ...s, auto_reapply: v })} />
          </div>
          <div className="set-row set-row--col">
            <Field
              label="Свой Application ID (необязательно)"
              hint="По умолчанию используется встроенный ID. Своё приложение Discord Developer Portal позволяет добавлять свои ассеты и имя."
            >
              <TextInput
                value={cid}
                placeholder={appState.builtinClientId}
                onChange={(e) => setCid(e.target.value)}
              />
            </Field>
          </div>
        </Section>

        <Section title="Окно">
          <div className="set-row">
            <div className="set-row__info">
              <div className="set-row__name">Сворачивать в трей</div>
              <div className="set-row__desc">Закрытие окна скроет приложение в область уведомлений</div>
            </div>
            <Toggle checked={s.minimize_to_tray} onChange={(v) => setS({ ...s, minimize_to_tray: v })} />
          </div>
          <div className="set-row">
            <div className="set-row__info">
              <div className="set-row__name">Запускать свёрнутым</div>
              <div className="set-row__desc">При старте Windows окно не открывается</div>
            </div>
            <Toggle checked={s.start_minimized} onChange={(v) => setS({ ...s, start_minimized: v })} />
          </div>
          <div className="set-row">
            <div className="set-row__info">
              <div className="set-row__name">Запускать при старте Windows</div>
              <div className="set-row__desc">Larpscord стартует вместе с системой</div>
            </div>
            <Toggle
              checked={s.launch_on_startup}
              onChange={async (v) => {
                try {
                  const { enable, disable } = await import("@tauri-apps/plugin-autostart");
                  if (v) await enable(); else await disable();
                  setS({ ...s, launch_on_startup: v });
                } catch (e) {
                  toast("error", `Автозапуск: ${e}`);
                }
              }}
            />
          </div>
        </Section>

        <Section title="Данные">
          <div className="set-row">
            <div className="set-row__info">
              <div className="set-row__name">Каталог данных</div>
              <div className="set-row__desc">{appState.storageDir}</div>
            </div>
          </div>
        </Section>

        <Section title="О приложении">
          <div className="about-row">
            <div className="about-row__name">Larpscord</div>
            <div className="about-row__version">Версия {appState.appVersion}</div>
            <div className="about-row__text">
              Студия настройки Discord Rich Presence. Подключается к локальному Discord
              через официальный RPC-протокол и не собирает никаких данных.
            </div>
          </div>
        </Section>
      </div>
    </div>
  );
}
