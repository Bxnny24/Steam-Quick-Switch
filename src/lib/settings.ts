import { load, type Store } from "@tauri-apps/plugin-store";
import { disable, enable, isEnabled } from "@tauri-apps/plugin-autostart";
import type { Account } from "./api";

export type Language = "en" | "de";
export type NameMode = "persona" | "account";

export interface Settings {
  language: Language;
  autostart: boolean;
  nameMode: NameMode;
  nicknames: Record<string, string>;
}

/** Default language from the OS locale on first run. */
function detectLanguage(): Language {
  return navigator.language.toLowerCase().startsWith("de") ? "de" : "en";
}

let storePromise: Promise<Store> | null = null;
function getStore(): Promise<Store> {
  if (!storePromise) {
    storePromise = load("settings.json", { autoSave: true, defaults: {} });
  }
  return storePromise;
}

/** Load all settings, falling back to sensible defaults. */
export async function loadSettings(): Promise<Settings> {
  const store = await getStore();
  const language = (await store.get<Language>("language")) ?? detectLanguage();
  const nameMode = (await store.get<NameMode>("nameMode")) ?? "persona";
  const nicknames =
    (await store.get<Record<string, string>>("nicknames")) ?? {};
  let autostart = false;
  try {
    autostart = await isEnabled();
  } catch {
    autostart = false;
  }
  return { language, autostart, nameMode, nicknames };
}

export async function setLanguage(language: Language): Promise<void> {
  const store = await getStore();
  await store.set("language", language);
  await store.save();
}

export async function setNameMode(nameMode: NameMode): Promise<void> {
  const store = await getStore();
  await store.set("nameMode", nameMode);
  await store.save();
}

/** Set or clear the per-account nickname (empty string clears it). */
export async function setNickname(
  steamId64: string,
  nickname: string,
): Promise<void> {
  const store = await getStore();
  const nicknames =
    (await store.get<Record<string, string>>("nicknames")) ?? {};
  const trimmed = nickname.trim();
  if (trimmed) {
    nicknames[steamId64] = trimmed;
  } else {
    delete nicknames[steamId64];
  }
  await store.set("nicknames", nicknames);
  await store.save();
}

export async function setAutostart(enabled: boolean): Promise<void> {
  if (enabled) {
    await enable();
  } else {
    await disable();
  }
}

/** Resolve the name shown for an account: nickname > nameMode preference. */
export function resolveDisplayName(account: Account, settings: Settings): string {
  const nickname = settings.nicknames[account.steamId64]?.trim();
  if (nickname) return nickname;
  if (settings.nameMode === "account") return account.accountName;
  return account.personaName.trim() || account.accountName;
}
