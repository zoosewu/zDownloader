// Everything the extension does, with the `chrome` API passed in so tests can fake it.
// background.js only wires these to browser events. Protocol: docs/architecture/browser-extension.md.

export const HOST = 'com.zoo.downloader';
export const PROTOCOL = 1;
export const MENU_ID = 'zoo-download';
export const TAKEOVER_TIMEOUT_MS = 15_000;
// A URL the app refused is left to the browser for this long, so retrying it in the browser works.
export const FALLBACK_MS = 60_000;

/** Send one request to the app through zoo-native-host. Always resolves to `{ ok, ... }`. */
export function send(chrome, request) {
  return new Promise((resolve) => {
    chrome.runtime.sendNativeMessage(HOST, { v: PROTOCOL, ...request }, (answer) => {
      const error = chrome.runtime.lastError;
      if (error) {
        // "Specified native messaging host not found." / "...is forbidden.": the app never registered it.
        const notInstalled = /not found|forbidden/i.test(error.message ?? '');
        resolve({ ok: false, code: notInstalled ? 'notInstalled' : 'internal', message: error.message ?? '' });
      } else {
        resolve(answer ?? { ok: false, code: 'internal', message: 'no answer' });
      }
    });
  });
}

function withTimeout(promise, ms) {
  const timeout = new Promise((resolve) => setTimeout(() => resolve({ ok: false, code: 'timeout', message: '' }), ms));
  return Promise.race([promise, timeout]);
}

/** The `add` request for `url`, with the site's cookies unless it comes from an incognito window. */
export async function addRequest(chrome, { url, filename, referer, userAgent, incognito }) {
  const request = { type: 'add', url };
  if (filename) request.filename = filename;
  if (referer) request.referer = referer;
  if (userAgent) request.userAgent = userAgent;
  if (!incognito) {
    const cookies = await chrome.cookies.getAll({ url }).catch(() => []);
    if (cookies.length) request.cookie = cookies.map((c) => `${c.name}=${c.value}`).join('; ');
  }
  return request;
}

/** The popup's connection line for a ping answer. */
export function statusKey(answer) {
  if (answer.ok) return 'statusConnected';
  const keys = { notInstalled: 'statusNotInstalled', appNotRunning: 'statusNotRunning', unsupportedVersion: 'statusVersion' };
  return keys[answer.code] ?? 'statusError';
}

export function errorText(chrome, answer) {
  return chrome.i18n.getMessage(`error_${answer.code}`, [answer.message ?? '']) || answer.message || answer.code;
}

// ---- context menu ----

export function createMenu(chrome) {
  chrome.contextMenus.removeAll(() => {
    chrome.contextMenus.create({
      id: MENU_ID,
      title: chrome.i18n.getMessage('menuDownload'),
      contexts: ['link', 'image', 'video', 'audio'],
    });
  });
}

export async function onMenuClick(chrome, info, tab, userAgent) {
  const url = info.linkUrl || info.srcUrl;
  if (info.menuItemId !== MENU_ID || !url) return;
  const request = await addRequest(chrome, {
    url,
    referer: info.frameUrl || info.pageUrl,
    userAgent,
    incognito: Boolean(tab?.incognito),
  });
  const answer = await send(chrome, request);
  if (answer.ok) flashBadge(chrome);
  else notify(chrome, errorText(chrome, answer));
}

function flashBadge(chrome) {
  chrome.action.setBadgeBackgroundColor({ color: '#189172' }); // the app's brand color, hsl(165 72% 33%)
  chrome.action.setBadgeText({ text: '✓' });
  setTimeout(() => chrome.action.setBadgeText({ text: '' }), 2000);
}

function notify(chrome, message) {
  chrome.notifications.create({
    type: 'basic',
    iconUrl: 'icons/icon128.png',
    title: chrome.i18n.getMessage('extName'),
    message,
  });
}

// ---- download takeover ----

export async function takeoverEnabled(chrome) {
  const { takeover = true } = await chrome.storage.local.get('takeover');
  return takeover;
}

const basename = (path) => path.split(/[\\/]/).pop() ?? '';

/** Whether a new browser download should go to the app instead. */
export function shouldTakeOver(item, { enabled, fallbacks, now }) {
  const url = item.finalUrl || item.url;
  if (!enabled || item.incognito || item.byExtensionId) return false;
  // blob:, data:, file: and browser-internal URLs only exist inside the browser.
  if (!/^https?:/i.test(url)) return false;
  return !(fallbacks.get(url) > now);
}

/**
 * The `downloads.onDeterminingFilename` listener. The browser has the final file name by then
 * (Content-Disposition included). Downloads the app accepts are cancelled and erased from the
 * browser's list; anything else continues in the browser.
 */
export function createTakeover(chrome, { userAgent, now = Date.now, timeoutMs = TAKEOVER_TIMEOUT_MS } = {}) {
  const fallbacks = new Map();
  return (item, suggest) => {
    let suggested = false;
    const keep = () => {
      if (suggested) return;
      suggested = true;
      try {
        suggest();
      } catch {
        // The download is gone (cancelled); nothing to suggest for.
      }
    };
    (async () => {
      const enabled = await takeoverEnabled(chrome);
      if (!shouldTakeOver(item, { enabled, fallbacks, now: now() })) return keep();
      const url = item.finalUrl || item.url;
      const request = await addRequest(chrome, { url, filename: basename(item.filename), referer: item.referrer, userAgent });
      const answer = await withTimeout(send(chrome, request), timeoutMs);
      if (!answer.ok) {
        fallbacks.set(url, now() + FALLBACK_MS);
        return keep();
      }
      await chrome.downloads.cancel(item.id);
      await chrome.downloads.erase({ id: item.id });
      flashBadge(chrome);
      keep();
    })().catch(keep);
    return true;
  };
}
