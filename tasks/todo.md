# 架構深度檢視（2026-10-30）

目標：以最終成果為導向，重新檢驗 specs 001–004 與實作。不限於 deepening：可提出局部重寫、刪除、或整體架構翻新，每個範圍給出「保留 / 深化 / 重寫 / 刪除」判斷。產出 HTML 報告後逐項討論。

## 檢視切分（平行、唯讀）

- [x] A 下載引擎：`src-tauri/src/{download,storage,proxy,models}`、`commands/download.rs` ↔ spec 001
- [x] B 模組系統後端：`src-tauri/src/modules`、`commands/{module_management,captcha}.rs` ↔ spec 004
- [x] C 前端：`src/`（stores / hooks / components / api / utils）↔ spec 002、003、004 UI
- [x] D 跨端接縫：`lib.rs`、Tauri command 契約、`http_bridge`、`native_messaging`、`chrome-extension`、`modules/` JS 與型別
- [x] 彙整：HTML 報告 `%TEMP%\architecture-review-20261030-171712.html`（8 候選 + 重建藍圖 + Top recommendation）
- [x] 檢視結果入庫：`docs/architecture/review-2026-10-30.{md,html}`、`GLOSSARY.md`、CLAUDE.md 指引
- [x] 新需求記錄：ADR-0001 網站模組 marketplace（方向已接受，細節待設計）
- [x] 與使用者逐項 grilling，決定處理順序（依 Phase 0–4 與擴充重寫逐步處理完畢；2026-11-15 檢視紀錄全面同步，剩餘項目見紀錄開頭）

## 建議階段（待確認）

