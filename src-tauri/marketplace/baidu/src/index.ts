// Baidu Netdisk share links (pan.baidu.com/s/1<surl>?pwd=<code>), through the user's own account.
//
// The web API hands out download links for shared files only in encrypted form, so the file is
// saved into the user's netdisk (FOLDER) and the desktop client's download API gives a plain
// link for it. Recorded from the live site (tests/cassettes/single-file.json, 2026-11-15; the
// logged-in page and the link reply are trimmed to the fields read here, without account data):
//   - pcs download servers serve only the client's User-Agent, need no cookie, and refuse
//     ranges over about 4 MiB for non-members (403); links expire after 8 hours;
//   - non-members get ~15 KiB/s per connection and are cut off above 4 connections;
//   - a link keeps working after the file is deleted from the netdisk.
// Saved files are left for the download and cleaned up on later calls: anything in FOLDER
// saved more than LINK_LIFETIME ago has no live link any more. A download that outlives its
// link is resolved again by the app, which saves the file again if it is gone.

import { defineModule, type Zoo } from 'zoo-module';

const FOLDER = '/zoo-downloader';
const LINK_LIFETIME = 8 * 3600;
const PAN = 'https://pan.baidu.com';
const WEB = 'channel=chunlei&web=1&app_id=250528&clienttype=0';
const BROWSER = 'Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/131.0.0.0 Safari/537.36';
const CLIENT = 'netdisk;7.42.0.5;PC;PC-Windows;10.0.22631;WindowsBaiduYunGuanJia';

interface Reply {
  errno: number;
  show_msg?: string;
  [field: string]: unknown;
}

interface Entry {
  path: string;
  server_filename: string;
  size: number;
  isdir: number | string;
  server_mtime: number;
  fs_id: number | string;
}

/** `/s/1<surl>` or `/share/init?surl=<surl>`; the API wants the surl without the leading 1. */
function shareOf(url: string, zoo: Zoo): { surl: string; pwd?: string } {
  const u = new URL(url);
  const fromPath = /^\/s\/1([\w-]+)/.exec(u.pathname)?.[1];
  const surl = fromPath ?? u.searchParams.get('surl') ?? undefined;
  if (!surl) throw zoo.fail('this is not a Baidu Netdisk share link', { hint: 'copy the whole link, like https://pan.baidu.com/s/1...?pwd=...' });
  return { surl, pwd: u.searchParams.get('pwd') ?? undefined };
}

/** The meaning of the errno values seen on these endpoints, with what the user can do. */
function failure(step: string, reply: Reply, zoo: Zoo): Error {
  const said = reply.show_msg ? ` (${reply.show_msg})` : '';
  switch (reply.errno) {
    case -6:
      return zoo.fail('Baidu did not accept the login cookies', { hint: 'copy BDUSS and STOKEN again from pan.baidu.com (Settings > Site modules > 百度网盘)' });
    case -9:
      return step === 'verify'
        ? zoo.fail('wrong extraction code (提取码)', { hint: 'check the ?pwd= part of the link' })
        : zoo.fail('the share needs an extraction code (提取码)', { hint: 'add it to the link: https://pan.baidu.com/s/1...?pwd=code' });
    case -62:
    case -63:
      return zoo.fail('Baidu wants a verification code for this share', { hint: 'open the link in a browser once, then try again' });
    case -10:
    case 12:
      return zoo.fail(`your netdisk has no room to save the file${said}`, { hint: 'free some space in your netdisk (the file is saved there before downloading)' });
    case 105:
    case 115:
    case 145:
      return zoo.fail(`the share is gone or blocked${said}`, { hint: 'ask the sharer for a new link' });
    default:
      return zoo.fail(`Baidu Netdisk ${step} failed: errno ${reply.errno}${said}`);
  }
}

