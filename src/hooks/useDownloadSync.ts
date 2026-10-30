// Keeps the download store in sync with the backend. Mount once, at the app root.
import { useEffect } from 'react';
import { useSetAtom } from 'jotai';
import type { UnlistenFn } from '@tauri-apps/api/event';
import { backend } from '../backend';
import { applyEventAtom, mergeSnapshotAtom } from '../stores/downloadStore';

/**
 * Subscribe first, then load the full list, so nothing that happens in between is missed;
 * `rev` ordering in the store makes the event/snapshot race harmless.
 */
export function useDownloadSync(ready: boolean) {
  const applyEvent = useSetAtom(applyEventAtom);
  const mergeSnapshot = useSetAtom(mergeSnapshotAtom);

  useEffect(() => {
    if (!ready) return;
    let disposed = false;
    const unlisten: UnlistenFn[] = [];

    const refresh = () =>
      backend.downloads
        .list()
        .then((tasks) => {
          if (!disposed) mergeSnapshot(tasks);
        })
        .catch((error) => console.error('[downloads] failed to load the list:', error.message));

    (async () => {
      const subscriptions = await Promise.all([
        backend.events.onDownloadEvent(applyEvent),
        backend.events.onResync(refresh),
      ]);
      // Unmounted while subscribing: release now instead of leaking the listeners.
      if (disposed) {
        subscriptions.forEach((u) => u());
        return;
      }
      unlisten.push(...subscriptions);
      await refresh();
    })();

    return () => {
      disposed = true;
      unlisten.forEach((u) => u());
    };
  }, [ready, applyEvent, mergeSnapshot]);
}
