// "5 minutes ago" that keeps up with the clock. One timer for every row; a row re-renders only
// when its wording changes, so most ticks re-render nothing.
import { useSyncExternalStore } from 'react';
import { formatRelativeTime } from '../utils/formatters';

const TICK_MS = 15_000;
const listeners = new Set<() => void>();
let now = Date.now();
let timer: ReturnType<typeof setInterval> | undefined;

function subscribe(listener: () => void): () => void {
  listeners.add(listener);
  if (timer === undefined) {
    now = Date.now();
    timer = setInterval(() => {
      now = Date.now();
      listeners.forEach((l) => l());
    }, TICK_MS);
  }
  return () => {
    listeners.delete(listener);
    if (listeners.size === 0) {
      clearInterval(timer);
      timer = undefined;
    }
  };
}

/** How long ago `isoString` was, in `lang`, refreshed as time passes. */
export function useRelativeTime(isoString: string, lang: string): string {
  return useSyncExternalStore(subscribe, () => formatRelativeTime(isoString, lang, now));
}
