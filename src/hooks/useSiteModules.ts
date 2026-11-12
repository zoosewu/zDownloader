// Installed modules and marketplaces, refetched whenever the backend says they changed
// (commands, refreshes, edits to local modules).
import { useCallback, useEffect, useState } from 'react';
import { backend, type InstalledInfo, type SourceInfo } from '../backend';

export function useSiteModules() {
  const [installed, setInstalled] = useState<InstalledInfo[]>([]);
  const [sources, setSources] = useState<SourceInfo[]>([]);
  const [error, setError] = useState<string | null>(null);
  const [loaded, setLoaded] = useState(false);

  const reload = useCallback(async () => {
    try {
      const [modules, marketplaces] = await Promise.all([backend.modules.list(), backend.modules.sources()]);
      setInstalled(modules);
      setSources(marketplaces);
      setError(null);
    } catch (e) {
      setError((e as Error).message);
    } finally {
      setLoaded(true);
    }
  }, []);

  useEffect(() => {
    let disposed = false;
    let unlisten: (() => void) | null = null;
    reload();
    backend.events.onModulesChanged(reload).then((stop) => {
      if (disposed) stop();
      else unlisten = stop;
    });
    return () => {
      disposed = true;
      unlisten?.();
    };
  }, [reload]);

  return { installed, sources, error, loaded, reload };
}
