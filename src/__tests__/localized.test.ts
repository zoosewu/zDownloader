import { describe, expect, test } from 'vitest';
import { localized } from '../utils/localized';

describe('localized', () => {
  const text = { en: 'Password', 'zh-TW': '密碼', ja: 'パスワード' };

  test('a plain string is the same in every language', () => {
    expect(localized('BDUSS', 'ja')).toBe('BDUSS');
  });

  test('the user language, then the same base language, then English, then the first', () => {
    expect(localized(text, 'zh-TW')).toBe('密碼');
    expect(localized({ en: 'Password', zh: '密码' }, 'zh-TW')).toBe('密码');
    expect(localized({ en: 'Password', 'zh-CN': '密码' }, 'zh-TW')).toBe('密码');
    expect(localized(text, 'fr')).toBe('Password');
    expect(localized({ ja: 'パスワード', de: 'Passwort' }, 'fr')).toBe('パスワード');
  });
});
