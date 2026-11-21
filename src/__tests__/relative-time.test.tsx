import { describe, test, expect, vi, afterEach } from 'vitest';
import { act, renderHook } from '@testing-library/react';
import { useRelativeTime } from '../hooks/useRelativeTime';

afterEach(() => vi.useRealTimers());

describe('useRelativeTime', () => {
  test('keeps up with the clock, re-rendering only when the wording changes', () => {
    vi.useFakeTimers();
    vi.setSystemTime(new Date('2026-11-22T10:00:00Z'));
    let renders = 0;
    const { result, unmount } = renderHook(() => {
      renders++;
      return useRelativeTime('2026-11-22T10:00:00Z', 'en');
    });
    expect(result.current).toBe('now');
    const before = renders;
    act(() => vi.advanceTimersByTime(30_000));
    expect(result.current).toBe('now');
    expect(renders).toBe(before);
    act(() => vi.advanceTimersByTime(3 * 60_000));
    expect(result.current).toBe('3 minutes ago');
    unmount();
    // The last row gone: the timer stops.
    expect(vi.getTimerCount()).toBe(0);
  });
});
