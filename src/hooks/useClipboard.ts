// A copied download link starts at once when the backend says so (the app-wide switch is on
// and the link's module allows it) and it is one file; otherwise it waits in the
// add-downloads window, where the user picks the files of a shared folder. The
// backend watches the clipboard (OS change notifications, no polling) and sends copied links
// while monitoring is enabled in settings.
import { useEffect } from 'react';
import { backend } from '../backend';

export function useClipboard(tauriReady: boolean = false) {
  useEffect(() => {
    if (!tauriReady) return;
    let disposed = false;
    let unlisten: (() => void) | undefined;
    backend.events
      .onClipboardLink(async (url) => {
        try {
          const info = await backend.urls.inspect(url);
          if (!info.downloadable) return;
          if (info.autoStart && (await backend.urls.list(url).catch(() => [])).length === 1) {
            try {
              await backend.downloads.add({ url, start: { mode: 'now' } });
              return;
            } catch {
              // Could not add it: the add window shows the link and lets the user see why.
            }
          }
          await backend.addWindow.open([{ url, filename: null, headers: [], pageUrl: null }]);
        } catch {
          // Not a URL the backend understands: ignore it.
        }
      })
      .then((u) => (disposed ? u() : (unlisten = u)));
    return () => {
      disposed = true;
      unlisten?.();
    };
  }, [tauriReady]);
}
