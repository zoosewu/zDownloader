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
- [ ] 與使用者逐項 grilling，決定處理順序

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
  - 不做：i18n 未用 key（動態 key 難以可靠判定）、http_bridge 與 docker（待決策）、DownloadStatus 不可達變體（影響持久化資料，Phase 2 處理）
- [x] Phase 2 下載核心重建（已合併 main）— 分支 `rebuild/phase-2`，規格 `docs/architecture/phase-2-download-core.md`（已確認）
  - 範圍擴大：併入原 Phase 3（產生式契約、DownloadSync）+ 代理 + 自動重試/排程/限速/通知
  - [x] 2a workspace + zoo-core 骨架（naming、part_file、store、rate）
  - [x] 2b transfer + Direct + fixture server
  - [x] 2c manager actor（狀態機、並行、控制、自動重試、排程、持久化）
  - [x] 2d ProxyPool adapter（shell 改接留到 2e 與刪舊引擎一起做）
  - [x] 2e Tauri shell（ts-rs 匯出、CommandError、入口、通知、刪舊引擎）— 前端在 2f 改接前執行期不可用
  - [x] 2f 前端（backend adapter、DownloadSync、元件改接、列操作、排程）— 分段視覺化已補回（downloads_segments）
  - [ ] 2g 實機驗證與文件 — 文件已更新、app 可啟動；手動清單見 phase-2-download-core.md
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
    - [ ] 用真實 k2s 帳號錄製 cassette（免費流程的 `free_download_key`、等待欄位、登入 reCAPTCHA 頁的 site key 都是依文件推測）
- [ ] Chrome 擴充重寫（native messaging）— 另排

## 已知待辦（合併時發現）

- `.env.development` 設 `VITE_DEV_MODE=true`：`pnpm tauri dev` 的視窗也會走 HTTP bridge（只支援 5 個 command），開發時大部分功能不能用。應改成只有瀏覽器開前端時才走 bridge（2026-11-12 發現，待決定）
- `D:/rust_env/target/zoo-downloader` 會長到上百 GB（2026-11-12 達 175 GB、D 槽滿），需要時清 `debug/incremental` 或 `cargo clean`

- `@mui/icons-material` v7 與 `@mui/material` v6 版本不一致
- `package.json` 缺 `packageManager`
- 正式建置 chunk 722 kB
- shell 的 `utils/clipboard.rs` 另外輪詢剪貼簿並送出 `clipboard-detected` 事件，前端沒有人聽（前端 `useClipboard` 自己輪詢），可刪（4f 發現）
