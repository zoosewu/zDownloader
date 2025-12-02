/**
 * Tauri API utilities
 * Provides a reliable way to invoke Tauri commands
 * Supports both production (Tauri IPC) and development (HTTP bridge) modes
 */

// Check if we're in development mode (browser-based)
const isDevelopment = import.meta.env.VITE_DEV_MODE === 'true';
const httpBridgeUrl = import.meta.env.VITE_TAURI_HTTP_URL || 'http://localhost:3030';

/**
 * Safe invoke function that works in both development and production
 * - Development: Uses HTTP bridge to communicate with Tauri backend in Docker
 * - Production: Uses window.__TAURI__ for native Tauri IPC
 */
export async function invoke<T = unknown>(
  cmd: string,
  args?: Record<string, unknown>
): Promise<T> {
  // Development mode: Use HTTP bridge
  if (isDevelopment) {
    return invokeViaHttp<T>(cmd, args);
  }

  // Production mode: Use Tauri IPC
  return invokeViaTauri<T>(cmd, args);
}

/**
 * Invoke command via HTTP bridge (development mode)
 */
async function invokeViaHttp<T = unknown>(
  cmd: string,
  args?: Record<string, unknown>
): Promise<T> {
  console.log(`[HTTP Bridge] Invoking '${cmd}' via ${httpBridgeUrl}/invoke`);

  try {
    const response = await fetch(`${httpBridgeUrl}/invoke`, {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json',
      },
      body: JSON.stringify({ cmd, args }),
    });

    if (!response.ok) {
      throw new Error(`HTTP ${response.status}: ${response.statusText}`);
    }

    const result = await response.json();

    if (!result.success) {
      throw new Error(result.error || 'Unknown error');
    }

    console.log(`[HTTP Bridge] Command '${cmd}' succeeded`);
    return result.data as T;
  } catch (error) {
    console.error(`[HTTP Bridge] Failed to invoke command '${cmd}':`, error);
    throw error;
  }
}

/**
 * Invoke command via Tauri IPC (production mode)
 */
async function invokeViaTauri<T = unknown>(
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
 * In development mode, always returns true (uses HTTP bridge)
 * In production mode, checks for window.__TAURI__
 */
export function isTauriAvailable(): boolean {
  // Development mode: HTTP bridge is always available
  if (isDevelopment) {
    return true;
  }

  // Production mode: Check for Tauri IPC
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
 * In development mode, checks HTTP bridge availability
 * In production mode, waits for window.__TAURI__
 */
export async function waitForTauri(maxAttempts = 50): Promise<boolean> {
  // Development mode: Check HTTP bridge
  if (isDevelopment) {
    console.log(`[Tauri] Development mode - checking HTTP bridge at ${httpBridgeUrl}`);

    for (let i = 0; i < maxAttempts; i++) {
      try {
        const response = await fetch(`${httpBridgeUrl}/health`, {
          method: 'GET',
        });

        if (response.ok) {
          console.log('[Tauri] HTTP Bridge is ready');
          return true;
        }
      } catch (error) {
        console.log(`[Tauri] Waiting for HTTP Bridge... (${i + 1}/${maxAttempts})`);
      }

      await new Promise(resolve => setTimeout(resolve, 100));
    }

    console.error('[Tauri] HTTP Bridge failed to connect');
    return false;
  }

  // Production mode: Wait for Tauri IPC
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
