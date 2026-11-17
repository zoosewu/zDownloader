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
import type { CaptchaEvent } from './bindings/CaptchaEvent';
import type { CaptchaRequest } from './bindings/CaptchaRequest';
import type { InstalledInfo } from './bindings/InstalledInfo';
import type { PlanSummary } from './bindings/PlanSummary';
import type { SourceInfo } from './bindings/SourceInfo';
import type { AddItem } from './bindings/AddItem';
import type { Event as DownloadEvent } from './bindings/Event';
import type { ImportResult } from './bindings/ImportResult';
import type { ProxyInfo } from './bindings/ProxyInfo';
import type { StartMode } from './bindings/StartMode';
import type { SegmentMap } from './bindings/SegmentMap';
import type { Task } from './bindings/Task';
import type { TaskId } from './bindings/TaskId';
import type { TaskUpdate } from './bindings/TaskUpdate';
import type { ListedFile } from './bindings/ListedFile';
import type { UrlInfo } from './bindings/UrlInfo';

export type { AppSettings } from './bindings/AppSettings';
export type { AddItem } from './bindings/AddItem';
export type { CaptchaChallenge } from './bindings/CaptchaChallenge';
export type { CaptchaEvent } from './bindings/CaptchaEvent';
export type { CaptchaRequest } from './bindings/CaptchaRequest';
export type { CatalogModule } from './bindings/CatalogModule';
export type { CredentialStatus } from './bindings/CredentialStatus';
export type { InstalledInfo } from './bindings/InstalledInfo';
export type { PermissionSummary } from './bindings/PermissionSummary';
export type { PlanSummary } from './bindings/PlanSummary';
export type { SourceInfo } from './bindings/SourceInfo';
export type { Trust } from './bindings/Trust';
export type { ListedFile } from './bindings/ListedFile';
export type { UrlInfo } from './bindings/UrlInfo';
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
export type { Category } from './bindings/Category';
export type { Matching } from './bindings/Matching';
export type { Rule } from './bindings/Rule';
export type { Target } from './bindings/Target';
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

/** Subscribe to a backend event; resolves to a no-op when events are unavailable (Tauri not ready). */
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
  /** Categories chosen by the user; `null` lets the category rules decide. */
  tags?: string[] | null;
  /** Folder inside a share (`a/b`), created under `dir`. */
  folder?: string | null;
}

