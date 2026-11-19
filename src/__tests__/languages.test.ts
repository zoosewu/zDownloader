import { beforeEach, describe, expect, test, vi } from 'vitest';
import { getDefaultStore } from 'jotai';
import i18n, { matchLanguage } from '../i18n';
import { backend } from '../backend';
import { languageName, languagesAtom, loadUserLocales, localeProblemsAtom, resolveLanguage } from '../utils/languages';

const builtIn = ['zh-TW', 'en', 'ja'];

describe('matchLanguage', () => {
  test('the same tag, then its base, then one with the same base', () => {
    expect(matchLanguage(['ja-JP'], builtIn)).toBe('ja');
    expect(matchLanguage(['zh-tw'], builtIn)).toBe('zh-TW');
    expect(matchLanguage(['zh-HK'], builtIn)).toBe('zh-TW');
    expect(matchLanguage(['en-GB'], builtIn)).toBe('en');
  });

  test('the first preferred language we have, else English', () => {
    expect(matchLanguage(['fr-FR', 'ja', 'en'], builtIn)).toBe('ja');
    expect(matchLanguage(['fr-FR', 'de'], builtIn)).toBe('en');
    expect(matchLanguage([], builtIn)).toBe('en');
    expect(matchLanguage(['fr-CA'], [...builtIn, 'fr'])).toBe('fr');
  });
});

test('a setting we have is used; "" or a missing one follows the system', () => {
  const system = navigator.languages;
  expect(resolveLanguage('ja', builtIn)).toBe('ja');
  expect(resolveLanguage('', builtIn)).toBe(matchLanguage(system, builtIn));
  expect(resolveLanguage('fr', builtIn)).toBe(matchLanguage(system, builtIn));
});

test("a language's name: its _name, else the built-in name, else what the system calls it", () => {
  expect(languageName('fr', { _name: 'Français (QC)' })).toBe('Français (QC)');
  expect(languageName('ja', {})).toBe('日本語');
  expect(languageName('fr', { _name: ' ' }).toLowerCase()).toBe('français');
});

describe('loadUserLocales', () => {
  beforeEach(async () => {
    await i18n.changeLanguage('en');
  });

  test('adds languages, overrides only the keys a file has, and reports broken files', async () => {
    vi.spyOn(backend.locales, 'list').mockResolvedValue([
      { code: 'fr', text: '﻿{"_name": "Français", "settings": {"language": "Langue"}}' },
      { code: 'en', text: '{"settings": {"language": "Tongue"}}' },
      { code: 'de', text: '{"settings": ' },
      { code: 'it', text: '["not", "an", "object"]' },
    ]);
    await loadUserLocales();

    const store = getDefaultStore();
    expect(store.get(languagesAtom).map((l) => l.code)).toEqual([...builtIn, 'fr']);
    expect(store.get(localeProblemsAtom).map((p) => p.file)).toEqual(['de.json', 'it.json']);
    expect(i18n.t('settings.language')).toBe('Tongue');
    expect(i18n.t('settings.theme')).toBe('Theme'); // not in the file: the built-in text
    expect(i18n.t('settings.language', { lng: 'fr' })).toBe('Langue');
    expect(i18n.t('settings.theme', { lng: 'fr' })).toBe('Theme'); // missing: English

    // The files change: removed overrides and languages are gone.
    vi.spyOn(backend.locales, 'list').mockResolvedValue([]);
    await loadUserLocales();
    expect(store.get(languagesAtom).map((l) => l.code)).toEqual(builtIn);
    expect(store.get(localeProblemsAtom)).toEqual([]);
    expect(i18n.t('settings.language')).toBe('Language');
    expect(i18n.hasResourceBundle('fr', 'translation')).toBe(false);
  });
});
