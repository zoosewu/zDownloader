// Watch the clipboard and open the add-download page when a download link is copied.
import { useEffect, useRef } from 'react';
import { useAtomValue } from 'jotai';
import { backend } from '../backend';
import { invoke } from '../utils/tauri';
import { settingsAtom } from '../stores/settingsStore';
import { useNavigation } from './useNavigation';

const POLL_MS = 400;

export function useClipboard(tauriReady: boolean = false) {
  /** null until the first read: whatever is on the clipboard at startup is not new. */
  const lastRef = useRef<string | null>(null);
  const busyRef = useRef(false);
  const { navigateTo } = useNavigation();
  const settings = useAtomValue(settingsAtom);

  useEffect(() => {
    if (!tauriReady || !settings.enableClipboardMonitoring) return;

    const interval = setInterval(async () => {
      if (busyRef.current) return;
      busyRef.current = true;
      try {
        const content = (await invoke<string>('get_clipboard_content').catch(() => '')).trim();
        const first = lastRef.current === null;
        if (content === lastRef.current) return;
        lastRef.current = content;
        if (first || !/^https?:\/\//i.test(content)) return;
        if ((await backend.urls.inspect(content)).downloadable) {
          navigateTo('add-download', { url: content });
        }
      } catch {
        // Not ready yet, or not a URL the backend understands: wait for the next copy.
      } finally {
        busyRef.current = false;
      }
    }, POLL_MS);

    return () => clearInterval(interval);
  }, [tauriReady, settings.enableClipboardMonitoring, navigateTo]);
}
