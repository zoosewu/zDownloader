/** Pasted or dropped text becomes one row per http(s) link. */
import { describe, test, expect } from 'vitest';
import { linksIn } from '../utils/links';

describe('links in pasted text', () => {
  test('one per line, other text and schemes ignored', () => {
    const text = [
      'https://cdn.example.com/a.mp4',
      '  http://127.0.0.1:8080/b.bin  ',
      'not a link',
      'ftp://example.com/c.zip',
      'https://例子.測試/檔案.zip',
      '',
    ].join('\r\n');
    expect(linksIn(text)).toEqual(['https://cdn.example.com/a.mp4', 'http://127.0.0.1:8080/b.bin', 'https://例子.測試/檔案.zip']);
  });

  test('a browser drop (uri-list) and plain text together', () => {
    expect(linksIn('https://a.example/x\nhttps://b.example/y https://c.example/z')).toHaveLength(3);
    expect(linksIn('')).toEqual([]);
  });
});
