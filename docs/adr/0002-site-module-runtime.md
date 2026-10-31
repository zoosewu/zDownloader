# ADR-0002：網站模組執行環境 — V8 獨立行程、Web 標準 API、為 AI 開發者設計

- 狀態：方向已接受（2026-10-31），細節待 Phase 4 規格
- 相關：[ADR-0001 marketplace](0001-site-module-marketplace.md)、[架構檢視](../architecture/review-2026-10-30.md) B 段、SEC-3..6、D-4

## 背景

- 模組將開放給社群，透過 marketplace 安裝（ADR-0001）。
- **大部分模組會以 vibe coding 撰寫**：實際作者多半是 AI，使用者不一定能審查程式碼。
- 現況（`deno_core` 嵌入主程式）的問題：沒有記憶體上限、無法中斷無窮迴圈、JS 層的 sandbox 可被繞過、每次呼叫洩漏 isolate、沒有任何 op、V8 拖慢整個 app 的建置、模組崩潰會拖垮 app。這些多數是「沒做」，不是 V8 本身的限制。

## 決策

### 1. 執行環境：V8（deno_core），跑在獨立行程 `zoo-module-host`

- 主程式不連結 V8；host 以 Tauri `externalBin` 隨附，主程式透過 IPC 呼叫，崩潰時自動重啟。
- 每個模組一個 V8 context；heap 上限（`near_heap_limit_callback`）＋每次呼叫的執行時限（`terminate_execution`）；Windows 另以 Job Object 限制行程記憶體。
- 選 V8 而非 QuickJS 的理由：AI 產生的程式慣用最新語法與 npm 套件，V8 相容性最好；支援 Inspector 除錯與 source map。QuickJS 的優勢（建置快、內建資源限制）主要有利於 app 開發，而非模組開發與使用；獨立行程後主程式的建置成本問題也消失。

### 2. 安全：預設最嚴格，不依賴模組作者

- 所有對外 I/O 只有一條路：host 提供的函式 → `Policy`（manifest 網域白名單 → SSRF 檢查含 redirect → 限速 → 稽核）。
- 帳密只在 host 端、只對白名單網域替換；JS 永遠拿不到帳密值。
- 不提供檔案系統、行程、任意網路存取。
- 安裝與更新時顯示權限摘要；權限擴大須重新同意（ADR-0001）。

### 3. 模組 API：以 Web 標準為主

- 提供 `fetch`、`URL`、`URLSearchParams`、`Headers`、`TextEncoder`/`TextDecoder`、`crypto.subtle`（子集）、HTML 解析（`DOMParser` 相容的子集），行為與瀏覽器一致，底層經過 Policy。
- 平台專屬能力（captcha、帳密需求、等待倒數、回報下載資訊）集中在單一小型命名空間，數量越少越好。
- 違規或錯誤時回傳**可照著修的訊息**（指出 manifest 該加什麼、哪一行）。
- manifest 宣告 `apiVersion`；舊版 API 保持相容。
- d.ts、文件、範例由 host 函式的同一份定義產生（修正 D-4）。

### 4. 開發工具：為 AI agent 的「寫 → 跑 → 讀輸出 → 修」迴圈設計

- `zoo-module test <dir> <url>`：直接用 host 執行模組，輸出結構化結果（每個請求、Policy 判定、解析結果、錯誤與修正提示）。
- 錄製與重播：首次實連時錄下回應，之後離線重播，可反覆修改而不必打真實網站。
- `zoo-module check`：manifest schema、權限、apiVersion、打包結果的靜態檢查。
- 模組範本附 `AGENTS.md` / `CLAUDE.md`、產生的 d.ts、可執行範例與測試。
- Inspector 中斷點除錯：提供，但優先序最低。

## 不採用

- **QuickJS**：見決策 1。
- **嵌入主程式的 V8（現況修好）**：無法隔離崩潰；V8 持續拖慢 app 建置。
- **WebView 執行模組**：難以控管網路與帳密，綁定 UI 生命週期。
- **Node.js sidecar**：需隨附完整 runtime，沙箱更難做。
- **`deno_runtime` 的權限系統**：粒度是行程層級，無法做到每個模組各自的網域白名單與帳密替換，仍需自訂 op。

## 待 Phase 4 規格決定

- IPC 傳輸方式（stdio / named pipe）與訊息協定
- HTML 解析的實作（Rust 端解析後提供 DOM 子集，或在 JS 端打包解析器）
- 錄製檔格式與存放位置
- 現有 k2s 模組遷移到新 API 的方式
- Policy 的稽核紀錄要保留多少、是否給使用者檢視
