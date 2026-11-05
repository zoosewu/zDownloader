# Phase 4 設計規格：網站模組系統重建

- 狀態：已確認（2026-10-31），實作中：4a～4d、4.5 完成
- 依據：[ADR-0001 marketplace](../adr/0001-site-module-marketplace.md)、[ADR-0002 執行環境](../adr/0002-site-module-runtime.md)
- 修正：review B 段全部、SEC-3..6、D-4、D-5、C-11（偵測競態）
- 術語見 [GLOSSARY.md](../../GLOSSARY.md)

## 目標

社群（多半透過 vibe coding）寫的網站模組，經由 marketplace 安裝後：

1. **使用者**：貼上分享頁網址就能下載；模組再糟也只會讓自己失敗，不會拖垮 app、偷帳密或亂連網路。
2. **模組作者（多半是 AI）**：用熟悉的 Web API 寫一個 `resolve` 函式，`zoo-module test` 一跑就知道對不對，錯誤訊息告訴你怎麼修。

## 整體架構

```
┌──────────── Zoo Downloader（主程式，不含 V8）────────────┐
│ zoo-core DownloadManager                                  │
│   └─ Resolver（seam）── Direct：直接下載                   │
│                      └─ ModuleResolver ──┐                │
│ shell：zoo-module-client 的 Supervisor（啟動/重啟 host）   │
│        Marketplace（來源、目錄、安裝、權限同意）           │
└──────────────────────────────────────────┼────────────────┘
                     stdio JSON lines（zoo-module-client::protocol）
┌──────────────────── zoo-module（獨立執行檔）─────────────────┐
│ serve 模式：給 app 用    test/check/new 模式：給模組作者用     │
│ ├─ runtime：每次 resolve 一個 isolate，heap 上限、執行時限    │
│ ├─ Web API：fetch / URL / Headers / TextEncoder / crypto /     │
│ │           DOMParser / console                                │
│ ├─ zoo API：credentials.placeholder / captcha / wait / log     │
│ └─ Policy：網域白名單 → SSRF（含每次 redirect）→ 限速 →       │
│            帳密替換 → 稽核；test 模式下另有錄製/重播           │
└───────────────────────────────────────────────────────────────┘
```

### 新增的 crate

| crate | 內容 | 依賴 |
|---|---|---|
| `zoo-module-client` | IPC 協定型別（`Request`/`Reply`/`Resolved`/`ModuleError`/`Limits`）與 `Supervisor`；app 需要的全部 | serde、tokio（無 V8） |
| `zoo-module` (bin) | host 與開發 CLI 同一個執行檔 | deno_core、client |
| `zoo-core`（擴充） | `Resolver` seam、任務 phase | client 的型別 |

主程式（shell）不依賴 `zoo-module`，只依賴 client；V8 只在 `zoo-module` 編譯。Supervisor 放在 client 而非 shell，才能在 `zoo-module` 的整合測試中對真實 host 行程測試。

## 模組格式（給作者看的樣子）

```
my-site/
  manifest.json
  src/index.ts        ← 作者寫的
  dist/index.js       ← 打包後（ESM 單檔），marketplace 發佈這個
  tests/cases.json    ← zoo-module test 的案例
  tests/cassettes/    ← 錄下來的 HTTP 回應
  AGENTS.md           ← 給 AI 的開發說明（範本附帶）
```

```ts
// src/index.ts
import { defineModule } from 'zoo-module';

export default defineModule({
  async resolve(url, zoo) {
    const page = await fetch(url).then((r) => r.text());
    const doc = new DOMParser().parseFromString(page, 'text/html');
    const link = doc.querySelector('a.download')?.getAttribute('href');
    if (!link) throw zoo.fail('找不到下載連結', { hint: '頁面結構可能改了' });
    return { url: new URL(link, url).href, filename: doc.title };
  },
});
```

