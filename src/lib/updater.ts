import { check } from "@tauri-apps/plugin-updater";
import { relaunch } from "@tauri-apps/plugin-process";

export type UpdateStatus =
  | { state: "idle" }
  | { state: "checking" }
  | { state: "downloading"; version: string }
  | { state: "ready" }
  | { state: "error"; message: string };

/**
 * Check for an update; if one is available, download, install, and relaunch.
 * Failures (offline, no release yet) are reported but stay silent in the UI.
 */
export async function runUpdate(
  setStatus: (status: UpdateStatus) => void,
): Promise<void> {
  try {
    setStatus({ state: "checking" });
    const update = await check();
    if (!update) {
      setStatus({ state: "idle" });
      return;
    }
    setStatus({ state: "downloading", version: update.version });
    await update.downloadAndInstall();
    setStatus({ state: "ready" });
    await relaunch();
  } catch (e) {
    setStatus({ state: "error", message: String(e) });
  }
}