- [x] Phase 0 熱修：檔名清理 / 本機 HTTP 來源檢查 / http_bridge 改綁 loopback / enum 大小寫（含 proxy 欄位）+ 雙邊契約測試 / reqwest 30s 總逾時
- [x] Phase 1 刪除假表面（候選 6）— 分支 `rebuild/phase-1`
  - 原則：只刪無呼叫者的程式碼；已註冊但無作用的 command 留給 Phase 2；仍在執行路徑上的（sandbox）保留；每批編譯 + 測試
  - [x] 1a Rust 網站模組 scaffolding（browser/auth/init/error_format/resource_limiter/network_monitor/audit/context_api/permission/interface 死的部分/registry Rust 半邊/安全測試框架）
  - [x] 1b Rust 下載/代理死碼（verification.rs、proxy validator/health_scorer、storage/tasks.rs、RetryManager）— engine 內部重複路徑移到 1c 由編譯器判定
  - [x] 1c 移除 `#![allow(dead_code)]`，清掉編譯器指出的剩餘死碼
  - [x] 1d Cargo 依賴（libloading、tracing-appender、headless_chrome）、examples/*.rs、modules/example-downloader；test-malicious 移到 tests/fixtures
  - [x] 1e 前端死碼（UrlInput、CredentialManager、ProgressBar、ScheduleDialog、CredentialRetryHandler+其測試、credentialError、api/scheduler、無效 atom、dev mock 層）
  - [x] 1f 文件：specs/004 狀態文件、CLAUDE.md 去重、重疊的模組文件
  - 不做：i18n 未用 key（動態 key 難以可靠判定）、http_bridge 與 docker（2026-11-14 決定刪除，已刪）、DownloadStatus 不可達變體（影響持久化資料，Phase 2 處理）
- [x] Phase 2 下載核心重建（已合併 main）— 分支 `rebuild/phase-2`，規格 `docs/architecture/phase-2-download-core.md`（已確認）
  - 範圍擴大：併入原 Phase 3（產生式契約、DownloadSync）+ 代理 + 自動重試/排程/限速/通知
  - [x] 2a workspace + zoo-core 骨架（naming、part_file、store、rate）
  - [x] 2b transfer + Direct + fixture server
  - [x] 2c manager actor（狀態機、並行、控制、自動重試、排程、持久化）
  - [x] 2d ProxyPool adapter（shell 改接留到 2e 與刪舊引擎一起做）
  - [x] 2e Tauri shell（ts-rs 匯出、CommandError、入口、通知、刪舊引擎）— 前端在 2f 改接前執行期不可用
  - [x] 2f 前端（backend adapter、DownloadSync、元件改接、列操作、排程）— 分段視覺化已補回（downloads_segments）
  - [x] 2g 實機驗證與文件 — 2026-11-14 以 CDP 與本機測試站跑完清單（只剩系統通知需人工目視），修正多連線 flush 競爭
- ~~Phase 3 產生式契約 + 前端 DownloadSync（候選 2、5）~~ 併入 Phase 2
- [ ] Phase 4 網站模組系統重建 — 分支 `rebuild/phase-4`，規格 `docs/architecture/phase-4-site-modules.md`（已確認），依 ADR-0001、ADR-0002
  - [x] 4a client（協定 + Supervisor）+ host 骨架（資源上限、崩潰隔離）— shell 接線移到 4c
  - [x] 4b Policy + fetch + Web API + DOMParser + zoo API（帳密佔位、wait、captcha 事件）
  - [x] 4c zoo-core Resolver seam、任務 phase、重新解析、取消、打包設定（captcha UI 移到 4e）
  - [x] 4d 開發 CLI（new/check/test、錄製重播、--json）、AGENTS.md、SDK 型別、manifest schema、範本（--inspect 未做）
  - [x] 4d+ 模組端補充：`Resolved.maxConnections`（下載核心依此限制連線數）、SDK 說明以 `headers` 覆寫 User-Agent 並附測試
  - [x] 4.5 下載核心補強（2026-11-05 決定，排在 4d 之後、4e 之前；UI 與 4e 一起）
    - [x] 更新網址：`update` / `restart` 命令、大小與抽樣比對、`SourceChanged`（UI 詢問、預設從頭下載：4e）
    - [x] 直接連結自訂 header：`Task.headers`、設定 `userAgent`（新增任務的 header 欄位、設定頁常用預設：4e）
    - [ ] 不做：TLS 指紋偽裝（代價過高，有需要再評估）
  - [x] 4e Marketplace 與前端模組頁（規格 2026-11-05 確認：四種來源都支援、圖片與 reCAPTCHA 都做、分三段每段回報）
    - [x] 4e1 `zoo-marketplace` crate：來源、catalog、plan/install、sha256、權限同意、順序、帳密 seam
    - [x] 4e2 shell：command 與事件、captcha 互動、本機來源熱重載、內附官方來源、啟動時檢查更新
    - [x] 4e3 前端：模組頁、權限同意、帳密表單、captcha 對話框；4.5 的 UI（換網址、SourceChanged 詢問、header、User-Agent 預設）、錯誤訊息在地化
  - [x] 4f k2s 遷移、刪除舊模組系統（規格 2026-11-13 確認）
    - [x] k2s 依官方 API v2 重寫，放進官方 marketplace（手寫 cassette，4g 實錄）
    - [x] `urls_inspect` 取代 `detect_url` 與 `modules_match`
    - [x] 刪除舊模組系統（Rust、`modules/`、deno_core 依賴、文件）
  - [ ] 4g 實機驗證與文件
    - [x] 免費流程以真實網站錄製（2026-11-15，使用者在終端機回答驗證碼）：錯誤碼 31 → 驗證碼（答錯再 31、換新圖）→ `free_download_key` + `time_wait: 30` → 連結在 `*.filestore.app`（支援 Range、附 Content-Disposition）。模組推測的流程完全正確，不需修改
    - [ ] 用真實 k2s 帳號錄製登入、登入 reCAPTCHA 頁、premium 連結（使用者暫緩）
- [x] 瀏覽器擴充重寫：規格 [browser-extension.md](../docs/architecture/browser-extension.md)（2026-11-15 確認）
  - [x] 1 協定與 app 端（`browser_bridge`、host manifest 註冊；刪 HTTP 入口與 warp）
    - [x] `zoo-native-host` lib：協定、socket 名稱、固定擴充 ID；錯誤碼 `invalidUrl` 改為 `invalid`（header、檔名無效也用它）
    - [x] pipe：只有本人與 SYSTEM 可連（`D:P(A;;GA;;;OW)(A;;GA;;;SY)`），interprocess 預設已帶 first-instance 與拒絕遠端
    - [x] 實機：以 pipe client 送 ping、帶 Cookie/Referer/UA 的 add（測試站確認 header、下載完成）、無 Cookie（403）、`../../evil.bat`（只留 basename）、CRLF 注入 Cookie（拒絕）、v2、亂碼；Chrome/Edge 登錄與 manifest 正確
    - 註：登錄目前指向 debug 版 host（尚未存在），第 2 階段補上
  - [x] 2 `zoo-native-host`（轉送、啟動 app、pipe 擁有者檢查、打包）
    - [x] 同步小程式（release 396 KB）：框架讀寫、origin 檢查、轉送前重新編碼成一行、app 沒開就啟動（脫離瀏覽器 job）並等 10 秒、pipe 擁有者必須是旁邊的 app
    - [x] `pnpm build:sidecars` 取代 `build:module-host`，`externalBin` 加入 host
    - [x] 實機（以 node 模擬瀏覽器啟動 host）：錯誤 origin 拒絕；app 未開時啟動它並回應（約 0.5 秒），host 結束後 app 繼續執行；之後約 10 ms
    - 修正：app 繼承 host 的 std handle（瀏覽器的 pipe）會讓呼叫端一直等，啟動前改為不可繼承
  - [x] 3 擴充（右鍵、Cookie、接管下載、popup、圖示）
    - [x] `browser-extension/`：MV3、`src/core.js` 注入 `chrome`、三語 `_locales`、app 圖示產生 16/32/48/128；刪除 `chrome-extension/`
    - [x] popup 的 ping 不啟動 app（host 對 `ping` 不 launch），只顯示「未開啟」
    - [x] 加 `notifications` 權限（右鍵失敗通知）
    - [x] vitest 24 項：manifest key 與 Rust `EXTENSION_ID` 一致、權限、語系完整、送出組成、無痕不讀 Cookie、選單只建立一次、每種不接管條件、成功取消並清除、失敗與逾時回退、回退後 60 秒內同網址不再送
  - [x] 4 實機驗證（Edge 獨立 profile 自動跑；Chrome 由使用者載入確認）與文件
    - [x] Edge 157 獨立 profile（`--disable-sync`、`--load-extension` 仍可用）以 CDP 驗證：popup 在 app 未開時顯示「未開啟」且不啟動 app；接管下載時 host 啟動 app，app 帶 Cookie/Referer/UA 下載完成、瀏覽器紀錄清除；右鍵處理函式（真 API）送出成功、badge ✓；關閉接管時留給瀏覽器；host 不在時回退給瀏覽器下載、錯誤碼 `notInstalled` 與中文訊息正確；通知 API 回 id、權限 granted
    - [x] README 安裝步驟、review 標註 D-1..D-3 與 SEC-1
    - [x] 使用者：在 Edge 開發人員模式載入 `browser-extension/` 並實際用右鍵選單一次（原生選單無法以 CDP 點選）；目視系統通知
    - Chrome：使用者沒有安裝，視為與 Edge 相同（同一套 Chromium 擴充與 native messaging 實作；app 兩邊的登錄都寫了；已知差異只有 Chrome 正式版不接受 `--load-extension`，只影響自動化測試）
    - 發現：同時裝了 NDM 等下載管理器擴充時，兩者都會接管下載（NDM 也送了請求）；這類衝突只能由使用者擇一啟用

## 規格補完與整理（2026-11-14 計畫）

- [x] FR-012 佇列順序（使用者選「拖曳排序」）：`Task::order`（f64）決定列表與佇列順序，manager 的 `tasks` 維持依 order 排序，`pump` 不變
  - [x] zoo-core `move_before(id, before)`：取前後鄰居中點，精度用完時全部重新編號；舊資料（order 皆 0）載入時依原順序編號
  - [x] `downloads_move` + `src/backend`；表格以 `@dnd-kit/sortable` 拖曳整列（移動 5px 才開始，點擊選取與按鈕不受影響）
  - [x] 實機：拖上、拖下、取消執行中的任務後依新順序開始
- [x] FR-061 磁碟空間：probe 後比較剩餘位元組與 `fs4::available_space`（目錄不存在時查最近的既有上層），不足即 `insufficientSpace { needed, available }`，不重試、不寫任何檔案；查不到空間時放行
  - [x] 測試用 fixture 宣稱 1 PiB 的檔案；實機錯誤分頁顯示「磁碟空間不足：需要 1024.00 TB，只剩 31.13 GB」
- [x] 整理
  - [x] `@mui/material` 6.5 → 7.3.5，與 icons 對齊（程式沒用到 v7 移除的 API：Grid、深層 import、`onBackdropClick` 等）；實機逐頁檢查畫面
  - [x] chunk：桌面 app 從本機載入，拆分沒有實益 → `chunkSizeWarningLimit: 1024` 並註明原因
  - [x] 刪除 161 個沒用到的 i18n key（三語各剩 192 個；`t()` 全部是字面字串或 template，掃描可靠）；實機逐頁掃描沒有顯示成原始 key 的文字
  - [x] `packageManager` 早已存在，移除過期項目

## 已知待辦（合併時發現）

- [x] 架構檢視紀錄全面同步（2026-11-15）：75 列標上現況與證據（測試名稱、檔案）；刪除測試早已不存在型別的 `api-serialization.test.ts`

- [x] 擴充送出的任務帶瀏覽器 User-Agent 時，Google（YouTube）重置 HTTP/2 串流（`http2 error: stream error … protocol error`），自動重試用完後失敗。下載核心改為只用 HTTP/1.1（分段本來就該各用一條連線）；以一次性 reqwest 程式確認 h2 失敗、h1 成功，實機同一請求下載完成（2026-11-15）
- [x] 網站模組的 `fetch`（zoo-module）也明確只用 HTTP/1.1（workspace feature 合併會讓開發版開啟 HTTP/2，打包版不會；改成兩邊一致）（2026-11-15）

- [x] 下載核心讀 `Content-Disposition`（2026-11-15）：`transfer::run` 拆成 `probe_source` + `fetch`，manager 在兩者之間改名（與模組建議檔名同一條路；優先序：使用者 > 模組 > 伺服器 > 網址；只在尚未下載任何資料時）。`filename*`（UTF-8／ISO-8859-1）優先，只把 `\"`、`\` 當跳脫以保留 Windows 路徑讓 sanitize 截斷。實機：`secret.bin` → `secret-3.bin`

- `D:/rust_env/target/zoo-downloader` 會長到上百 GB（2026-11-12 達 175 GB、D 槽滿），需要時清 `debug/incremental` 或 `cargo clean`
- [x] 代理匯入接受 `http://host:port`（其他 scheme 明確拒絕；pool 只支援 HTTP 代理）（2026-11-14）
- [x] 自動重試等待中的任務顯示「等待重試 m:ss」，模組倒數也顯示剩餘時間（2026-11-14，實機確認倒數、重新排入、用完重試後回到「失敗」）