- **網址比對是宣告式的**：只看 manifest 的 `urlPatterns`，不執行 JS。移除舊的 `canHandle` 與 class 形式，AI 要寫的東西越少越不容易錯。
- **回傳值**：`{ url, filename?, headers?, size? }`。
- **zoo API（平台專屬，刻意保持很小）**：
  - `zoo.credentials.has(key)`、`zoo.credentials.placeholder(key)`：回傳不透明的佔位字串，放進 fetch 的 body 或 header；host 只在送往白名單網域時替換成真值。JS 永遠拿不到帳密。
  - `zoo.captcha.recaptcha({ siteKey, pageUrl })` / `zoo.captcha.image(imageUrl)`：等待使用者在 app 中完成。
  - `zoo.wait(seconds, reason)`：免費帳號倒數；UI 顯示倒數，不計入執行時限。
  - `zoo.log.*`、`zoo.fail(message, { hint })`。
- manifest 新增 `apiVersion: 1`；舊格式（k2s 現行）不相容，由 4f 遷移。

### zoo API 的實作細節（4b）

- **帳密佔位字串**：`zoocred-<每次呼叫的隨機值>-<key>-`，只含 URL、表單、JSON 編碼都不會改變的字元；key 限 `[A-Za-z0-9_]`。
  - host 依位置換成真值：URL 中做 percent-encoding、`application/x-www-form-urlencoded` body 做表單編碼、JSON body 做 JSON 跳脫、header 與其他 body 原樣。
  - 回應的 body、header、URL 中出現的真值會被換回佔位字串（長度 ≥ 6 的值才換，避免誤傷一般內容）。
  - `resolve` 回傳的 `url`、`headers` 中的佔位字串也會換成真值，交給下載核心使用。
  - 尚不支援：在 JS 中對佔位字串做運算（例如 `btoa(user + ':' + pass)` 組 Basic auth）。需要時再加專用 API。
- **Session 延續**：`resolve` 沒有自己設定 `Cookie` header 時，host 會把這次呼叫的 cookie jar 中屬於下載網址的 cookie 加進 `Resolved.headers`。
- **等待與 captcha**：
  - host 送出 `Event`（`waiting` / `captcha`），app 以 `Answer` 回覆 captcha。
  - 時間預算在等待期間暫停：host 端由 `Budget` 處理，Supervisor 端則把期限往後延。
  - 上限：單次 `zoo.wait` 600 秒，captcha 等待使用者 900 秒（逾時視為拒絕）。
  - 圖片 captcha 由 host 透過 Policy 與該次呼叫的 cookie 抓取，再以 base64 交給 app。

## 安全與資源

| 項目 | 做法 |
|---|---|
| 行程隔離 | 模組只在 `zoo-module serve` 中執行；崩潰時 Supervisor 重啟並讓進行中的解析失敗（可重試） |
| 記憶體 | 每次 resolve 一個獨立 isolate，heap 上限預設 128 MB，`near_heap_limit_callback` 觸發即終止該次解析；呼叫之間不共享狀態、不洩漏。Windows 另以 Job Object 限制整個 host 2 GB，且 app 結束時 host 一併結束 |
| CPU | 每次 `resolve` 的執行時限（預設 30 秒，不含 `zoo.wait` 與等待 captcha）：同步迴圈由 watchdog 執行緒以 `terminate_execution` 中斷，等待 I/O 由 tokio timeout 中斷；host 本身卡住時 Supervisor 在時限加 2 秒後 kill 並重啟 |
| 網路 | 只有 `fetch`，且只能連 manifest `permissions.network.allowDomains`（含子網域）；擋私有/保留位址（含 IPv4-mapped、NAT64、CGNAT、link-local），在自訂 DNS resolver 中檢查以防 rebinding；redirect 由 host 逐跳處理並重新檢查；不經系統 proxy；回應上限 16 MB；每次 resolve 一個 cookie jar；預設瀏覽器 User-Agent |
| 帳密 | 只存在主程式的系統鑰匙圈；每次解析只把該模組宣告的 key 傳給 host；替換只發生在送往白名單網域的請求 |
| 其他能力 | 沒有檔案系統、子行程、環境變數、`eval` 以外的動態載入；不再用 JS 層刪全域物件的假 sandbox |
| 稽核 | 每次解析記錄請求的網域與 Policy 判定，保留最近 N 筆，可在模組詳情檢視 |

