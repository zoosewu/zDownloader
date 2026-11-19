// Screenshots (1280x800) of the dev app with demo downloads put into the frontend store only;
// the backend and the user's data are untouched, and a reload brings the real list back.
// Writes docs/screenshots/<NN-name>.<lang>.png (the portfolio's format, docs/screenshots/captions.json).
// Needs the dev app with CDP on port 9222 (WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS=--remote-debugging-port=9222).
//   node scripts/screenshots.mjs <lang: zh-TW|en> [outDir]
// Then: node scripts/optimize-png.mjs docs/screenshots/*.png
import { writeFileSync, mkdirSync } from 'node:fs';
import { join } from 'node:path';

const [lang, outDir = 'docs/screenshots'] = process.argv.slice(2);
if (lang !== 'zh-TW' && lang !== 'en') throw new Error('usage: node scripts/screenshots.mjs <zh-TW|en> [outDir]');
mkdirSync(outDir, { recursive: true });
const target = (await (await fetch('http://127.0.0.1:9222/json')).json()).find((t) => t.type === 'page' && t.url.includes('localhost:10020'));
const ws = new WebSocket(target.webSocketDebuggerUrl);
await new Promise((ok) => ws.addEventListener('open', ok));
let next = 1;
const waiting = new Map();
ws.addEventListener('message', (m) => { const msg = JSON.parse(m.data); waiting.get(msg.id)?.(msg); });
const send = (method, params = {}) => new Promise((ok) => { const id = next++; waiting.set(id, ok); ws.send(JSON.stringify({ id, method, params })); });
const run = async (expr) => {
  const r = await send('Runtime.evaluate', { expression: `(async () => { ${expr} })()`, awaitPromise: true, returnByValue: true });
  if (r.result?.exceptionDetails) throw new Error(r.result.exceptionDetails.exception?.description ?? 'script failed');
  return r.result?.result?.value;
};
const wait = (ms) => new Promise((r) => setTimeout(r, ms));
const shot = async (name) => {
  // Laid out at 1440x900 so every column fits, saved at the store's 1280x800.
  const r = await send('Page.captureScreenshot', { format: 'png', clip: { x: 0, y: 0, width: 1440, height: 900, scale: 1280 / 1440 } });
  const file = `${name}.${lang}.png`;
  writeFileSync(join(outDir, file), Buffer.from(r.result.data, 'base64'));
  console.log(file);
};

// Start from a fresh page (a reload from an earlier run may still be pending), and let the app
// load its settings (language, categories) first, or it overwrites the demo afterwards.
await run(`setTimeout(() => location.reload(), 0); return 'reloading';`);
await wait(2000);
for (let i = 0; i < 40; i++) {
  const ready = await run(`return document.querySelectorAll('tbody tr').length > 0 && performance.now() > 6000;`).catch(() => false);
  if (ready) break;
  await wait(500);
}
await wait(1500);
// Module URLs as the page loaded them (dev server versions change after a reload).
const modules = await run(`return performance.getEntriesByType('resource').map((e) => e.name).filter((n) => n.includes('deps/jotai.js') || n.includes('deps/i18next.js') || n.includes('/src/backend/index.ts'));`);
const jotaiUrl = new URL(modules.find((n) => n.includes('jotai.js'))).pathname + new URL(modules.find((n) => n.includes('jotai.js'))).search;
const backendUrl = (() => { const u = new URL(modules.find((n) => n.includes('/src/backend/index.ts'))); return u.pathname + u.search; })();
const i18nUrl = (() => { const u = new URL(modules.find((n) => n.includes('deps/i18next.js'))); return u.pathname + u.search; })();
await send('Emulation.setDeviceMetricsOverride', { width: 1440, height: 900, deviceScaleFactor: 1, mobile: false });
await wait(500);

const names = lang === 'en'
  ? { video: 'Video', software: 'Software', archive: 'Archives' }
  : { video: '影片', software: '軟體', archive: '壓縮檔' };

