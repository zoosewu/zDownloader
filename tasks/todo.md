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
- Phase 2 core crate：DownloadManager + PartFile、Downloads::add、Settings（候選 1、4、7）
- Phase 3 產生式契約 + 前端 DownloadSync（候選 2、5）
- Phase 4 網站模組 host + 擴充重寫（候選 3、8）

## 已知待辦（合併時發現）

- `@mui/icons-material` v7 與 `@mui/material` v6 版本不一致
- `package.json` 缺 `packageManager`
- 正式建置 chunk 722 kB
- `pub mod modules` 是讓 examples 能編譯的最小修正，需重新評估公開面
