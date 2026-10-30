/**
 * Store merge rules: events and the list snapshot can arrive in any order, and the copy
 * with the higher `rev` must always win.
 */
import { describe, test, expect } from 'vitest';
import { applyEvent, mergeSnapshot, type TaskMap } from '../stores/downloadStore';
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
