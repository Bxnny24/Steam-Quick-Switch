import { useCallback, useEffect, useState } from "react";
import { Account, displayName, listAccounts, switchAccount } from "../lib/api";

export function AccountSwitcher() {
  const [accounts, setAccounts] = useState<Account[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [switching, setSwitching] = useState<string | null>(null);

  const load = useCallback(async () => {
    setLoading(true);
    setError(null);
    try {
      setAccounts(await listAccounts());
    } catch (e) {
      setError(String(e));
    } finally {
      setLoading(false);
    }
  }, []);

  useEffect(() => {
    load();
  }, [load]);

  async function handleSwitch(account: Account) {
    if (switching || account.isCurrent) return;
    setSwitching(account.steamId64);
    setError(null);
    try {
      await switchAccount(account);
      // Steam needs a moment to relaunch; then reflect the new active account.
      window.setTimeout(load, 1500);
    } catch (e) {
      setError(String(e));
    } finally {
      setSwitching(null);
    }
  }

  return (
    <div className="app">
      <header className="app__header">
        <h1 className="app__title">Steam Quick Switch</h1>
        <button
          className="icon-button"
          onClick={load}
          title="Refresh"
          disabled={loading}
        >
          ⟳
        </button>
      </header>

      {error && <div className="banner banner--error">{error}</div>}

      {loading && accounts.length === 0 ? (
        <div className="state">Loading accounts…</div>
      ) : accounts.length === 0 ? (
        <div className="state">
          No saved Steam accounts found.
          <br />
          Enable “Remember password” when logging in to Steam.
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
                        {displayName(account).charAt(0).toUpperCase()}
                      </span>
                    )}
                  </span>
                  <span className="account__info">
                    <span className="account__name">{displayName(account)}</span>
                    <span className="account__login">{account.accountName}</span>
                  </span>
                  <span className="account__status">
                    {isSwitching
                      ? "Switching…"
                      : account.isCurrent
                        ? "Active"
                        : ""}
                  </span>
                </button>
              </li>
            );
          })}
        </ul>
      )}

      <footer className="app__footer">
        {accounts.length} account{accounts.length === 1 ? "" : "s"}
      </footer>
    </div>
  );
}
