// Open the add-download page when a download link is copied. The backend watches the
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
          if ((await backend.urls.inspect(url)).downloadable) navigateTo('add-download', { url });
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
