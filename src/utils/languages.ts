// The languages to choose from: the built-in ones plus the user's files in <app data>/locales
// (`<code>.json`, the same keys as src/locales; missing keys show in English). A file named after
// a built-in language overrides only the keys it has. The setting "" follows the system.
import { atom, getDefaultStore } from 'jotai';
import i18n, { BUILT_IN, matchLanguage } from '../i18n';
import { backend } from '../backend';

export type Language = { code: string; name: string };
/** A language file that could not be used, and why. */
export type LocaleProblem = { file: string; message: string };

const builtIns: Language[] = Object.entries(BUILT_IN).map(([code, { name }]) => ({ code, name }));

export const languagesAtom = atom<Language[]>(builtIns);
export const localeProblemsAtom = atom<LocaleProblem[]>([]);

/** The language's name in itself: the file's `_name`, else what the system calls it, else the code. */
export function languageName(code: string, translation: Record<string, unknown>): string {
  if (typeof translation._name === 'string' && translation._name.trim()) return translation._name.trim();
  if (BUILT_IN[code]) return BUILT_IN[code].name;
  try {
    return new Intl.DisplayNames([code], { type: 'language' }).of(code) ?? code;
  } catch {
    return code;
  }
}

/** The code to show for a setting: itself if we have it, else the system's language. */
export function resolveLanguage(setting: string, available: readonly string[] = getDefaultStore().get(languagesAtom).map((l) => l.code)): string {
  if (setting && available.includes(setting)) return setting;
  return matchLanguage(navigator.languages, available);
}

let loaded: string[] = [];

/** Read the user's language files again; a file changed or removed since takes effect now. */
export async function loadUserLocales(): Promise<void> {
  const files = await backend.locales.list();
  // Start again from the built-in texts, so a key removed from a file is gone too.
  for (const code of loaded) i18n.removeResourceBundle(code, 'translation');
  for (const [code, { translation }] of Object.entries(BUILT_IN)) {
    i18n.removeResourceBundle(code, 'translation');
    i18n.addResourceBundle(code, 'translation', structuredClone(translation));
  }

  const problems: LocaleProblem[] = [];
  const added: Language[] = [];
  for (const { code, text } of files) {
    let translation: unknown;
    try {
      translation = JSON.parse(text.replace(/^﻿/, ''));
    } catch (e) {
      problems.push({ file: `${code}.json`, message: e instanceof Error ? e.message : String(e) });
      continue;
    }
    if (typeof translation !== 'object' || translation === null || Array.isArray(translation)) {
      problems.push({ file: `${code}.json`, message: 'not a JSON object' });
      continue;
    }
    i18n.addResourceBundle(code, 'translation', translation, true, true);
    if (!BUILT_IN[code]) added.push({ code, name: languageName(code, translation as Record<string, unknown>) });
  }
  loaded = files.map((f) => f.code);
  added.sort((a, b) => a.name.localeCompare(b.name));

  const store = getDefaultStore();
  store.set(languagesAtom, [...builtIns, ...added]);
  store.set(localeProblemsAtom, problems);
  // Show the new texts in the current language.
  await i18n.changeLanguage(i18n.language);
}

/** Show the app in the language a setting asks for (the tray menu too). */
export async function applyLanguage(setting: string): Promise<void> {
  const code = resolveLanguage(setting);
  document.documentElement.lang = code;
  if (code !== i18n.language) await i18n.changeLanguage(code);
  const t = i18n.getFixedT(code);
  await backend.app
    .setTrayLabels({ showHide: t('tray.showHide'), pauseAll: t('tray.pauseAll'), resumeAll: t('tray.resumeAll'), quit: t('tray.quit') })
    .catch((error) => console.warn('[tray] cannot set the menu language:', error.message));
}
