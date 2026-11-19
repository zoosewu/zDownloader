// i18n: the built-in languages, starting in the system's. The saved setting and the user's own
// language files (utils/languages.ts) are applied once the backend answers.
import i18n from 'i18next';
import { initReactI18next } from 'react-i18next';

import zhTW from './locales/zh-TW.json';
import en from './locales/en.json';
import ja from './locales/ja.json';

/** Code -> its texts and its name in itself. */
export const BUILT_IN: Record<string, { translation: object; name: string }> = {
  'zh-TW': { translation: zhTW, name: '繁體中文' },
  en: { translation: en, name: 'English' },
  ja: { translation: ja, name: '日本語' },
};

/**
 * The first of `wanted` (the system's languages, most preferred first) we have: the same tag,
 * its base language ("ja-JP" -> "ja"), or one with the same base ("zh-HK" -> "zh-TW"); else English.
 */
export function matchLanguage(wanted: readonly string[], available: readonly string[]): string {
  for (const tag of wanted) {
    const base = tag.split('-')[0];
    const hit =
      available.find((c) => c.toLowerCase() === tag.toLowerCase()) ??
      available.find((c) => c === base) ??
      available.find((c) => c.split('-')[0] === base);
    if (hit) return hit;
  }
  return 'en';
}

i18n.use(initReactI18next).init({
  // Copies: user files are merged into these, and the imported JSON must stay as built.
  resources: Object.fromEntries(
    Object.entries(BUILT_IN).map(([code, { translation }]) => [code, { translation: structuredClone(translation) }]),
  ),
  lng: matchLanguage(typeof navigator === 'undefined' ? [] : navigator.languages, Object.keys(BUILT_IN)),
  fallbackLng: 'en',
  interpolation: {
    escapeValue: false, // React already escapes by default
  },
});

export default i18n;
