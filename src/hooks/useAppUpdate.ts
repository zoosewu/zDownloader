// App updates (tauri-plugin-updater, GitHub Releases' latest.json): checked at start and every
// few hours while enabled in settings; installing always waits for the user. Shared state, so
// the banner and the settings page show the same update.
import { useCallback, useEffect } from 'react';
import { atom, useAtom, useAtomValue } from 'jotai';
import { check, type Update } from '@tauri-apps/plugin-updater';
import { relaunch } from '@tauri-apps/plugin-process';
import { backend } from '../backend';
import { settingsAtom } from '../stores/settingsStore';

const CHECK_EVERY_MS = 6 * 60 * 60 * 1000;

/**
 * A release is published as soon as one platform is built, and latest.json lists the others as
 * they finish: until this platform is in, there is no update for it yet (not an error).
 */
export function isPlatformNotReleased(message: string): boolean {
  return message.includes('was not found on the response `platforms` object');
}

export type UpdateState =
  | { kind: 'idle' }
  | { kind: 'checking' }
  | { kind: 'upToDate' }
  | { kind: 'available'; update: Update }
  /** `total` is null when the server does not say. */
  | { kind: 'downloading'; update: Update; done: number; total: number | null }
  | { kind: 'installing'; update: Update }
  | { kind: 'failed'; message: string; update?: Update };

export const updateStateAtom = atom<UpdateState>({ kind: 'idle' });
/** The version the user put off with "Later"; not offered again until the next start. */
const dismissedAtom = atom<string | null>(null);

export function useAppUpdate() {
  const [state, setState] = useAtom(updateStateAtom);
  const [dismissed, setDismissed] = useAtom(dismissedAtom);

  /** `quiet`: an automatic check; its failures are only logged (offline is normal). */
  const checkNow = useCallback(
    async (quiet = false) => {
      setState((s) => (s.kind === 'downloading' || s.kind === 'installing' ? s : { kind: 'checking' }));
      try {
        const update = await check();
        setState(update ? { kind: 'available', update } : { kind: 'upToDate' });
      } catch (e) {
        const message = e instanceof Error ? e.message : String(e);
        if (isPlatformNotReleased(message)) {
          setState({ kind: 'upToDate' });
        } else if (quiet) {
          console.warn('[update] check failed:', message);
          setState({ kind: 'idle' });
        } else {
          setState({ kind: 'failed', message });
        }
      }
    },
    [setState],
  );

  /** Download, stop the downloads cleanly, install, restart. */
  const install = useCallback(
    async (update: Update) => {
      let done = 0;
      let total: number | null = null;
      try {
        setState({ kind: 'downloading', update, done, total });
        await update.download((event) => {
          if (event.event === 'Started') total = event.data.contentLength ?? null;
          if (event.event === 'Progress') done += event.data.chunkLength;
          setState({ kind: 'downloading', update, done, total });
        });
        setState({ kind: 'installing', update });
        await backend.app.prepareUpdate();
        await update.install();
        await relaunch(); // Windows ends the app in install(); elsewhere restart here
      } catch (e) {
        setState({ kind: 'failed', message: e instanceof Error ? e.message : String(e), update });
      }
    },
    [setState],
  );

  return { state, checkNow, install, dismissed, dismiss: setDismissed };
}

/** Run the automatic checks (main window only). */
export function useAutomaticUpdateChecks(ready: boolean) {
  const enabled = useAtomValue(settingsAtom).checkAppUpdates;
  const { checkNow } = useAppUpdate();
  useEffect(() => {
    // Dev builds have no release to compare with.
    if (!ready || !enabled || import.meta.env.DEV) return;
    checkNow(true);
    const timer = setInterval(() => checkNow(true), CHECK_EVERY_MS);
    return () => clearInterval(timer);
  }, [ready, enabled, checkNow]);
}
