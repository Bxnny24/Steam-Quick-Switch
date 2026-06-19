import { useState } from "react";
import { useTranslation } from "react-i18next";
import { Language, Settings, setAutostart, setLanguage } from "../lib/settings";
import i18n from "../i18n";

interface Props {
  settings: Settings;
  /** Called after any change so the parent can reload settings. */
  onChanged: () => void;
  onClose: () => void;
}

export function SettingsView({ settings, onChanged, onClose }: Props) {
  const { t } = useTranslation();
  const [busy, setBusy] = useState(false);

  async function changeLanguage(language: Language) {
    await setLanguage(language);
    await i18n.changeLanguage(language);
    onChanged();
  }

  async function toggleAutostart(enabled: boolean) {
    setBusy(true);
    try {
      await setAutostart(enabled);
      onChanged();
    } finally {
      setBusy(false);
    }
  }

  return (
    <div className="app">
      <header className="app__header">
        <button className="icon-button" onClick={onClose} title={t("settings.back")}>
          ←
        </button>
        <h1 className="app__title">{t("settings.title")}</h1>
        <span className="icon-button icon-button--ghost" aria-hidden="true" />
      </header>

      <div className="settings">
        <section className="settings__group">
          <label className="settings__row">
            <span>{t("settings.language")}</span>
            <select
              value={settings.language}
              onChange={(e) => changeLanguage(e.target.value as Language)}
            >
              <option value="en">English</option>
              <option value="de">Deutsch</option>
            </select>
          </label>

          <label className="settings__row">
            <span>{t("settings.autostart")}</span>
            <input
              type="checkbox"
              checked={settings.autostart}
              disabled={busy}
              onChange={(e) => toggleAutostart(e.target.checked)}
            />
          </label>
        </section>
      </div>
    </div>
  );
}
