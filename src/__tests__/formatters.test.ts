/** Times read in the app's language, not always Chinese. */
import { describe, expect, test } from 'vitest';
import { formatAbsoluteTime, formatDuration, formatRelativeTime } from '../utils/formatters';

const now = Date.parse('2026-11-17T12:00:00Z');
const ago = (seconds: number) => new Date(now - seconds * 1000).toISOString();

describe('relative time', () => {
  test('in each language', () => {
    expect(formatRelativeTime(ago(18 * 60), 'en', now)).toBe('18 minutes ago');
    expect(formatRelativeTime(ago(18 * 60), 'zh-TW', now)).toBe('18 分鐘前');
    expect(formatRelativeTime(ago(18 * 60), 'ja', now)).toBe('18 分前');
    expect(formatRelativeTime(ago(30 * 3600), 'en', now)).toBe('yesterday');
    expect(formatRelativeTime(ago(30 * 3600), 'zh-TW', now)).toBe('昨天');
    expect(formatRelativeTime(ago(10), 'en', now)).toBe('now');
  });
});

describe('durations', () => {
  test('two units at most, in each language', () => {
    expect(formatDuration(177, 'en')).toBe('2m 57s');
    expect(formatDuration(4500, 'en')).toBe('1h 15m');
    expect(formatDuration(40, 'en')).toBe('40s');
    expect(formatDuration(177, 'zh-TW')).toMatch(/^2\s?分鐘 57\s?秒$/);
    expect(formatDuration(null, 'en')).toBe('--');
  });
});

test('absolute time follows the language', () => {
  const iso = '2026-11-17T06:05:00Z';
  expect(formatAbsoluteTime(iso, 'en')).not.toBe(formatAbsoluteTime(iso, 'zh-TW'));
  expect(formatAbsoluteTime('not a date', 'en')).toBe('--');
});
