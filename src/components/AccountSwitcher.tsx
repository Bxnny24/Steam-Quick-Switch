import { useCallback, useEffect, useState } from "react";
import { useTranslation } from "react-i18next";
import { Account, listAccounts, switchAccount } from "../lib/api";
import { Settings, loadSettings, resolveDisplayName } from "../lib/settings";
import i18n from "../i18n";
import { SettingsView } from "./Settings";

export function AccountSwitcher() {
  const { t } = useTranslation();
  const [accounts, setAccounts] = useState<Account[]>([]);
  const [settings, setSettings] = useState<Settings | null>(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [switching, setSwitching] = useState<string | null>(null);
  const [view, setView] = useState<"switcher" | "settings">("switcher");

  const loadAll = useCallback(async () => {
    setLoading(true);
    setError(null);
    try {
      const [accs, setts] = await Promise.all([listAccounts(), loadSettings()]);
      setAccounts(accs);
      setSettings(setts);
      void i18n.changeLanguage(setts.language);
    } catch (e) {
      setError(String(e));
    } finally {
      setLoading(false);
    }
  }, []);

  const reloadSettings = useCallback(async () => {
    try {
      setSettings(await loadSettings());
    } catch (e) {
      setError(String(e));
    }
  }, []);

  useEffect(() => {
    loadAll();
  }, [loadAll]);

  async function handleSwitch(account: Account) {
    if (switching || account.isCurrent) return;
    setSwitching(account.steamId64);
    setError(null);
    try {
      await switchAccount(account);
      // Steam needs a moment to relaunch; then reflect the new active account.
      window.setTimeout(loadAll, 1500);
    } catch (e) {
      setError(String(e));
    } finally {
      setSwitching(null);
    }
  }

  function nameOf(account: Account): string {
    return settings
      ? resolveDisplayName(account, settings)
      : account.personaName.trim() || account.accountName;
  }

  if (view === "settings" && settings) {
    return (
      <SettingsView
        accounts={accounts}
        settings={settings}
        onChanged={reloadSettings}
        onClose={() => setView("switcher")}
      />
    );
  }

  return (
    <div className="app">
      <header className="app__header">
        <h1 className="app__title">{t("app.title")}</h1>
        <div className="app__actions">
          <button
            className="icon-button"
            onClick={loadAll}
            title={t("app.refresh")}
            disabled={loading}
          >
            ⟳
          </button>
          <button
            className="icon-button"
            onClick={() => setView("settings")}
            title={t("app.settings")}
            disabled={!settings}
          >
            ⚙
          </button>
        </div>
      </header>

      {error && <div className="banner banner--error">{error}</div>}

      {loading && accounts.length === 0 ? (
        <div className="state">{t("app.loading")}</div>
      ) : accounts.length === 0 ? (
        <div className="state">
          {t("app.noAccounts")}
          <br />
          {t("app.noAccountsHint")}
        </div>
      ) : (
        <ul className="accounts">
          {accounts.map((account) => {
            const isSwitching = switching === account.steamId64;
            return (
              <li key={account.steamId64}>
                <button
                  className={`account${account.isCurrent ? " account--current" : ""}`}
                  onClick={() => handleSwitch(account)}
                  disabled={!!switching || account.isCurrent}
                >
                  <span className="account__avatar">
                    {account.avatar ? (
                      <img src={account.avatar} alt="" />
                    ) : (
                      <span className="account__avatar-fallback">
                        {nameOf(account).charAt(0).toUpperCase()}
                      </span>
                    )}
                  </span>
                  <span className="account__info">
                    <span className="account__name">{nameOf(account)}</span>
                    <span className="account__login">{account.accountName}</span>
                  </span>
                  <span className="account__status">
                    {isSwitching
                      ? t("app.switching")
                      : account.isCurrent
                        ? t("app.active")
                        : ""}
                  </span>
                </button>
              </li>
            );
          })}
        </ul>
      )}

      <footer className="app__footer">
        {t("app.accountCount", { count: accounts.length })}
      </footer>
    </div>
  );
}
