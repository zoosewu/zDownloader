// src-tauri/marketplace/baidu/src/index.ts
import { defineModule } from "zoo-module";
var FOLDER = "/zoo-downloader";
var LINK_LIFETIME = 8 * 3600;
var PAN = "https://pan.baidu.com";
var WEB = "channel=chunlei&web=1&app_id=250528&clienttype=0";
var BROWSER = "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/131.0.0.0 Safari/537.36";
var CLIENT = "netdisk;7.42.0.5;PC;PC-Windows;10.0.22631;WindowsBaiduYunGuanJia";
function shareOf(url, zoo) {
  const u = new URL(url);
  const fromPath = /^\/s\/1([\w-]+)/.exec(u.pathname)?.[1];
  const surl = fromPath ?? u.searchParams.get("surl") ?? void 0;
  if (!surl) throw zoo.fail("this is not a Baidu Netdisk share link", { hint: "copy the whole link, like https://pan.baidu.com/s/1...?pwd=..." });
  return { surl, pwd: u.searchParams.get("pwd") ?? void 0 };
}
function failure(step, reply, zoo) {
  const said = reply.show_msg ? ` (${reply.show_msg})` : "";
  switch (reply.errno) {
    case -6:
      return zoo.fail("Baidu did not accept the login cookies", { hint: "copy BDUSS and STOKEN again from pan.baidu.com (Settings > Site modules > \u767E\u5EA6\u7F51\u76D8)" });
    case -9:
      return step === "verify" ? zoo.fail("wrong extraction code (\u63D0\u53D6\u7801)", { hint: "check the ?pwd= part of the link" }) : zoo.fail("the share needs an extraction code (\u63D0\u53D6\u7801)", { hint: "add it to the link: https://pan.baidu.com/s/1...?pwd=code" });
    case -62:
    case -63:
      return zoo.fail("Baidu wants a verification code for this share", { hint: "open the link in a browser once, then try again" });
    case -10:
    case 12:
      return zoo.fail(`your netdisk has no room to save the file${said}`, { hint: "free some space in your netdisk (the file is saved there before downloading)" });
    case 105:
    case 115:
    case 145:
      return zoo.fail(`the share is gone or blocked${said}`, { hint: "ask the sharer for a new link" });
    default:
      return zoo.fail(`Baidu Netdisk ${step} failed: errno ${reply.errno}${said}`);
  }
}
var index_default = defineModule({
  async resolve(url, zoo) {
    if (!zoo.credentials.has("BDUSS") || !zoo.credentials.has("STOKEN")) {
      throw zoo.fail("Baidu Netdisk needs your login cookies", {
        hint: "in a browser logged in to pan.baidu.com, copy the BDUSS and STOKEN cookies into Settings > Site modules > \u767E\u5EA6\u7F51\u76D8"
      });
    }
    const { surl, pwd } = shareOf(url, zoo);
    const sharePage = `${PAN}/s/1${surl}`;
    const login = `BDUSS=${zoo.credentials.placeholder("BDUSS")}; STOKEN=${zoo.credentials.placeholder("STOKEN")}`;
    let cookie = login;
    const web = async (path, init = {}) => {
      const response = await fetch(PAN + path, {
        method: init.method ?? (init.form ? "POST" : "GET"),
        headers: {
          "User-Agent": BROWSER,
          Cookie: cookie,
          Referer: sharePage,
          ...init.form ? { "Content-Type": "application/x-www-form-urlencoded; charset=UTF-8" } : {}
        },
        body: init.form ? new URLSearchParams(init.form) : void 0
      });
      try {
        return await response.json();
      } catch {
        throw zoo.fail(`Baidu Netdisk answered HTTP ${response.status} without JSON`, { hint: "the site may be down or blocking this network; try again later" });
      }
    };
    if (pwd) {
      const verify = await web(`/share/verify?surl=${surl}&${WEB}`, { form: { pwd, vcode: "", vcode_str: "" } });
      if (verify.errno !== 0 || typeof verify.randsk !== "string") throw failure("verify", verify, zoo);
      cookie = `${login}; BDCLND=${verify.randsk}`;
    }
    const shared = await web(`/share/list?shorturl=${surl}&dir=&root=1&page=1&num=100&order=time&desc=1&showempty=0&${WEB}`);
    if (shared.errno !== 0) throw failure("share listing", shared, zoo);
    const files = shared.list ?? [];
    if (files.length !== 1 || Number(files[0].isdir) === 1) {
      throw zoo.fail(`this share holds ${files.length === 1 ? "a folder" : `${files.length} items`}; only single-file shares are supported`, {
        hint: "open the share in a browser and share or save the files one by one"
      });
    }
    const file = files[0];
    const page = await (await fetch(sharePage, { headers: { "User-Agent": BROWSER, Cookie: cookie } })).text();
    const bdstoken = /"bdstoken"\s*:\s*"([0-9a-f]{16,})"/.exec(page)?.[1];
    if (!/"loginstate"\s*:\s*1/.test(page) || !bdstoken) {
      throw failure("login", { errno: -6 }, zoo);
    }
    let mine = await web(`/api/list?dir=${encodeURIComponent(FOLDER)}&${WEB}`);
    if (mine.errno === -9) {
      const made = await web(`/api/create?a=commit&bdstoken=${bdstoken}&${WEB}`, { form: { path: FOLDER, isdir: "1", block_list: "[]" } });
      if (made.errno !== 0) throw failure("folder creation", made, zoo);
      mine = { errno: 0, list: [] };
    }
    if (mine.errno !== 0) throw failure("folder listing", mine, zoo);
    const now = Date.now() / 1e3;
    const entries = mine.list ?? [];
    const expired = entries.filter((e) => Number(e.isdir) !== 1 && now - e.server_mtime > LINK_LIFETIME);
    if (expired.length) {
      const removed = await web(`/api/filemanager?opera=delete&async=0&onnest=fail&bdstoken=${bdstoken}&${WEB}`, {
        form: { filelist: JSON.stringify(expired.map((e) => e.path)) }
      });
      if (removed.errno !== 0) zoo.log.warn(`could not remove ${expired.length} old file(s) from ${FOLDER}: errno ${removed.errno}`);
    }
    let saved = entries.find((e) => !expired.includes(e) && e.server_filename === file.server_filename && Number(e.size) === Number(file.size));
    if (!saved) {
      const transfer = await web(
        `/share/transfer?shareid=${shared.share_id}&from=${shared.uk}&bdstoken=${bdstoken}&ondup=newcopy&async=1&${WEB}`,
        { form: { fsidlist: JSON.stringify([Number(file.fs_id)]), path: FOLDER } }
      );
      if (transfer.errno !== 0) throw failure("saving to your netdisk", transfer, zoo);
      const to = transfer.extra?.list?.[0]?.to;
      saved = { ...file, path: to ?? `${FOLDER}/${file.server_filename}` };
    }
    const located = await fetch(`https://d.pcs.baidu.com/rest/2.0/pcs/file?method=locatedownload&app_id=250528&ver=4.0&path=${encodeURIComponent(saved.path)}`, {
      headers: { "User-Agent": CLIENT, Cookie: login }
    });
    const urls = await located.json();
    const link = urls.urls?.find((u) => u.url)?.url;
    if (!link) throw zoo.fail(`Baidu Netdisk gave no download link (${urls.error_code ?? located.status} ${urls.error_msg ?? ""})`.trim(), { hint: "try again later" });
    return {
      url: link,
      filename: file.server_filename,
      size: Number(file.size),
      headers: { "User-Agent": CLIENT },
      maxConnections: 4,
      maxSegmentSize: 4 * 1024 * 1024
    };
  }
});
export {
  index_default as default
};
