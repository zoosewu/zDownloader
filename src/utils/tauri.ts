/**
 * Low-level Tauri transport. Application code should use `src/backend` instead.
 * Arguments are never logged: they can carry credentials.
 */

import { invoke as tauriInvoke } from '@tauri-apps/api/core';

export const invoke = tauriInvoke;

/**
 * Wait until the backend answers. Returns false after `maxAttempts` (100 ms apart) so the
 * caller can show a clear error instead of spinning forever.
 */
export async function waitForTauri(maxAttempts = 100): Promise<boolean> {
  for (let i = 0; i < maxAttempts; i++) {
    try {
      await tauriInvoke('greet', { name: 'ready-check' });
      return true;
    } catch {
      // Not ready yet.
    }
    await new Promise((resolve) => setTimeout(resolve, 100));
  }
  return false;
}
