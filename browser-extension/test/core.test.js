import { describe, expect, test, vi } from 'vitest';
import { readFileSync } from 'node:fs';
import { createHash } from 'node:crypto';
import { join } from 'node:path';
import {
  addRequest,
  createMenu,
  createTakeover,
  errorText,
  onMenuClick,
  send,
  shouldTakeOver,
  statusKey,
  FALLBACK_MS,
  HOST,
  MENU_ID,
} from '../src/core.js';

const dir = join(import.meta.dirname, '..');
const read = (path) => JSON.parse(readFileSync(join(dir, path), 'utf8'));

/** A fake `chrome` whose native host answers with `answer` (or fails with `lastError`). */
function fakeChrome({ answer = { ok: true, name: 'a.zip' }, lastError, cookies = [], storage = {} } = {}) {
  const chrome = {
    sent: [],
    runtime: {
      lastError: undefined,
      sendNativeMessage: vi.fn((host, message, callback) => {
        chrome.sent.push({ host, message });
        chrome.runtime.lastError = lastError ? { message: lastError } : undefined;
        callback(lastError ? undefined : answer);
        chrome.runtime.lastError = undefined;
      }),
    },
    cookies: { getAll: vi.fn(async () => cookies) },
    storage: { local: { get: vi.fn(async (key) => (key in storage ? { [key]: storage[key] } : {})), set: vi.fn() } },
    i18n: { getMessage: vi.fn((key, subs) => `${key}${subs?.length ? `(${subs.join(',')})` : ''}`) },
    contextMenus: { removeAll: vi.fn((done) => done()), create: vi.fn() },
    action: { setBadgeText: vi.fn(), setBadgeBackgroundColor: vi.fn() },
    notifications: { create: vi.fn() },
    downloads: { cancel: vi.fn(async () => {}), erase: vi.fn(async () => {}) },
  };
  return chrome;
}

const item = (extra = {}) => ({
  id: 7,
  url: 'https://site.example/get?id=1',
  finalUrl: 'https://cdn.example/files/a.zip',
  filename: 'C:\\Users\\me\\Downloads\\a.zip',
  referrer: 'https://site.example/page',
  incognito: false,
  ...extra,
});

/** Run the takeover listener on `downloadItem`; resolves once it suggested or erased. */
async function takeOver(chrome, downloadItem, options = {}) {
  const suggest = vi.fn();
  const listener = options.listener ?? createTakeover(chrome, { userAgent: 'UA', ...options });
  expect(listener(downloadItem, suggest)).toBe(true);
  await vi.waitFor(() => expect(suggest.mock.calls.length + chrome.downloads.erase.mock.calls.length).toBeGreaterThan(0));
  await new Promise((r) => setTimeout(r, 0));
  return suggest;
}

describe('manifest', () => {
  test('has the key that makes the extension ID the app trusts', () => {
    const { key } = read('manifest.json');
    const hex = createHash('sha256').update(Buffer.from(key, 'base64')).digest('hex').slice(0, 32);
    const id = [...hex].map((h) => String.fromCharCode(97 + parseInt(h, 16))).join('');
    const rust = readFileSync(join(dir, '..', 'src-tauri', 'crates', 'zoo-native-host', 'src', 'lib.rs'), 'utf8');
    expect(rust).toContain(`pub const EXTENSION_ID: &str = "${id}";`);
  });

  test('asks only for what it uses and injects nothing into pages', () => {
    const manifest = read('manifest.json');
    expect(manifest.permissions.sort()).toEqual(['contextMenus', 'cookies', 'downloads', 'nativeMessaging', 'notifications', 'storage']);
    expect(manifest.content_scripts).toBeUndefined();
  });

  test('every locale has every message', () => {
    const en = Object.keys(read('_locales/en/messages.json')).sort();
    for (const locale of ['zh_TW', 'ja']) expect(Object.keys(read(`_locales/${locale}/messages.json`)).sort()).toEqual(en);
  });
});

