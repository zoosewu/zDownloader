// Jotai atoms for proxy state management
import { atom, type Setter } from 'jotai';
import { backend, type ProxyInfo } from '../backend';

// Proxy list atom - all proxies
export const proxyListAtom = atom<ProxyInfo[]>([]);

// Active proxies atom (derived) - only active proxies
export const activeProxiesAtom = atom((get) => {
  const proxies = get(proxyListAtom);
  return proxies.filter((proxy) => proxy.status === 'active');
});

// Failed proxies atom (derived) - failed proxies
export const failedProxiesAtom = atom((get) => {
  const proxies = get(proxyListAtom);
  return proxies.filter((proxy) => proxy.status === 'failed');
});

// Blacklisted proxies atom (derived) - blacklisted proxies
export const blacklistedProxiesAtom = atom((get) => {
  const proxies = get(proxyListAtom);
  return proxies.filter((proxy) => proxy.status === 'blacklisted');
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
    (sum, p) => sum + p.successfulRequests,
    0
  );
  const totalFailed = proxies.reduce((sum, p) => sum + p.failedRequests, 0);
  const avgHealth =
    proxies.length > 0
      ? proxies.reduce((sum, p) => sum + p.healthScore * 100, 0) / proxies.length
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
// The backend owns the proxies. These ask it for changes; the list itself only ever comes from
// `loadProxyListAtom`, which the settings page runs on every `proxies-changed` event.

/** Fetch the list. Quiet (no loading state): it runs on every change the backend reports. */
export const loadProxyListAtom = atom(null, async (_get, set) => {
  try {
    set(proxyListAtom, await backend.proxies.list());
  } catch (error) {
    set(proxyErrorAtom, error instanceof Error ? error.message : String(error));
  }
});

/** Run a change with the loading and error state; the event brings the new list. */
async function change<T>(set: Setter, work: () => Promise<T>): Promise<T> {
  set(proxyLoadingAtom, true);
  set(proxyErrorAtom, null);
  try {
    return await work();
  } catch (error) {
    set(proxyErrorAtom, error instanceof Error ? error.message : String(error));
    throw error;
  } finally {
    set(proxyLoadingAtom, false);
  }
}

/** Import a list file; resolves to how many were added. */
export const importProxyListAtom = atom(null, (_get, set, filePath: string) =>
  change(set, async () => (await backend.proxies.import(filePath)).added),
);

export const removeProxyAtom = atom(null, (_get, set, proxyId: string) => change(set, () => backend.proxies.remove(proxyId)));

export const clearAllProxiesAtom = atom(null, (_get, set) => change(set, () => backend.proxies.clear()));

/** Give failed and blacklisted proxies another chance. */
export const resetFailedProxiesAtom = atom(null, (_get, set) => change(set, () => backend.proxies.resetFailed()));
