import { invoke } from "@tauri-apps/api/core";

/** A Steam account as returned by the Rust backend (camelCase JSON). */
export interface Account {
  steamId64: string;
  accountName: string;
  personaName: string;
  rememberPassword: boolean;
  mostRecent: boolean;
  timestamp: number;
  isCurrent: boolean;
  /** Base64 PNG data URL of the cached avatar, or null. */
  avatar: string | null;
}

/** List all Steam accounts known to this machine, newest first. */
export function listAccounts(): Promise<Account[]> {
  return invoke<Account[]>("list_accounts");
}

/** Switch the active Steam account (closes and relaunches Steam). */
export function switchAccount(account: Account): Promise<void> {
  return invoke("switch_account", {
    accountName: account.accountName,
    steamId64: account.steamId64,
  });
}

/** Best display name for an account, ignoring user settings. */
export function displayName(account: Account): string {
  return account.personaName.trim() || account.accountName;
}
