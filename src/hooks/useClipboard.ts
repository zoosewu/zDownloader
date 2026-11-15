// Open the add-download page when a download link is copied, or start it at once when the
// backend says so (app-wide switch on and the module allows it). The backend watches the
// clipboard (OS change notifications, no polling) and sends copied links as events while
// monitoring is enabled in settings.
import { useEffect } from 'react';
import { backend } from '../backend';
import { useNavigation } from './useNavigation';

export function useClipboard(tauriReady: boolean = false) {
  const { navigateTo } = useNavigation();

  useEffect(() => {
    if (!tauriReady) return;
    let disposed = false;
    let unlisten: (() => void) | undefined;
    backend.events
      .onClipboardLink(async (url) => {
        try {
          const info = await backend.urls.inspect(url);
          if (!info.downloadable) return;
          if (info.autoStart) {
            try {
              await backend.downloads.add({ url, start: { mode: 'now' } });
              return;
            } catch {
              // Could not add it: let the user see the link and the reason on the add page.
            }
          }
          navigateTo('add-download', { url });
        } catch {
          // Not a URL the backend understands: ignore it.
        }
      })
      .then((u) => (disposed ? u() : (unlisten = u)));
    return () => {
      disposed = true;
      unlisten?.();
    };
  }, [tauriReady, navigateTo]);
}
