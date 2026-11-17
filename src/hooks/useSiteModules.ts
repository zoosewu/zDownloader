// Installed modules and marketplaces: one copy for the whole app, loaded at startup and again
// whenever the backend says they changed (commands, refreshes, edits to local modules).
import { useEffect } from 'react';
import { atom, useAtomValue, useSetAtom } from 'jotai';
import { backend, type InstalledInfo, type SourceInfo } from '../backend';

interface Modules {
  installed: InstalledInfo[];
  sources: SourceInfo[];
  error: string | null;
  loaded: boolean;
}

const modulesAtom = atom<Modules>({ installed: [], sources: [], error: null, loaded: false });

const reloadModulesAtom = atom(null, async (get, set) => {
  try {
    const [installed, sources] = await Promise.all([backend.modules.list(), backend.modules.sources()]);
    set(modulesAtom, { installed, sources, error: null, loaded: true });
  } catch (e) {
    set(modulesAtom, { ...get(modulesAtom), error: e instanceof Error ? e.message : String(e), loaded: true });
  }
});

/** Keep the modules in sync with the backend. Mount once, at the app root. */
export function useModulesSync(ready: boolean) {
  const reload = useSetAtom(reloadModulesAtom);
  useEffect(() => {
    if (!ready) return;
    let disposed = false;
    let unlisten: (() => void) | undefined;
    // Subscribe first, then load, so a change in between is not missed.
    backend.events.onModulesChanged(() => reload()).then((stop) => {
      if (disposed) stop();
      else unlisten = stop;
      reload();
    });
    return () => {
      disposed = true;
      unlisten?.();
    };
  }, [ready, reload]);
}

export function useSiteModules() {
  return useAtomValue(modulesAtom);
}