## 下載流程整合（zoo-core）

- 新增 `Resolver` seam：`Direct`（現行行為）與 `ModuleResolver`（透過 Supervisor 呼叫 host）兩個 adapter。
- 任務加入 `phase`：`resolving` → `waiting { until }` → `awaitingUser`（captcha）→ `transferring`，UI 顯示目前階段。
- **每次開始傳輸前都重新解析**（分享站的直連多半會過期）；傳輸遇到 401/403/410 時自動重新解析一次。
- `.part` 續傳的比對改以**原始網址**加上大小與 ETag 為準，不受解析後網址變動影響。
- `Resolved.headers`（例如 cookie、referer）帶進每個傳輸請求。
- 實作（4c）：
  - zoo-core 的 `resolve::Resolver` 有兩個 adapter：`Direct`，以及 `zoo-module-client` 的 `resolver::ModuleResolver`（`core` feature）。後者依 `urlPatterns` 選第一個符合的模組，沒有符合的就直接下載。
  - `urlPatterns` 的 `*` 不會跨過 host 與 path 的分界。
  - 只有模組產生的連結（`Source.expires`）遇到 401/403/410 才會重新解析，且只重試一次。
  - 模組建議的檔名只在使用者沒指定檔名、而且還沒下載任何位元組時採用。
  - 暫停或取消時，Supervisor 送出 `Cancel`，host 立刻中斷該次解析。
  - 模組錯誤以 `CoreError::Resolve { message, hint, retryable }` 呈現：
    - 可重試：Script、Timeout、Crashed；
    - 不可重試：Load、OutOfMemory。
- 打包：
  - `pnpm build:module-host` 以 release 模式建置 host，放到 `src-tauri/binaries/zoo-module-<triple>`。
  - `tauri build --config src-tauri/tauri.bundle.json` 加上 externalBin。這個設定不放在主設定，否則平常的 `cargo build` 會因為缺少這個檔案而失敗。
  - shell 在 app 執行檔旁尋找 `zoo-module(.exe)`。開發時用 `cargo build -p zoo-module` 產生在同一個 target 目錄。
  - captcha 的 UI 延到 4e，在那之前 captcha 一律視為使用者拒絕。
- 下載核心補強（4.5，UI 與 4e 一起做）：
  - `DownloadManager::update(id, TaskUpdate { url, headers })`：
    - 可以換網址和 header，兩個都可省略；
    - 只能用在暫停、失敗、排隊、排程中的任務；
    - 新網址也可以是分享頁。
  - 換了網址，下一次傳輸時 `PartFile::open` 會回報 `Previous::Moved`，進入比對：
    - 大小相同，且從已下載的部分平均抽 4 段（每段 64 KiB，含頭尾）向新網址取回的位元組都相同，才續傳，並把進度記錄改成新網址的；
    - 大小不同、大小不明、不支援分段，或位元組不同，任務失敗，錯誤是 `CoreError::SourceChanged`；
    - 這個錯誤不會自動重試，原本的進度保留，把網址改回舊的就能接著下載。
  - `restart(id)`：刪除進度並重新排隊。UI 詢問時，預設的「從頭下載」就是這個命令。
  - 同一個網址、伺服器換了檔案（ETag 等不同）時維持原本行為：直接從頭下載。
  - `Task.headers`：新增任務時指定，同名時以模組回傳的 header 為準；`Range`、`Host`、`Content-Length` 不允許指定。
  - `Config.user_agent`（設定 `userAgent`）只在請求沒有指定 User-Agent 時使用。
  - 任務開始時進度計數沿用既有進度。修正前，傳輸開始前就失敗（例如 probe 失敗）會把畫面上的進度歸零。

## 開發工具（`zoo-module` CLI）

