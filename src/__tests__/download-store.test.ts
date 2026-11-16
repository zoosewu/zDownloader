/**
 * Store merge rules: events and the list snapshot can arrive in any order, and the copy
 * with the higher `rev` must always win.
 */
import { describe, test, expect } from 'vitest';
import { createStore } from 'jotai';
import {
  applyEvent,
  mergeSnapshot,
  sourceChangedTasksAtom,
  taskAtom,
  taskCountsAtom,
  taskIdsAtom,
  tasksByIdAtom,
  type TaskMap,
} from '../stores/downloadStore';
import { toBackendError, BackendError, type Task } from '../backend';

function task(id: string, rev: number, overrides: Partial<Task> = {}): Task {
  return {
    id,
    url: `https://example.com/${id}`,
    name: id,
    dir: '/downloads',
    finalPath: null,
    state: 'queued',
    error: null,
    downloaded: 0,
    total: 1000,
    speed: 0,
    createdAt: `2026-01-01T00:00:0${rev % 10}Z`,
    startedAt: null,
    finishedAt: null,
    scheduledAt: null,
    retryCount: 0,
    nextRetryAt: null,
    rev,
    order: 1,
    tags: [],
    customTags: false,
    ...overrides,
  };
}

const map = (...tasks: Task[]): TaskMap => Object.fromEntries(tasks.map((t) => [t.id, t]));

describe('applyEvent', () => {
  test('upsert keeps the newer revision', () => {
    const running = task('a', 5, { state: 'running' });
    const tasks = map(running);
    const stale = applyEvent(tasks, { type: 'upserted', data: task('a', 4, { state: 'queued' }) });
    expect(stale).toBe(tasks);
    const fresh = applyEvent(tasks, { type: 'upserted', data: task('a', 6, { state: 'completed' }) });
    expect(fresh.a.state).toBe('completed');
  });

  test('unknown tasks are added, not dropped', () => {
    const next = applyEvent({}, { type: 'upserted', data: task('new', 1) });
    expect(Object.keys(next)).toEqual(['new']);
  });

  test('progress updates counters of known tasks only', () => {
    const tasks = map(task('a', 1, { state: 'running' }));
    const next = applyEvent(tasks, {
      type: 'progress',
      data: [
        { id: 'a', downloaded: 500, total: 1000, speed: 42 },
        { id: 'ghost', downloaded: 1, total: null, speed: 1 },
      ],
    });
    expect(next.a).toMatchObject({ downloaded: 500, speed: 42, rev: 1, state: 'running' });
    expect(next.ghost).toBeUndefined();
  });

  test('removed deletes the task', () => {
    expect(applyEvent(map(task('a', 1)), { type: 'removed', data: 'a' })).toEqual({});
  });
});

describe('mergeSnapshot', () => {
  test('per task, the higher revision wins in either direction', () => {
    const local = map(task('a', 10, { state: 'completed' }), task('b', 2, { state: 'queued' }));
    const snapshot = [task('a', 7, { state: 'running' }), task('b', 9, { state: 'running' })];
    const merged = mergeSnapshot(local, snapshot);
    expect(merged.a.state).toBe('completed'); // event arrived after the snapshot was taken
    expect(merged.b.state).toBe('running'); // snapshot is newer than what we had
  });

  test('tasks missing from the snapshot are dropped unless added after it', () => {
    const local = map(task('removed-before', 3), task('added-after', 12));
    const merged = mergeSnapshot(local, [task('a', 10)]);
    expect(Object.keys(merged).sort()).toEqual(['a', 'added-after']);
  });
});

describe('toBackendError', () => {
  test('keeps code and message from a CommandError', () => {
    const e = toBackendError({ code: 'http', message: 'HTTP 404' });
    expect(e).toBeInstanceOf(BackendError);
    expect([e.code, e.message]).toEqual(['http', 'HTTP 404']);
  });

  test('turns strings and Errors into readable messages', () => {
    expect(toBackendError('boom').message).toBe('boom');
    expect(toBackendError(new Error('bad')).message).toBe('bad');
  });
});

describe('store views re-render only what changed', () => {
  // A progress tick must notify the views showing that task and nothing else: these
  // references are what React compares to skip re-rendering the table, other rows and the page.
  const setup = () => {
    const store = createStore();
    store.set(tasksByIdAtom, map(task('a', 1, { state: 'running', order: 1 }), task('b', 1, { state: 'paused', order: 2 })));
    const before = {
      ids: store.get(taskIdsAtom),
      a: store.get(taskAtom('a')),
      b: store.get(taskAtom('b')),
      counts: store.get(taskCountsAtom),
      changed: store.get(sourceChangedTasksAtom),
    };
    const tick = (downloaded: number, speed: number) =>
      store.set(tasksByIdAtom, applyEvent(store.get(tasksByIdAtom), { type: 'progress', data: [{ id: 'a', downloaded, total: 1000, speed }] }));
    return { store, before, tick };
  };

  test('a tick with the same numbers changes nothing', () => {
    const tasks = map(task('a', 1, { state: 'running', downloaded: 10, speed: 5 }));
    expect(applyEvent(tasks, { type: 'progress', data: [{ id: 'a', downloaded: 10, total: 1000, speed: 5 }] })).toBe(tasks);
  });

  test('a tick changes the task and the total speed only', () => {
    const { store, before, tick } = setup();
    tick(100, 7);
    expect(store.get(taskAtom('a'))).not.toBe(before.a);
    expect(store.get(taskIdsAtom)).toBe(before.ids);
    expect(store.get(taskAtom('b'))).toBe(before.b);
    expect(store.get(sourceChangedTasksAtom)).toBe(before.changed);
    expect(store.get(taskCountsAtom)).toEqual({ ...before.counts, speed: 7 });
  });

  test('counts keep their reference while only bytes change', () => {
    const { store, tick } = setup();
    tick(100, 7);
    const counts = store.get(taskCountsAtom);
    tick(200, 7);
    expect(store.get(taskCountsAtom)).toBe(counts);
  });

  test('the order view changes on add, remove and move', () => {
    const { store, before } = setup();
    store.set(tasksByIdAtom, applyEvent(store.get(tasksByIdAtom), { type: 'upserted', data: task('c', 1, { order: 0 }) }));
    const added = store.get(taskIdsAtom);
    expect(added).not.toBe(before.ids);
    expect(added).toEqual(['c', 'a', 'b']);
    store.set(tasksByIdAtom, applyEvent(store.get(tasksByIdAtom), { type: 'removed', data: 'a' }));
    expect(store.get(taskIdsAtom)).toEqual(['c', 'b']);
  });
});
