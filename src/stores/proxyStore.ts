// Jotai atoms for proxy state management
import { atom } from 'jotai';
import { invoke } from '../utils/tauri';
import { ProxyInfo, ProxyStatus } from '../types/proxy';

// Proxy list atom - all proxies
export const proxyListAtom = atom<ProxyInfo[]>([]);

// Active proxies atom (derived) - only active proxies
export const activeProxiesAtom = atom((get) => {
  const proxies = get(proxyListAtom);
  return proxies.filter((proxy) => proxy.status === ProxyStatus.Active);
});

// Failed proxies atom (derived) - failed proxies
export const failedProxiesAtom = atom((get) => {
  const proxies = get(proxyListAtom);
  return proxies.filter((proxy) => proxy.status === ProxyStatus.Failed);
});

// Blacklisted proxies atom (derived) - blacklisted proxies
export const blacklistedProxiesAtom = atom((get) => {
  const proxies = get(proxyListAtom);
  return proxies.filter((proxy) => proxy.status === ProxyStatus.Blacklisted);
});

// Active proxy count atom (derived)
export const activeProxyCountAtom = atom((get) => {
  const activeProxies = get(activeProxiesAtom);
  return activeProxies.length;
});

// Total proxy count atom (derived)
export const totalProxyCountAtom = atom((get) => {
  const proxies = get(proxyListAtom);
  return proxies.length;
});

// Proxy statistics atom (derived)
export const proxyStatsAtom = atom((get) => {
  const proxies = get(proxyListAtom);
  const active = get(activeProxiesAtom);
  const failed = get(failedProxiesAtom);
  const blacklisted = get(blacklistedProxiesAtom);

  const totalSuccess = proxies.reduce(
    (sum, p) => sum + p.successful_requests,
    0
  );
  const totalFailed = proxies.reduce((sum, p) => sum + p.failed_requests, 0);
  const avgHealth =
    proxies.length > 0
      ? proxies.reduce((sum, p) => sum + p.health_score, 0) / proxies.length
      : 0;

  return {
    total: proxies.length,
    active: active.length,
    failed: failed.length,
    blacklisted: blacklisted.length,
    totalSuccess,
    totalFailed,
    avgHealth,
  };
});

// Loading state atom
export const proxyLoadingAtom = atom<boolean>(false);

// Error state atom
export const proxyErrorAtom = atom<string | null>(null);

// === Actions ===

// Load proxy list from backend
export const loadProxyListAtom = atom(null, async (_get, set) => {
  set(proxyLoadingAtom, true);
  set(proxyErrorAtom, null);

  try {
    const proxies = await invoke<ProxyInfo[]>('get_proxy_list');
    set(proxyListAtom, proxies);
  } catch (error) {
    const errorMsg = error instanceof Error ? error.message : String(error);
    set(proxyErrorAtom, errorMsg);
    console.error('Failed to load proxy list:', error);
  } finally {
    set(proxyLoadingAtom, false);
  }
});

// Import proxy list from file
export const importProxyListAtom = atom(
  null,
  async (_get, set, filePath: string) => {
    set(proxyLoadingAtom, true);
    set(proxyErrorAtom, null);

    try {
      const count = await invoke<number>('import_proxy_list', { filePath });
      // Reload the list after import
      await set(loadProxyListAtom);
      return count;
    } catch (error) {
      const errorMsg = error instanceof Error ? error.message : String(error);
      set(proxyErrorAtom, errorMsg);
      console.error('Failed to import proxy list:', error);
      throw error;
    } finally {
      set(proxyLoadingAtom, false);
    }
  }
);

// Remove a proxy
export const removeProxyAtom = atom(
  null,
  async (_get, set, proxyId: string) => {
    set(proxyLoadingAtom, true);
    set(proxyErrorAtom, null);

    try {
      await invoke('remove_proxy', { proxyId });
      // Reload the list after removal
      await set(loadProxyListAtom);
    } catch (error) {
      const errorMsg = error instanceof Error ? error.message : String(error);
      set(proxyErrorAtom, errorMsg);
      console.error('Failed to remove proxy:', error);
      throw error;
    } finally {
      set(proxyLoadingAtom, false);
    }
  }
);

// Clear all proxies
export const clearAllProxiesAtom = atom(null, async (_get, set) => {
  set(proxyLoadingAtom, true);
  set(proxyErrorAtom, null);

  try {
    await invoke('clear_all_proxies');
    set(proxyListAtom, []);
  } catch (error) {
    const errorMsg = error instanceof Error ? error.message : String(error);
    set(proxyErrorAtom, errorMsg);
    console.error('Failed to clear proxies:', error);
    throw error;
  } finally {
    set(proxyLoadingAtom, false);
  }
});

// Reset failed proxies
export const resetFailedProxiesAtom = atom(null, async (_get, set) => {
  set(proxyLoadingAtom, true);
  set(proxyErrorAtom, null);

  try {
    await invoke('reset_failed_proxies');
    // Reload the list after reset
    await set(loadProxyListAtom);
  } catch (error) {
    const errorMsg = error instanceof Error ? error.message : String(error);
    set(proxyErrorAtom, errorMsg);
    console.error('Failed to reset failed proxies:', error);
    throw error;
  } finally {
    set(proxyLoadingAtom, false);
  }
});