export const backend = {
  downloads: {
    add: ({ url, filename = null, dir = null, start, headers = null, tags = null, folder = null }: AddDownload) =>
      call<Task>('downloads_add', { url, filename, dir, start, headers, tags, folder }),
    list: () => call<Task[]>('downloads_list'),
    pause: (id: TaskId) => call<Task>('downloads_pause', { id }),
    resume: (id: TaskId) => call<Task>('downloads_resume', { id }),
    cancel: (id: TaskId) => call<Task>('downloads_cancel', { id }),
    retry: (id: TaskId) => call<Task>('downloads_retry', { id }),
    /** Discard the partial data and download from the start. */
    restart: (id: TaskId) => call<Task>('downloads_restart', { id }),
    /** Put the task in exactly these categories; their rules no longer change it. */
    setTags: (id: TaskId, tags: string[]) => call<Task>('downloads_set_tags', { id, tags }),
    /** Change the URL or headers of a task that is not running. */
    update: (id: TaskId, update: TaskUpdate) => call<Task>('downloads_update', { id, update }),
    /** `deleteFile`: also move a finished download's file to the recycle bin. */
    remove: (id: TaskId, deleteFile = false) => call<null>('downloads_remove', { id, deleteFile }),
    /** Open a finished download with its default app. */
    openFile: (id: TaskId) => call<null>('downloads_open_file', { id }),
    /** Show a finished download in its folder. */
    showInFolder: (id: TaskId) => call<null>('downloads_show_in_folder', { id }),
    /** Move a task in front of `before` (or to the end); queued tasks start in list order. */
    move: (id: TaskId, before: TaskId | null) => call<Task>('downloads_move', { id, before }),
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
  categories: {
    /** The categories the rules give this download (to pre-fill the add window). */
    match: (url: string, filename: string | null = null) => call<string[]>('categories_match', { url, filename }),
  },
  app: {
    /** Stop downloads and save their state; call right before installing an update. */
    prepareUpdate: () => call<null>('app_prepare_update'),
  },
  addWindow: {
    /** Show the add-downloads window with these links added to its list. */
    open: (items: AddItem[] = []) => call<null>('add_window_open', { items }),
    /** Links waiting for the add window (each is handed out once). */
    take: () => call<AddItem[]>('add_window_take'),
    /** A text file, one link per line, into the add window; resolves to how many were read. */
    import: (filePath: string) => call<number>('add_window_import', { filePath }),
  },
  clipboard: {
    /** Copy text; the clipboard watcher does not offer the app's own copy back as a download. */
    copy: (text: string) => call<null>('clipboard_copy', { text }),
  },
  urls: {
    /** File name, resolving module, and whether it is worth offering as a download. */
    inspect: (url: string) => call<UrlInfo>('urls_inspect', { url }),
    /** The files behind a link: several for a shared folder, else the link itself. Runs its module. */
    list: (url: string) => call<ListedFile[]>('urls_list', { url }),
  },
  modules: {
    list: () => call<InstalledInfo[]>('modules_list'),
    sources: () => call<SourceInfo[]>('modules_sources'),
    /** `owner/repo`, an https:// catalog URL, a git URL or a folder path. */
    addSource: (location: string) => call<SourceInfo>('modules_add_source', { location }),
    removeSource: (name: string) => call<null>('modules_remove_source', { name }),
    /** Fetch every catalog again and look for updates. */
    refresh: () => call<SourceInfo[]>('modules_refresh'),
    /** Download and verify (install or update); show the summary, then `install` or `cancelInstall`. */
    planInstall: (key: string) => call<PlanSummary>('modules_plan_install', { key }),
    install: (key: string) => call<InstalledInfo>('modules_install', { key }),
    cancelInstall: (key: string) => call<null>('modules_cancel_install', { key }),
    remove: (key: string) => call<null>('modules_remove', { key }),
    setEnabled: (key: string, enabled: boolean) => call<null>('modules_set_enabled', { key, enabled }),
    /** Links this module resolves start at once (only with `autoStartLinks` on). */
    setAutoStart: (key: string, autoStart: boolean) => call<null>('modules_set_auto_start', { key, autoStart }),
    /** Move to `index` in the priority order. */
    move: (key: string, index: number) => call<null>('modules_move', { key, index }),
    /** `null` deletes a stored value. */
    setCredentials: (key: string, values: Record<string, string | null>) =>
      call<InstalledInfo>('modules_set_credentials', { key, values }),
  },
  captcha: {
    pending: () => call<CaptchaRequest[]>('captcha_pending'),
    /** `null` declines. */
    answer: (id: string, answer: string | null) => call<null>('captcha_answer', { id, answer }),
    /** reCAPTCHA: open the site's page in its own window; solving it answers. */
    openWindow: (id: string) => call<null>('captcha_open_window', { id }),
  },
  events: {
    onModulesChanged: (handler: () => void) => subscribe<null>('modules-changed', () => handler()),
    onCaptchaEvent: (handler: (event: CaptchaEvent) => void) => subscribe('captcha-event', handler),
    onDownloadEvent: (handler: (event: DownloadEvent) => void) => subscribe('download-event', handler),
    /** The backend dropped events; refetch the full list. */
    onResync: (handler: () => void) => subscribe<null>('downloads-resync', () => handler()),
    /** A single http(s) link was copied (sent only while clipboard monitoring is enabled). */
    onClipboardLink: (handler: (url: string) => void) => subscribe('clipboard-link', handler),
    /** Links are waiting for the add window (`addWindow.take`). */
    onAddItemsReady: (handler: () => void) => subscribe<null>('add-items-ready', () => handler()),
  },
};

export type Backend = typeof backend;
