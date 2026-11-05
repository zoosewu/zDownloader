/**
 * The frontend's only way to talk to the backend.
 *
 * Types come from `./bindings` (generated from Rust by ts-rs; never edit them by hand).
 * Command names must match `generate_handler!` in src-tauri/src/lib.rs; a test checks this.
 * Every failure is thrown as a `BackendError` with a human-readable `message`.
 */
import { listen, type UnlistenFn } from '@tauri-apps/api/event';
import { invoke } from '../utils/tauri';
import type { AppSettings } from './bindings/AppSettings';
import type { BatchResult } from './bindings/BatchResult';
import type { Event as DownloadEvent } from './bindings/Event';
import type { ImportResult } from './bindings/ImportResult';
import type { ProxyInfo } from './bindings/ProxyInfo';
import type { StartMode } from './bindings/StartMode';
import type { SegmentMap } from './bindings/SegmentMap';
import type { Task } from './bindings/Task';
import type { TaskId } from './bindings/TaskId';
import type { TaskUpdate } from './bindings/TaskUpdate';

export type { AppSettings } from './bindings/AppSettings';
export type { BatchResult } from './bindings/BatchResult';
export type { CoreError } from './bindings/CoreError';
export type { Event as DownloadEvent } from './bindings/Event';
export type { ImportResult } from './bindings/ImportResult';
export type { ProgressTick } from './bindings/ProgressTick';
export type { ProxyInfo } from './bindings/ProxyInfo';
export type { ProxyStatus } from './bindings/ProxyStatus';
export type { SpeedLimitMode } from './bindings/SpeedLimitMode';
export type { StartMode } from './bindings/StartMode';
export type { SegmentMap } from './bindings/SegmentMap';
export type { ActiveRange } from './bindings/ActiveRange';
export type { Task } from './bindings/Task';
export type { TaskId } from './bindings/TaskId';
export type { TaskState } from './bindings/TaskState';
export type { TaskUpdate } from './bindings/TaskUpdate';
export type { Theme } from './bindings/Theme';

export class BackendError extends Error {
  constructor(public readonly code: string, message: string) {
    super(message);
    this.name = 'BackendError';
  }
}

/** Normalize whatever a command rejected with (CommandError object, string, Error). */
export function toBackendError(error: unknown): BackendError {
  if (error instanceof BackendError) return error;
  if (error && typeof error === 'object' && 'message' in error) {
    const { code, message } = error as { code?: unknown; message: unknown };
    return new BackendError(typeof code === 'string' ? code : 'unknown', String(message));
  }
  return new BackendError('unknown', String(error));
}

async function call<T>(command: string, args?: Record<string, unknown>): Promise<T> {
  try {
    return await invoke<T>(command, args);
  } catch (error) {
    throw toBackendError(error);
  }
}

/** Subscribe to a backend event; resolves to a no-op when events are unavailable (HTTP bridge). */
async function subscribe<T>(name: string, handler: (payload: T) => void): Promise<UnlistenFn> {
  try {
    return await listen<T>(name, (event) => handler(event.payload));
  } catch (error) {
    console.warn(`[backend] cannot listen for '${name}':`, error);
    return () => {};
  }
}

export interface AddDownload {
  url: string;
  filename?: string | null;
  /** Defaults to the download folder from settings. */
  dir?: string | null;
  start: StartMode;
  /** Sent with every request of the download (User-Agent, Referer, Cookie, ...). */
  headers?: Array<[string, string]> | null;
}

export const backend = {
  downloads: {
    add: ({ url, filename = null, dir = null, start, headers = null }: AddDownload) =>
      call<Task>('downloads_add', { url, filename, dir, start, headers }),
    importBatch: (filePath: string, start: StartMode, dir: string | null = null) =>
      call<BatchResult>('downloads_import_batch', { filePath, dir, start }),
    list: () => call<Task[]>('downloads_list'),
    pause: (id: TaskId) => call<Task>('downloads_pause', { id }),
    resume: (id: TaskId) => call<Task>('downloads_resume', { id }),
    cancel: (id: TaskId) => call<Task>('downloads_cancel', { id }),
    retry: (id: TaskId) => call<Task>('downloads_retry', { id }),
    /** Discard the partial data and download from the start. */
    restart: (id: TaskId) => call<Task>('downloads_restart', { id }),
    /** Change the URL or headers of a task that is not running. */
    update: (id: TaskId, update: TaskUpdate) => call<Task>('downloads_update', { id, update }),
    remove: (id: TaskId) => call<null>('downloads_remove', { id }),
    pauseAll: () => call<Task[]>('downloads_pause_all'),
    resumeAll: () => call<Task[]>('downloads_resume_all'),
    clearFinished: () => call<TaskId[]>('downloads_clear_finished'),
    segments: (id: TaskId) => call<SegmentMap>('downloads_segments', { id }),
  },
  settings: {
    get: () => call<AppSettings>('settings_get'),
    update: (settings: AppSettings) => call<AppSettings>('settings_update', { settings }),
    export: (path: string) => call<null>('settings_export', { path }),
    import: (path: string) => call<AppSettings>('settings_import', { path }),
    reset: () => call<AppSettings>('settings_reset'),
  },
  proxies: {
    list: () => call<ProxyInfo[]>('proxies_list'),
    import: (filePath: string) => call<ImportResult>('proxies_import', { filePath }),
    remove: (id: string) => call<null>('proxies_remove', { id }),
    clear: () => call<null>('proxies_clear'),
    resetFailed: () => call<null>('proxies_reset_failed'),
  },
  events: {
    onDownloadEvent: (handler: (event: DownloadEvent) => void) => subscribe('download-event', handler),
    /** The backend dropped events; refetch the full list. */
    onResync: (handler: () => void) => subscribe<null>('downloads-resync', () => handler()),
  },
};

export type Backend = typeof backend;
