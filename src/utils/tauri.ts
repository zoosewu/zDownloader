/**
 * Low-level Tauri transport. Application code should use `src/backend` instead.
 *
 * Production talks to Rust over Tauri IPC; `VITE_DEV_MODE=true` routes through the
 * development HTTP bridge (src-tauri/src/http_bridge.rs). Arguments are never logged:
 * they can carry credentials.
 */

import { invoke as tauriInvoke } from '@tauri-apps/api/core';

const isDevelopment = import.meta.env.VITE_DEV_MODE === 'true';
const httpBridgeUrl = import.meta.env.VITE_TAURI_HTTP_URL || 'http://localhost:3030';

export async function invoke<T = unknown>(cmd: string, args?: Record<string, unknown>): Promise<T> {
  return isDevelopment ? invokeViaHttp<T>(cmd, args) : tauriInvoke<T>(cmd, args);
}

async function invokeViaHttp<T>(cmd: string, args?: Record<string, unknown>): Promise<T> {
  const response = await fetch(`${httpBridgeUrl}/invoke`, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({ cmd, args }),
  });
  if (!response.ok) {
    throw new Error(`HTTP bridge: ${response.status} ${response.statusText}`);
  }
  const result = await response.json();
  if (!result.success) {
    throw new Error(result.error || `HTTP bridge: '${cmd}' failed`);
  }
  return result.data as T;
}

/**
 * Wait until the backend answers. Returns false after `maxAttempts` (100 ms apart) so the
 * caller can show a clear error instead of spinning forever.
 */
export async function waitForTauri(maxAttempts = 100): Promise<boolean> {
  for (let i = 0; i < maxAttempts; i++) {
    try {
      if (isDevelopment) {
        if ((await fetch(`${httpBridgeUrl}/health`)).ok) return true;
      } else {
        await tauriInvoke('greet', { name: 'ready-check' });
        return true;
      }
    } catch {
      // Not ready yet.
    }
    await new Promise((resolve) => setTimeout(resolve, 100));
  }
  return false;
}
