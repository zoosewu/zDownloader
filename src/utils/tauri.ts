/**
 * Tauri API utilities
 * Provides a reliable way to invoke Tauri commands
 */

/**
 * Safe invoke function that works in both development and production
 * Falls back to window.__TAURI__ if direct import doesn't work
 */
export async function invoke<T = unknown>(
  cmd: string,
  args?: Record<string, unknown>
): Promise<T> {
  // Check if we're in a Tauri environment
  if (typeof window === 'undefined') {
    throw new Error('[Tauri] Not running in a browser environment');
  }

  const windowWithTauri = window as any;

  // First try: Use window.__TAURI__.invoke (Tauri 2.x)
  if (windowWithTauri.__TAURI__?.invoke) {
    try {
      console.log(`[Tauri] Invoking '${cmd}' via __TAURI__.invoke`);
      return await windowWithTauri.__TAURI__.invoke(cmd, args);
    } catch (error) {
      console.error(`[Tauri] Failed to invoke command '${cmd}' via __TAURI__.invoke:`, error);
      throw error;
    }
  }

  // Second try: Use window.__TAURI__.core.invoke
  if (windowWithTauri.__TAURI__?.core?.invoke) {
    try {
      console.log(`[Tauri] Invoking '${cmd}' via __TAURI__.core.invoke`);
      return await windowWithTauri.__TAURI__.core.invoke(cmd, args);
    } catch (error) {
      console.error(`[Tauri] Failed to invoke command '${cmd}' via __TAURI__.core.invoke:`, error);
      throw error;
    }
  }

  // Third try: Use window.__TAURI_INVOKE__ (older Tauri versions)
  if (windowWithTauri.__TAURI_INVOKE__) {
    try {
      console.log(`[Tauri] Invoking '${cmd}' via __TAURI_INVOKE__`);
      return await windowWithTauri.__TAURI_INVOKE__(cmd, args);
    } catch (error) {
      console.error(`[Tauri] Failed to invoke command '${cmd}' via __TAURI_INVOKE__:`, error);
      throw error;
    }
  }

  // If none of the above work, throw a descriptive error
  console.error('[Tauri] No invoke method available. __TAURI__ object:', windowWithTauri.__TAURI__);
  throw new Error(`[Tauri] Cannot invoke command '${cmd}': Tauri API not available`);
}

/**
 * Check if Tauri API is available
 */
export function isTauriAvailable(): boolean {
  if (typeof window === 'undefined') {
    return false;
  }

  const windowWithTauri = window as any;

  // Check if any of the invoke methods are available
  const hasInvoke =
    windowWithTauri.__TAURI__?.invoke ||
    windowWithTauri.__TAURI__?.core?.invoke ||
    windowWithTauri.__TAURI_INVOKE__;

  return !!hasInvoke;
}

/**
 * Wait for Tauri API to be ready
 */
export async function waitForTauri(maxAttempts = 50): Promise<boolean> {
  for (let i = 0; i < maxAttempts; i++) {
    if (isTauriAvailable()) {
      const windowWithTauri = window as any;
      console.log('[Tauri] API is ready. Available methods:', {
        hasDirectInvoke: !!windowWithTauri.__TAURI__?.invoke,
        hasCoreInvoke: !!windowWithTauri.__TAURI__?.core?.invoke,
        hasLegacyInvoke: !!windowWithTauri.__TAURI_INVOKE__,
      });
      return true;
    }
    console.log(`[Tauri] Waiting for API... (${i + 1}/${maxAttempts})`);
    await new Promise(resolve => setTimeout(resolve, 100));
  }
  console.error('[Tauri] API failed to initialize');
  return false;
}