describe('send', () => {
  test('adds the protocol version and passes the answer through', async () => {
    const chrome = fakeChrome();
    expect(await send(chrome, { type: 'ping' })).toEqual({ ok: true, name: 'a.zip' });
    expect(chrome.sent).toEqual([{ host: HOST, message: { v: 1, type: 'ping' } }]);
  });

  test('a host the browser cannot find means the app is not installed', async () => {
    const chrome = fakeChrome({ lastError: 'Specified native messaging host not found.' });
    expect(await send(chrome, { type: 'ping' })).toMatchObject({ ok: false, code: 'notInstalled' });
    const crashed = fakeChrome({ lastError: 'Native host has exited.' });
    expect(await send(crashed, { type: 'ping' })).toMatchObject({ ok: false, code: 'internal' });
  });
});

describe('addRequest', () => {
  test("joins the site's cookies and leaves out what is missing", async () => {
    const chrome = fakeChrome({ cookies: [{ name: 'a', value: '1' }, { name: 'sid', value: 'x y' }] });
    const request = await addRequest(chrome, { url: 'https://a/f.zip', referer: 'https://a/', userAgent: 'UA' });
    expect(request).toEqual({ type: 'add', url: 'https://a/f.zip', referer: 'https://a/', userAgent: 'UA', cookie: 'a=1; sid=x y' });
    expect(chrome.cookies.getAll).toHaveBeenCalledWith({ url: 'https://a/f.zip' });
  });

  test('never reads cookies for incognito windows', async () => {
    const chrome = fakeChrome({ cookies: [{ name: 'a', value: '1' }] });
    expect(await addRequest(chrome, { url: 'https://a/', incognito: true })).toEqual({ type: 'add', url: 'https://a/' });
    expect(chrome.cookies.getAll).not.toHaveBeenCalled();
  });
});

describe('popup status and messages', () => {
  test('maps answers to status lines', () => {
    expect(statusKey({ ok: true })).toBe('statusConnected');
    expect(statusKey({ ok: false, code: 'notInstalled' })).toBe('statusNotInstalled');
    expect(statusKey({ ok: false, code: 'appNotRunning' })).toBe('statusNotRunning');
    expect(statusKey({ ok: false, code: 'unsupportedVersion' })).toBe('statusVersion');
    expect(statusKey({ ok: false, code: 'internal' })).toBe('statusError');
  });

  test('error text carries the app message', () => {
    expect(errorText(fakeChrome(), { ok: false, code: 'invalid', message: 'bad URL' })).toBe('error_invalid(bad URL)');
  });
});

describe('context menu', () => {
  test('is created once, for links and media', () => {
    const chrome = fakeChrome();
    createMenu(chrome);
    expect(chrome.contextMenus.removeAll).toHaveBeenCalledOnce();
    expect(chrome.contextMenus.create).toHaveBeenCalledOnce();
    expect(chrome.contextMenus.create.mock.calls[0][0]).toMatchObject({ id: MENU_ID, contexts: ['link', 'image', 'video', 'audio'] });
  });

  test('sends the link with the page as referer and flashes the badge', async () => {
    vi.useFakeTimers();
    const chrome = fakeChrome({ cookies: [{ name: 's', value: '1' }] });
    await onMenuClick(chrome, { menuItemId: MENU_ID, linkUrl: 'https://a/f.zip', pageUrl: 'https://a/page' }, { incognito: false }, 'UA');
    expect(chrome.sent[0].message).toEqual({ v: 1, type: 'add', url: 'https://a/f.zip', referer: 'https://a/page', userAgent: 'UA', cookie: 's=1' });
    expect(chrome.action.setBadgeText).toHaveBeenLastCalledWith({ text: '✓' });
    vi.advanceTimersByTime(2000);
    expect(chrome.action.setBadgeText).toHaveBeenLastCalledWith({ text: '' });
    vi.useRealTimers();
  });

  test('uses the media URL and a frame referer; failures become a notification', async () => {
    const chrome = fakeChrome({ answer: { ok: false, code: 'invalid', message: 'nope' } });
    const info = { menuItemId: MENU_ID, srcUrl: 'https://a/v.mp4', frameUrl: 'https://a/frame', pageUrl: 'https://a/page' };
    await onMenuClick(chrome, info, { incognito: true }, 'UA');
    expect(chrome.sent[0].message).toMatchObject({ url: 'https://a/v.mp4', referer: 'https://a/frame' });
    expect(chrome.cookies.getAll).not.toHaveBeenCalled();
    expect(chrome.notifications.create.mock.calls[0][0].message).toBe('error_invalid(nope)');
  });
});