await run(`
  const { getDefaultStore } = await import('${jotaiUrl}');
  const ds = await import('/src/stores/downloadStore.ts');
  const ss = await import('/src/stores/settingsStore.ts');
  const vs = await import('/src/stores/viewStore.ts');
  const i18n = (await import('${i18nUrl}')).default;
  const store = getDefaultStore();
  // The app re-reads its settings when the language changes: answer that read (in this page
  // only) with the demo language and categories. The saved settings are not touched.
  const demoCategories = [
    { id: 'video', name: ${JSON.stringify(names.video)}, matching: 'any', rules: [] },
    { id: 'software', name: ${JSON.stringify(names.software)}, matching: 'any', rules: [] },
    { id: 'archive', name: ${JSON.stringify(names.archive)}, matching: 'any', rules: [] },
  ];
  // (window.__TAURI_INTERNALS__.invoke is read-only; the app reads settings through backend.settings.get.)
  const { backend } = await import('${backendUrl}');
  const get = backend.settings.get;
  backend.settings.get = async () => ({ ...(await get()), language: ${JSON.stringify(lang)}, categories: demoCategories, defaultDownloadDir: 'D:\\\\Downloads' });
  await i18n.changeLanguage(${JSON.stringify(lang)});
  await new Promise((r) => setTimeout(r, 800));
  const now = Date.now();
  const ago = (min) => new Date(now - min * 60000).toISOString();
  const MB = 1024 * 1024;
  const demo = [
    ['ubuntu-24.04.1-desktop-amd64.iso', 'https://releases.example.org/24.04/ubuntu-24.04.1-desktop-amd64.iso', 'running', 5.8 * 1024 * MB, 0.63, 12.4 * MB, 18, ['software']],
    ['Blender-4.2.3-windows-x64.msi', 'https://download.example.org/blender/Blender-4.2.3-windows-x64.msi', 'running', 341 * MB, 0.28, 6.1 * MB, 6, ['software']],
    ['lecture-week-07.mp4', 'https://media.example.edu/courses/cs101/lecture-week-07.mp4', 'queued', 812 * MB, 0, 0, 3, ['video']],
    ['project-assets.zip', 'https://files.example.com/share/project-assets.zip', 'paused', 1.2 * 1024 * MB, 0.45, 0, 42, ['archive']],
    ['dataset-2024.tar.gz', 'https://data.example.org/open/dataset-2024.tar.gz', 'completed', 2.1 * 1024 * MB, 1, 0, 95, ['archive']],
    ['trip-photos.zip', 'https://files.example.com/share/trip-photos.zip', 'completed', 640 * MB, 1, 0, 180, ['archive']],
    ['conference-keynote.mkv', 'https://media.example.org/2024/conference-keynote.mkv', 'completed', 1.6 * 1024 * MB, 1, 0, 320, ['video']],
    ['report-final.pdf', 'https://docs.example.com/reports/report-final.pdf', 'completed', 4.2 * MB, 1, 0, 1500, []],
  ];
  const tasks = {};
  demo.forEach(([name, url, state, total, frac, speed, minutes, tags], i) => {
    const id = '00000000-0000-4000-8000-00000000000' + i;
    const done = state === 'completed';
    tasks[id] = {
      id, url, name, customName: false, dir: 'D:\\\\Downloads', headers: [], finalPath: done ? 'D:\\\\Downloads\\\\' + name : null,
      state, phase: null, error: null, downloaded: Math.round(total * frac), total: Math.round(total), speed: Math.round(speed),
      createdAt: ago(minutes), startedAt: state === 'queued' ? null : ago(minutes - 1), finishedAt: done ? ago(minutes - 5) : null,
      scheduledAt: null, retryCount: 0, nextRetryAt: null, rev: 1000000 + i, order: 100 - i, tags, customTags: false, module: null,
    };
  });
  store.set(ds.tasksByIdAtom, tasks);
  const settings = store.get(ss.settingsAtom);
  store.set(ss.settingsAtom, { ...settings, defaultDownloadDir: 'D:\\\\Downloads', categories: [
    { id: 'video', name: ${JSON.stringify(names.video)}, matching: 'any', rules: [] },
    { id: 'software', name: ${JSON.stringify(names.software)}, matching: 'any', rules: [] },
    { id: 'archive', name: ${JSON.stringify(names.archive)}, matching: 'any', rules: [] },
  ] });
  store.set(vs.viewAtom, 'all');
  store.set(ds.selectionAtom, { ids: [], anchor: null });
  return 'demo ready';
`);
await wait(1200);
await shot('01-downloads');

// One selected: the detail panel.
await run(`
  const { getDefaultStore } = await import('${jotaiUrl}');
  const ds = await import('/src/stores/downloadStore.ts');
  getDefaultStore().set(ds.selectionAtom, { ids: ['00000000-0000-4000-8000-000000000000'], anchor: '00000000-0000-4000-8000-000000000000' });
`);
await wait(1000);
await shot('02-details');

// The add dialog with links (demo URLs; nothing is added).
await run(`
  const { getDefaultStore } = await import('${jotaiUrl}');
  const ds = await import('/src/stores/downloadStore.ts');
  getDefaultStore().set(ds.selectionAtom, { ids: [], anchor: null });
  await window.__TAURI_INTERNALS__.invoke('add_window_open', { items: [
    { url: 'https://releases.example.org/tools/editor-setup-3.2.exe', filename: null, headers: [] },
    { url: 'https://media.example.org/talks/opening-session.mp4', filename: null, headers: [] },
    { url: 'https://files.example.com/share/design-kit.zip', filename: null, headers: [] },
  ] });
`);
await wait(2500);
// The dialog fills its input from the clipboard: don't show what the user copied.
await run(`
  const input = document.querySelector('[role=dialog] input');
  Object.getOwnPropertyDescriptor(HTMLInputElement.prototype, 'value').set.call(input, '');
  input.dispatchEvent(new Event('input', { bubbles: true }));
  input.blur();
`);
await wait(500);
await shot('03-add');
await run(`[...document.querySelectorAll('[role=dialog] button')].find((b) => /取消|Cancel/.test(b.textContent))?.click();`);
await wait(500);

// Site modules: browsing the official marketplace (real, not personal, data).
await run(`
  const { getDefaultStore } = await import('${jotaiUrl}');
  const nav = await import('/src/stores/navigationStore.ts');
  getDefaultStore().set(nav.currentPageAtom, 'settings');
`);
await wait(800);
await run(`[...document.querySelectorAll('[role=tab]')].find((t) => /網站模組|Site modules/.test(t.textContent))?.click();`);
await wait(800);
await run(`[...document.querySelectorAll('[role=tab]')].find((t) => /^(瀏覽|Browse)$/.test(t.textContent.trim()))?.click();`);
await wait(2000);
await shot('04-modules');

await send('Emulation.clearDeviceMetricsOverride');
// Back to the real list, settings and language.
await run(`location.reload();`).catch(() => {});
ws.close();