export default defineModule({
  async resolve(url, zoo) {
    if (!zoo.credentials.has('BDUSS') || !zoo.credentials.has('STOKEN')) {
      throw zoo.fail('Baidu Netdisk needs your login cookies', {
        hint: 'in a browser logged in to pan.baidu.com, copy the BDUSS and STOKEN cookies into Settings > Site modules > 百度网盘',
      });
    }
    const { surl, pwd } = shareOf(url, zoo);
    const sharePage = `${PAN}/s/1${surl}`;
    const login = `BDUSS=${zoo.credentials.placeholder('BDUSS')}; STOKEN=${zoo.credentials.placeholder('STOKEN')}`;
    // The share's password cookie (BDCLND) goes with the login cookies once verified.
    let cookie = login;

    const web = async (path: string, init: { method?: string; form?: Record<string, string> } = {}): Promise<Reply> => {
      const response = await fetch(PAN + path, {
        method: init.method ?? (init.form ? 'POST' : 'GET'),
        headers: {
          'User-Agent': BROWSER,
          Cookie: cookie,
          Referer: sharePage,
          ...(init.form ? { 'Content-Type': 'application/x-www-form-urlencoded; charset=UTF-8' } : {}),
        },
        body: init.form ? new URLSearchParams(init.form) : undefined,
      });
      try {
        return (await response.json()) as Reply;
      } catch {
        throw zoo.fail(`Baidu Netdisk answered HTTP ${response.status} without JSON`, { hint: 'the site may be down or blocking this network; try again later' });
      }
    };

    // 1. Extraction code.
    if (pwd) {
      const verify = await web(`/share/verify?surl=${surl}&${WEB}`, { form: { pwd, vcode: '', vcode_str: '' } });
      if (verify.errno !== 0 || typeof verify.randsk !== 'string') throw failure('verify', verify, zoo);
      cookie = `${login}; BDCLND=${verify.randsk}`;
    }

    // 2. The shared file.
    const shared = await web(`/share/list?shorturl=${surl}&dir=&root=1&page=1&num=100&order=time&desc=1&showempty=0&${WEB}`);
    if (shared.errno !== 0) throw failure('share listing', shared, zoo);
    const files = (shared.list as Entry[] | undefined) ?? [];
    if (files.length !== 1 || Number(files[0].isdir) === 1) {
      throw zoo.fail(`this share holds ${files.length === 1 ? 'a folder' : `${files.length} items`}; only single-file shares are supported`, {
        hint: 'open the share in a browser and share or save the files one by one',
      });
    }
    const file = files[0];

    // 3. Logged in? The page also carries bdstoken, which writes need.
    const page = await (await fetch(sharePage, { headers: { 'User-Agent': BROWSER, Cookie: cookie } })).text();
    const bdstoken = /"bdstoken"\s*:\s*"([0-9a-f]{16,})"/.exec(page)?.[1];
    if (!/"loginstate"\s*:\s*1/.test(page) || !bdstoken) {
      throw failure('login', { errno: -6 }, zoo);
    }

    // 4. FOLDER: create it, clean up old saves, reuse a recent save of the same file.
    let mine = await web(`/api/list?dir=${encodeURIComponent(FOLDER)}&${WEB}`);
    if (mine.errno === -9) {
      const made = await web(`/api/create?a=commit&bdstoken=${bdstoken}&${WEB}`, { form: { path: FOLDER, isdir: '1', block_list: '[]' } });
      if (made.errno !== 0) throw failure('folder creation', made, zoo);
      mine = { errno: 0, list: [] };
    }
    if (mine.errno !== 0) throw failure('folder listing', mine, zoo);
    const now = Date.now() / 1000;
    const entries = (mine.list as Entry[] | undefined) ?? [];
    const expired = entries.filter((e) => Number(e.isdir) !== 1 && now - e.server_mtime > LINK_LIFETIME);
    if (expired.length) {
      const removed = await web(`/api/filemanager?opera=delete&async=0&onnest=fail&bdstoken=${bdstoken}&${WEB}`, {
        form: { filelist: JSON.stringify(expired.map((e) => e.path)) },
      });
      // Cleanup is a courtesy; the download does not depend on it.
      if (removed.errno !== 0) zoo.log.warn(`could not remove ${expired.length} old file(s) from ${FOLDER}: errno ${removed.errno}`);
    }
    let saved = entries.find((e) => !expired.includes(e) && e.server_filename === file.server_filename && Number(e.size) === Number(file.size));
    if (!saved) {
      const transfer = await web(
        `/share/transfer?shareid=${shared.share_id}&from=${shared.uk}&bdstoken=${bdstoken}&ondup=newcopy&async=1&${WEB}`,
        { form: { fsidlist: JSON.stringify([Number(file.fs_id)]), path: FOLDER } },
      );
      if (transfer.errno !== 0) throw failure('saving to your netdisk', transfer, zoo);
      const to = (transfer.extra as { list?: { to?: string }[] } | undefined)?.list?.[0]?.to;
      saved = { ...file, path: to ?? `${FOLDER}/${file.server_filename}` };
    }

    // 5. A plain link from the desktop client's API.
    const located = await fetch(`https://d.pcs.baidu.com/rest/2.0/pcs/file?method=locatedownload&app_id=250528&ver=4.0&path=${encodeURIComponent(saved.path)}`, {
      headers: { 'User-Agent': CLIENT, Cookie: login },
    });
    const urls = ((await located.json()) as { urls?: { url?: string }[]; error_code?: number; error_msg?: string });
    const link = urls.urls?.find((u) => u.url)?.url;
    if (!link) throw zoo.fail(`Baidu Netdisk gave no download link (${urls.error_code ?? located.status} ${urls.error_msg ?? ''})`.trim(), { hint: 'try again later' });

    return {
      url: link,
      filename: file.server_filename,
      size: Number(file.size),
      headers: { 'User-Agent': CLIENT },
      maxConnections: 4,
      maxSegmentSize: 4 * 1024 * 1024,
    };
  },
});
