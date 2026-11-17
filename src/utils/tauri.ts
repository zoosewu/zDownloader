/**
 * Low-level Tauri transport. Application code should use `src/backend` instead.
 * Arguments are never logged: they can carry credentials.
 */

import { invoke as tauriInvoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';

export const invoke = tauriInvoke;

/**
 * Wait until the backend has started (the page can load before it has). Listens for
 * `app-ready` first, then asks once, so the moment it becomes ready is never missed.
 * False after `timeoutMs`, so the caller can show a clear error instead of spinning forever.
 */
export async function waitForTauri(timeoutMs = 15_000): Promise<boolean> {
  let done: (ready: boolean) => void = () => {};
  const finished = new Promise<boolean>((resolve) => (done = resolve));
  const timer = setTimeout(() => done(false), timeoutMs);
  try {
    const unlisten = await listen('app-ready', () => done(true));
    finished.then(unlisten);
    if (await tauriInvoke<boolean>('app_ready')) done(true);
  } catch {
    done(false); // No Tauri at all (opened in a browser) or the backend is broken.
  }
  const ready = await finished;
  clearTimeout(timer);
  return ready;
}
