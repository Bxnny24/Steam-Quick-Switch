import { useState } from "react";
import { useTranslation } from "react-i18next";
import type { Account } from "../lib/api";
import {
  Language,
  NameMode,
  Settings,
  setAutostart,
  setLanguage,
  setNameMode,
  setNickname,
} from "../lib/settings";
import i18n from "../i18n";

interface Props {
  accounts: Account[];
  settings: Settings;
  /** Called after any change so the parent can reload settings. */
  onChanged: () => void;
  onClose: () => void;
}

export function SettingsView({ accounts, settings, onChanged, onClose }: Props) {
  const { t } = useTranslation();
  const [busy, setBusy] = useState(false);

  async function changeLanguage(language: Language) {
    await setLanguage(language);
    await i18n.changeLanguage(language);
    onChanged();
  }

  async function changeNameMode(mode: NameMode) {
    await setNameMode(mode);
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

  async function changeNickname(steamId64: string, value: string) {
    await setNickname(steamId64, value);
    onChanged();
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

          <label className="settings__row">
            <span>{t("settings.displayName")}</span>
            <select
              value={settings.nameMode}
              onChange={(e) => changeNameMode(e.target.value as NameMode)}
            >
              <option value="persona">{t("settings.nameProfile")}</option>
              <option value="account">{t("settings.nameAccount")}</option>
            </select>
          </label>
        </section>

        <section className="settings__group">
          <h2 className="settings__heading">{t("settings.nicknames")}</h2>
          {accounts.map((account) => (
            <label className="settings__row" key={account.steamId64}>
              <span className="settings__account">
                {account.personaName.trim() || account.accountName}
              </span>
              <input
                type="text"
                placeholder={t("settings.nicknamePlaceholder")}
                defaultValue={settings.nicknames[account.steamId64] ?? ""}
                onBlur={(e) => changeNickname(account.steamId64, e.target.value)}
              />
            </label>
          ))}
        </section>
      </div>
    </div>
  );
}