describe('takeover', () => {
  const base = { enabled: true, fallbacks: new Map(), now: 0 };

  test.each([
    ['turned off', item(), { enabled: false }],
    ['incognito', item({ incognito: true }), {}],
    ['started by an extension', item({ byExtensionId: 'abc' }), {}],
    ['blob URL', item({ url: 'blob:https://a/1', finalUrl: 'blob:https://a/1' }), {}],
    ['data URL', item({ url: 'data:text/plain,hi', finalUrl: 'data:text/plain,hi' }), {}],
    ['file URL', item({ url: 'file:///C:/a.zip', finalUrl: 'file:///C:/a.zip' }), {}],
    ['just fell back', item(), { fallbacks: new Map([['https://cdn.example/files/a.zip', 10]]), now: 5 }],
  ])('leaves the download to the browser when %s', (_, downloadItem, options) => {
    expect(shouldTakeOver(downloadItem, { ...base, ...options })).toBe(false);
  });

  test('takes over plain http(s) downloads, and a fallback expires', () => {
    expect(shouldTakeOver(item(), base)).toBe(true);
    expect(shouldTakeOver(item({ finalUrl: undefined, url: 'http://a/b' }), base)).toBe(true);
    expect(shouldTakeOver(item(), { ...base, fallbacks: new Map([['https://cdn.example/files/a.zip', 10]]), now: 10 })).toBe(true);
  });

  test('sends the final URL, file name and referer, then removes it from the browser', async () => {
    const chrome = fakeChrome({ cookies: [{ name: 'k', value: 'v' }] });
    const suggest = await takeOver(chrome, item());
    expect(chrome.sent[0].message).toEqual({
      v: 1,
      type: 'add',
      url: 'https://cdn.example/files/a.zip',
      filename: 'a.zip',
      referer: 'https://site.example/page',
      userAgent: 'UA',
      cookie: 'k=v',
    });
    expect(chrome.downloads.cancel).toHaveBeenCalledWith(7);
    expect(chrome.downloads.erase).toHaveBeenCalledWith({ id: 7 });
    expect(suggest.mock.calls.length).toBeLessThanOrEqual(1);
  });

  test('is on unless turned off', async () => {
    const off = fakeChrome({ storage: { takeover: false } });
    const suggest = await takeOver(off, item());
    expect(suggest).toHaveBeenCalledOnce();
    expect(off.sent).toEqual([]);
  });

  test('when the app fails, the browser keeps the download and the URL is left alone for a while', async () => {
    let now = 1000;
    const chrome = fakeChrome({ answer: { ok: false, code: 'appNotRunning', message: '' } });
    const listener = createTakeover(chrome, { userAgent: 'UA', now: () => now });
    const first = await takeOver(chrome, item(), { listener });
    expect(first).toHaveBeenCalledOnce();
    expect(first.mock.calls[0]).toEqual([]);
    expect(chrome.downloads.cancel).not.toHaveBeenCalled();

    // The same URL again (the user retries in the browser): not sent to the app.
    await takeOver(chrome, item(), { listener });
    expect(chrome.sent).toHaveLength(1);

    now += FALLBACK_MS;
    await takeOver(chrome, item(), { listener });
    expect(chrome.sent).toHaveLength(2);
  });

  test('a host that does not answer in time falls back to the browser', async () => {
    const chrome = fakeChrome();
    chrome.runtime.sendNativeMessage = vi.fn(); // never calls back
    const suggest = await takeOver(chrome, item(), { timeoutMs: 20 });
    expect(suggest).toHaveBeenCalledOnce();
    expect(chrome.downloads.cancel).not.toHaveBeenCalled();
  });
});
