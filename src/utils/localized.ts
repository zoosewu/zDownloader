// Text a module wrote for the user (manifest `label`, `help`): one string, or one per language.
import type { Text } from '../backend';

/** `text` in `lang` ("zh-TW"), else its base language ("zh"), else English, else the first given. */
export function localized(text: Text, lang: string): string {
  if (typeof text === 'string') return text;
  const base = lang.split('-')[0];
  const sameBase = Object.keys(text).find((tag) => tag.split('-')[0] === base);
  return text[lang] ?? text[base] ?? (sameBase && text[sameBase]) ?? text.en ?? Object.values(text)[0] ?? '';
}