| 指令 | 用途 |
|---|---|
| `zoo-module new <name>` | 從範本建立模組（含 TS、打包設定、AGENTS.md、測試案例） |
| `zoo-module check [dir]` | manifest schema、apiVersion、權限、打包結果的靜態檢查 |
| `zoo-module test [dir] [url]` | 執行 `resolve`，輸出每個請求與 Policy 判定、zoo 呼叫、結果或錯誤與修正提示；`--json` 給 AI 讀 |
| `zoo-module test --record` / 預設重播 | 首次實連並錄製到 `tests/cassettes/`，之後離線重播 |
| `zoo-module test --inspect` | 開 Inspector 埠，可從 VS Code 下中斷點（未做，優先序最低） |

- **SDK**（`crates/zoo-module/sdk/`，已完成）：`zoo-module.d.ts`、`globals.d.ts`、`manifest.schema.json`。
  - 型別是手寫的。執行期是否一致由 `tests/sdk.rs` 檢查：全域物件完全相同、宣告的成員都存在、schema 與 `Manifest::parse` 判定相同。
  - 範本以 `lib: ["ES2023"]`、`types: []` 檢查型別，用到不支援的 API 會被標錯。
  - `resolve` 回傳多餘欄位時，型別錯誤會指出欄位名稱與合法欄位。
- 不做自訂 LSP 與 MCP server（2026-11-04 決定）：編輯器靠 TypeScript 與 JSON Schema 提供補全與檢查。
- CLI 實作（4d）：
  - `runtime::run_call` 由兩個 seam 驅動：`hooks::Hooks`（倒數、captcha、log、取消）與 `net::Transport`（live / record / replay）。serve 與 CLI 共用同一個 runtime。
  - 錄製檔以 HTTP hop 為單位，redirect 的每一跳各一筆；帳密以 `{{credential:key}}` 儲存。
  - 重播時仍經過 Policy 檢查，倒數直接略過，captcha 使用錄製時的答案。
  - `check` 的檢查項目：manifest、bundle 能否載入及形狀、程式碼用到但未宣告的帳密 key、測試案例與錄製檔。
  - `.credentials.json` 存放測試用帳號；範本的 `.gitignore` 已排除它。
  - `test` 的預期值：`url` 與 `filename` 用 `*` 萬用字元比對，`error` 比對錯誤訊息是否包含指定文字。
- CLI 與 app 隨附的 host 是同一個執行檔；另外以 GitHub Release 單獨發佈給模組作者。

## Marketplace（ADR-0001）

- **來源**：GitHub `owner/repo`、git URL、catalog 的 HTTPS URL、本機資料夾（開發用，檔案變更自動重新載入）。
- **catalog**（`zoo-marketplace.json`）：`name`、`owner`、`modules[]`（`id`、`version`、`source`、`description`、`urlPatterns`、權限摘要、`sha256`）。
- **安裝**：下載 `dist/index.js` 與 manifest 到 `<app_data>/modules/<marketplace>/<id>/`，驗證 sha256，顯示權限摘要請使用者同意。
- **更新**：手動檢查並提供「一律檢查更新」選項；權限擴大時須重新同意。
- **識別**：`id@marketplace`；同一網址有多個模組符合時，以使用者設定的優先順序決定（預設先安裝者優先）。
- **信任等級**：`official`（app 內附）與 `community`；簽章留待之後。
- **UI**：設定頁的「模組」分頁改為：已安裝（啟用/停用/更新/移除/權限/稽核）、來源（新增/移除）、瀏覽（各來源的目錄）。

## 移除

`src-tauri/src/modules/` 全部（js_runtime、js_module、loader、registry、sandbox、credential_proxy 的 HTTP 部分、url_detector 的模組部分），`commands/module_management.rs`、`commands/captcha.rs` 改寫；`modules/@types`、`modules/.vscode`、`MODULES.md`、specs/004 的 MODULE_BUILD_SYSTEM 與 SMART_MODULE_PATH 由新文件取代。保留並移入新位置：manifest 驗證邏輯、keychain CRUD、`CaptchaManager` 的概念。

## 子階段

