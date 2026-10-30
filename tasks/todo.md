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

- Phase 0 熱修：檔名清理 / 本機 HTTP CORS / 關閉 http_bridge / enum 大小寫 / reqwest 30s 總逾時
- Phase 1 刪除假表面（候選 6）
- Phase 2 core crate：DownloadManager + PartFile、Downloads::add、Settings（候選 1、4、7）
- Phase 3 產生式契約 + 前端 DownloadSync（候選 2、5）
- Phase 4 網站模組 host + 擴充重寫（候選 3、8）

## 已知待辦（合併時發現）

- `@mui/icons-material` v7 與 `@mui/material` v6 版本不一致
- `package.json` 缺 `packageManager`
- 正式建置 chunk 722 kB
- `pub mod modules` 是讓 examples 能編譯的最小修正，需重新評估公開面
