import { describe, expect, test } from 'vitest';
import { emptyHeaderFields, fromHeaderFields, toHeaderFields } from '../utils/headers';
import en from '../locales/en.json';
import ja from '../locales/ja.json';
import zh from '../locales/zh-TW.json';

describe('task headers', () => {
  test('the three fields round-trip and other headers are kept', () => {
    const headers: Array<[string, string]> = [
      ['user-agent', 'netdisk'],
      ['X-Token', 't'],
      ['Cookie', 'sid=1'],
    ];
    const { fields, rest } = toHeaderFields(headers);
    expect(fields).toEqual({ userAgent: 'netdisk', referer: '', cookie: 'sid=1' });
    expect(rest).toEqual([['X-Token', 't']]);
    expect(fromHeaderFields({ ...fields, referer: ' https://a/ ' }, rest)).toEqual([
      ['User-Agent', 'netdisk'],
      ['Referer', 'https://a/'],
      ['Cookie', 'sid=1'],
      ['X-Token', 't'],
    ]);
  });

  test('empty fields send nothing', () => {
    expect(fromHeaderFields(emptyHeaderFields)).toEqual([]);
  });
});

/** Every key path of a translation tree. */
function keys(tree: object, prefix = ''): string[] {
  return Object.entries(tree).flatMap(([k, v]) =>
    v && typeof v === 'object' ? keys(v as object, `${prefix}${k}.`) : [`${prefix}${k}`]
  );
}

describe('translations', () => {
  test('every language has the same keys', () => {
    const reference = new Set(keys(zh));
    for (const [name, tree] of Object.entries({ en, ja })) {
      const own = new Set(keys(tree));
      const missing = [...reference].filter((k) => !own.has(k));
      const extra = [...own].filter((k) => !reference.has(k));
      expect({ name, missing, extra }).toEqual({ name, missing: [], extra: [] });
    }
  });
});
