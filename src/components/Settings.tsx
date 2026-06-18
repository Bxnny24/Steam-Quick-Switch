import { useState } from "react";
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

interface Props {
  accounts: Account[];
  settings: Settings;
  /** Called after any change so the parent can reload settings. */
  onChanged: () => void;
  onClose: () => void;
}

export function SettingsView({ accounts, settings, onChanged, onClose }: Props) {
  const [busy, setBusy] = useState(false);

  async function changeLanguage(language: Language) {
    await setLanguage(language);
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
        <button className="icon-button" onClick={onClose} title="Back">
          ←
        </button>
        <h1 className="app__title">Settings</h1>
        <span className="icon-button icon-button--ghost" aria-hidden="true" />
      </header>

      <div className="settings">
        <section className="settings__group">
          <label className="settings__row">
            <span>Language</span>
            <select
              value={settings.language}
              onChange={(e) => changeLanguage(e.target.value as Language)}
            >
              <option value="en">English</option>
              <option value="de">Deutsch</option>
            </select>
          </label>

          <label className="settings__row">
            <span>Start with Windows</span>
            <input
              type="checkbox"
              checked={settings.autostart}
              disabled={busy}
              onChange={(e) => toggleAutostart(e.target.checked)}
            />
          </label>

          <label className="settings__row">
            <span>Display name</span>
            <select
              value={settings.nameMode}
              onChange={(e) => changeNameMode(e.target.value as NameMode)}
            >
              <option value="persona">Profile name</option>
              <option value="account">Account name</option>
            </select>
          </label>
        </section>

        <section className="settings__group">
          <h2 className="settings__heading">Nicknames</h2>
          {accounts.map((account) => (
            <label className="settings__row" key={account.steamId64}>
              <span className="settings__account">
                {account.personaName.trim() || account.accountName}
              </span>
              <input
                type="text"
                placeholder="Nickname"
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
