/**
 * Sidebar views: which tasks each status or category shows, their counts, and that download
 * progress leaves both untouched (so the sidebar and the filtered list do not re-render).
 */
import { describe, test, expect } from 'vitest';
import { createStore } from 'jotai';
import { applyEvent, tasksByIdAtom, type TaskMap } from '../stores/downloadStore';
import { settingsAtom } from '../stores/settingsStore';
import { canReorder, countViews, statusOf, viewAtom, viewCountsAtom, visibleTaskIdsAtom } from '../stores/viewStore';
import type { Task } from '../backend';

function task(id: string, order: number, overrides: Partial<Task> = {}): Task {
  return {
    id,
    url: `https://example.com/${id}`,
    name: id,
    customName: false,
    dir: '/downloads',
    headers: [],
    finalPath: null,
    state: 'queued',
    phase: null,
    error: null,
    downloaded: 0,
    total: 1000,
    speed: 0,
    createdAt: '2026-01-01T00:00:00Z',
    startedAt: null,
    finishedAt: null,
    scheduledAt: null,
    retryCount: 0,
    nextRetryAt: null,
    rev: 1,
    order,
    tags: [],
    customTags: false,
    module: null,
    ...overrides,
  };
}

const map = (...tasks: Task[]): TaskMap => Object.fromEntries(tasks.map((t) => [t.id, t]));

describe('status groups', () => {
  test('each state lands in one group; a retry that is coming counts as waiting', () => {
    expect(statusOf(task('a', 1, { state: 'scheduled' }))).toBe('waiting');
    expect(statusOf(task('a', 1, { state: 'failed', nextRetryAt: '2026-01-01T00:01:00Z' }))).toBe('waiting');
    expect(statusOf(task('a', 1, { state: 'failed' }))).toBe('failed');
    expect(statusOf(task('a', 1, { state: 'cancelled' }))).toBe('cancelled');
    expect(statusOf(task('a', 1, { state: 'running' }))).toBe('running');
  });

  test('counts per status and per category', () => {
    const counts = countViews(
      map(
        task('a', 1, { state: 'running', tags: ['video'] }),
        task('b', 2, { state: 'completed', tags: ['video', 'k2s'] }),
        task('c', 3, { state: 'cancelled' }),
      ),
    );
    expect(counts).toMatchObject({ all: 3, running: 1, completed: 1, cancelled: 1, waiting: 0, 'cat:video': 2, 'cat:k2s': 1 });
  });

  test('only the whole queue can be reordered', () => {
    expect([canReorder('all'), canReorder('waiting'), canReorder('completed'), canReorder('cat:x')]).toEqual([true, true, false, false]);
  });
});

describe('the selected view', () => {
  const setup = () => {
    const store = createStore();
    store.set(settingsAtom, { ...store.get(settingsAtom), categories: [{ id: 'video', name: 'Video', match: 'any', rules: [] }] });
    store.set(
      tasksByIdAtom,
      map(task('a', 1, { state: 'running', tags: ['video'] }), task('b', 2, { state: 'paused' }), task('c', 3, { state: 'running' })),
    );
    return store;
  };

  test('shows the tasks of a status or a category, newest (last in the queue) first', () => {
    const store = setup();
    store.set(viewAtom, 'running');
    expect(store.get(visibleTaskIdsAtom)).toEqual(['c', 'a']);
    store.set(viewAtom, 'cat:video');
    expect(store.get(visibleTaskIdsAtom)).toEqual(['a']);
  });

  test('a deleted category falls back to all', () => {
    const store = setup();
    store.set(viewAtom, 'cat:gone');
    expect(store.get(viewAtom)).toBe('all');
    expect(store.get(visibleTaskIdsAtom)).toEqual(['c', 'b', 'a']);
  });

  test('progress changes neither the list nor the counts', () => {
    const store = setup();
    store.set(viewAtom, 'running');
    const ids = store.get(visibleTaskIdsAtom);
    const counts = store.get(viewCountsAtom);
    store.set(tasksByIdAtom, applyEvent(store.get(tasksByIdAtom), { type: 'progress', data: [{ id: 'a', downloaded: 500, total: 1000, speed: 9 }] }));
    expect(store.get(visibleTaskIdsAtom)).toBe(ids);
    expect(store.get(viewCountsAtom)).toBe(counts);
  });

  test('a finished task leaves "running"', () => {
    const store = setup();
    store.set(viewAtom, 'running');
    store.set(tasksByIdAtom, applyEvent(store.get(tasksByIdAtom), { type: 'upserted', data: task('a', 1, { state: 'completed', rev: 2, tags: ['video'] }) }));
    expect(store.get(visibleTaskIdsAtom)).toEqual(['c']);
  });
});
