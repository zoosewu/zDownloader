/**
 * Tauri API utilities
 * Provides a reliable way to invoke Tauri commands
 * Supports both production (Tauri IPC) and development (HTTP bridge) modes
 */

import { invoke as tauriInvoke } from '@tauri-apps/api/core';

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
  console.log(`[invoke] Calling command '${cmd}'`, { args, isDevelopment });

  try {
    // Development mode: Use HTTP bridge
    if (isDevelopment) {
      const result = await invokeViaHttp<T>(cmd, args);
      console.log(`[invoke] ✓ Command '${cmd}' completed successfully via HTTP bridge`);
      return result;
    }

    // Production mode: Use Tauri IPC
    const result = await invokeViaTauri<T>(cmd, args);
    console.log(`[invoke] ✓ Command '${cmd}' completed successfully via Tauri IPC`);
    return result;
  } catch (error) {
    console.error(`[invoke] ✗ Command '${cmd}' failed:`, error);
    throw error;
  }
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
 * Uses the official @tauri-apps/api package
 */
async function invokeViaTauri<T = unknown>(
  cmd: string,
  args?: Record<string, unknown>
): Promise<T> {
  console.log(`[Tauri] Invoking '${cmd}' via @tauri-apps/api`);

  try {
    // Use the official Tauri API
    const result = await tauriInvoke<T>(cmd, args);
    console.log(`[Tauri] ✓ Command '${cmd}' succeeded`);
    return result;
  } catch (error) {
    console.error(`[Tauri] ✗ Failed to invoke command '${cmd}':`, error);
    throw error;
  }
}

/**
 * Check if Tauri API is available
 * In development mode with HTTP bridge, always returns true
 * In production mode, checks if the Tauri API package is loaded
 */
export function isTauriAvailable(): boolean {
  console.log('[isTauriAvailable] Checking Tauri availability...');
  console.log('[isTauriAvailable] Environment:', {
    isDevelopment,
    VITE_DEV_MODE: import.meta.env.VITE_DEV_MODE,
    VITE_TAURI_HTTP_URL: import.meta.env.VITE_TAURI_HTTP_URL,
    hasWindow: typeof window !== 'undefined',
  });

  // Development mode: HTTP bridge mode
  if (isDevelopment) {
    console.log('[isTauriAvailable] ✓ Development mode - HTTP bridge will be used');
    return true;
  }

  // Production mode: Check if Tauri API is available
  // In Tauri 2.x, the @tauri-apps/api package handles availability internally
  // We just need to check if we're in a browser environment
  if (typeof window === 'undefined') {
    console.log('[isTauriAvailable] ✗ Not in browser environment');
    return false;
  }

  // The @tauri-apps/api package will work if properly loaded
  // We can't easily check this synchronously, so we assume it's available
  // The actual check happens in waitForTauri()
  console.log('[isTauriAvailable] ✓ Assuming Tauri API is available (will verify with test command)');
  return true;
}

/**
 * Wait for Tauri API to be ready
 * In development mode, checks HTTP bridge availability
 * In production mode, waits for window.__TAURI__
 */
export async function waitForTauri(maxAttempts = 100): Promise<boolean> {
  console.log('[waitForTauri] Starting Tauri initialization check...');
  console.log('[waitForTauri] isDevelopment:', isDevelopment);
  console.log('[waitForTauri] maxAttempts:', maxAttempts);

  // Development mode: Check HTTP bridge
  if (isDevelopment) {
    console.log(`[waitForTauri] Development mode - checking HTTP bridge at ${httpBridgeUrl}`);

    for (let i = 0; i < maxAttempts; i++) {
      try {
        const response = await fetch(`${httpBridgeUrl}/health`, {
          method: 'GET',
        });

        if (response.ok) {
          console.log('[waitForTauri] ✓ HTTP Bridge is ready');
          return true;
        }
      } catch (error) {
        if (i % 10 === 0) {  // Log every 10 attempts to reduce spam
          console.log(`[waitForTauri] Waiting for HTTP Bridge... (${i + 1}/${maxAttempts})`);
        }
      }

      await new Promise(resolve => setTimeout(resolve, 100));
    }

    console.error('[waitForTauri] ✗ HTTP Bridge failed to connect after', maxAttempts, 'attempts');
    return false;
  }

  // Production mode: Wait for Tauri IPC
  // In Tauri 2.x with @tauri-apps/api, we test by actually calling the API
  console.log('[waitForTauri] Production mode - testing Tauri API with actual invoke...');

  for (let i = 0; i < maxAttempts; i++) {
    try {
      // Try to invoke a simple command to test if API is ready
      // We use the imported tauriInvoke directly to bypass our wrapper
      await tauriInvoke('greet', { name: 'availability-test' });

      console.log('[waitForTauri] ✓ Tauri API is ready!');
      console.log('[waitForTauri] Successfully invoked test command');
      return true;
    } catch (error) {
      if (i % 10 === 0) {  // Log every 10 attempts to reduce spam
        console.log(`[waitForTauri] Attempt ${i + 1}/${maxAttempts} - API not ready yet`);
        console.log(`[waitForTauri] Error:`, error);
      }
    }

    await new Promise(resolve => setTimeout(resolve, 100));
  }

  console.error('[waitForTauri] ✗ Tauri API failed to initialize after', maxAttempts, 'attempts');
  return false;
}
