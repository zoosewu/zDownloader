// The user's categories live in the settings; saving them re-checks every rule in the backend.
import { useCallback } from 'react';
import { useAtom } from 'jotai';
import { settingsAtom } from '../stores/settingsStore';
import { backend, type Category } from '../backend';

export function useCategories() {
  const [settings, setSettings] = useAtom(settingsAtom);

  /** Throws the backend's message when a rule is invalid; nothing is saved then. */
  const save = useCallback(
    async (categories: Category[]) => setSettings(await backend.settings.update({ ...settings, categories })),
    [settings, setSettings],
  );

  return { categories: settings.categories, save };
}
