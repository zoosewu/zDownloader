import { describe, expect, test } from 'vitest';
import type { Task } from '../backend/bindings/Task';
import { errorText } from '../components/DownloadTableRow';
import { statusLabel } from '../utils/downloadCalculations';

const t = (key: string) => key;

function task(overrides: Partial<Task>): Task {
  return {
    id: '00000000-0000-0000-0000-000000000000',
    url: 'https://share.example/f/abc',
    name: 'f.bin',
    customName: false,
    dir: '/tmp',
    finalPath: null,
    state: 'running',
    phase: null,
    error: null,
    downloaded: 0,
    total: null,
    speed: 0,
    createdAt: '2026-11-04T00:00:00Z',
    startedAt: null,
    finishedAt: null,
    scheduledAt: null,
    retryCount: 0,
    nextRetryAt: null,
    rev: 1,
    ...overrides,
  };
}

describe('task status', () => {
  test('a running task shows its phase until it transfers', () => {
    expect(statusLabel(task({ phase: { kind: 'resolving', module: 'k2s@official' } }), t)).toBe('downloads.phase.resolving');
    expect(statusLabel(task({ phase: { kind: 'waiting', until: '2026-11-04T00:01:00Z', reason: null } }), t)).toBe('downloads.phase.waiting');
    expect(statusLabel(task({ phase: { kind: 'awaitingUser' } }), t)).toBe('downloads.phase.awaitingUser');
    expect(statusLabel(task({ phase: { kind: 'transferring' } }), t)).toBe('downloads.state.running');
    expect(statusLabel(task({ phase: null }), t)).toBe('downloads.state.running');
  });

  test('a stale phase on a stopped task is ignored', () => {
    expect(statusLabel(task({ state: 'paused', phase: { kind: 'awaitingUser' } }), t)).toBe('downloads.state.paused');
  });

  test('module errors show their message and hint', () => {
    const failed = task({
      state: 'failed',
      error: { kind: 'resolve', detail: { message: 'k2s@official: file deleted', hint: 'ask for a new link', retryable: false } },
    });
    expect(errorText(failed)).toBe('k2s@official: file deleted (ask for a new link)');
    expect(errorText(task({ state: 'failed', error: { kind: 'http', detail: 404 } }))).toBe('http: 404');
  });
});