| 子階段 | 內容 | 驗證 |
|---|---|---|
| 4a ✅ | client crate（協定 + Supervisor + Job Object）；`zoo-module serve`：載入 ESM 單檔、呼叫 `resolve`、heap 上限、執行時限、隱藏 `Deno.core`、console 改走 stderr | `crates/zoo-module/tests/host.rs`：無窮迴圈、OOM、並行隔離、模組形狀錯誤、回傳值驗證、host 崩潰/卡住/不存在 |
| 4b ✅ | Policy + `fetch` + Web API + `DOMParser` + zoo API（帳密佔位、captcha、wait） | 以 fixture server 測白名單、SSRF、redirect、帳密只送往白名單、回應大小上限 |
| 4c ✅ | zoo-core `Resolver` seam、任務 phase、重新解析、續傳比對改用原始網址；shell 的 ModuleResolver；host 執行檔隨附（`externalBin`）與 dev 建置 | 端到端：範例模組 + fixture server，經 DownloadManager 下載成功；連結過期重新解析 |
| 4d ✅ | CLI：new / check / test、錄製重播、`--json`、產生 d.ts；模組範本與 AGENTS.md；`Resolved.maxConnections`；SDK 說明以 `headers` 覆寫 User-Agent（偽裝成其他下載軟體，例如百度網盤） | 用範本新建模組並通過 test；讓 AI 依 AGENTS.md 寫一個 fixture 網站的模組 |
| 4.5 ✅ | 下載核心補強：更新網址（比對大小並抽樣比對位元組；不符時詢問，預設從頭下載）、直接連結自訂 header 與預設 User-Agent | zoo-core 整合測試：換成鏡像後續傳、內容不同時拒絕續傳、header 送達 |
| 4e | Marketplace：來源、catalog、安裝/更新/移除、權限同意、本機來源熱重載；前端模組頁 | 整合測試：本機 catalog 安裝、權限擴大要求重新同意、sha256 不符拒絕安裝 |
| 4f | k2s 遷移到新 API，改由內附的官方 marketplace 提供；刪除舊模組系統 | k2s 以錄製檔通過 test；實際帳號測試需要你操作 |
| 4g | 實機驗證與文件 | 驗證清單 |

每個子階段各自 commit 並通過全部測試；4f 之前舊模組系統保留不動。

## 已確認的決定（2026-10-31，全部照建議）

| # | 問題 | 決定 | 理由 |
|---|---|---|---|
| 1 | IPC 傳輸 | stdio JSON lines | 跨平台、無需處理 pipe 名稱與權限；host 的 log 走 stderr |
| 2 | HTML 解析 | 在 host 內預載 `linkedom`（純 JS），提供 `DOMParser` | 行為接近瀏覽器、AI 熟悉；Rust 端解析再橋接 DOM 太複雜 |
| 3 | Web API 實作 | **改為**：`URL`、`URLSearchParams`、`structuredClone`、`atob`/`btoa` 用 core-js，`DOMParser` 用 linkedom，兩者以 esbuild 打包成 `crates/zoo-module/src/gen/vendor.js`（`pnpm build:module-host-js`，產物入庫）；`fetch`、`Headers`、`Response`、`FormData`、`AbortController`、`TextEncoder`/`TextDecoder`、計時器、`crypto` 自己寫在 `web.js`，底層是 host op | 相容 deno_core 0.372 的 deno_web 0.253 連帶 deno_permissions、deno_features，且每次升級 deno_core 都要整組換版；自己的層沒有版本耦合。代價：不支援串流 body、Blob、WebSocket |
| 4 | 模組形狀 | 宣告式 `urlPatterns` + 單一 `resolve(url, zoo)` | AI 要寫的東西最少；比對不需要執行 JS |
| 5 | 模組語言 | 範本用 TypeScript，打包成 ESM 單檔；host 只吃打包後的 JS | 型別讓 AI 少犯錯；host 不必內建 TS 編譯 |
| 6 | 帳密 | 佔位字串 + host 端替換 | JS 拿不到帳密值，仍能用標準 fetch |
| 7 | 簽章 | 先不做，只有 official / community 兩級 | 先把沙箱做紮實；簽章牽涉金鑰發放流程，等社群成形再設計 |
| 8 | 舊 k2s 模組 | 在 4f 改寫成新 API | 舊 API 從未接通，沒有相容的必要 |
