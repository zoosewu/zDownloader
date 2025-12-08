# Implementation Tasks: 下載網站串接模組系統

**Feature**: 004-site-module-integration
**Branch**: `004-site-module-integration`
**Last Updated**: 2025-12-04

## Overview

本任務清單按照 User Story 優先級組織，確保每個 story 都能獨立實作和測試。採用 MVP-first 策略，優先完成最小可行產品（MVP = User Story 2 的核心架構）。

**總任務數**: 169 個任務
**MVP 範圍**: Phase 1-3 (Setup + Foundational + US2 核心架構)
**並行機會**: 35+ 個標記為 [P] 的任務可並行執行
**開發者體驗**: Phase 8 包含完整的 TypeScript 型別定義和 VS Code 整合

---

## Phase 1: Setup (專案初始化)

### 目標
建立專案基礎設施，安裝依賴，配置開發環境。

### Tasks

- [X] T001 新增 Rust dependencies 到 src-tauri/Cargo.toml (deno_core, tauri-plugin-keyring, wildmatch, tracing)
- [X] T002 建立模組目錄結構 modules/ 和 modules/_template/
- [X] T003 建立 Rust 模組原始檔結構 src-tauri/src/modules/ (9 個新檔案)
- [X] T004 [P] 新增前端 dependencies 到 package.json (若需要新的套件)
- [X] T005 [P] 建立前端目錄結構 src/components/, src/api/, src/types/module.ts
- [X] T006 初始化審計日誌目錄 logs/audit/
- [X] T007 更新 src-tauri/src/modules/mod.rs 註冊新模組檔案
- [X] T008 [P] 建立 K2S 模組模板檔案 modules/k2s/manifest.json, modules/k2s/index.js, modules/k2s/README.md
- [X] T009 驗證編譯：執行 cargo check 確認 Rust 依賴正確安裝
- [X] T010 [P] 驗證編譯：執行 npm run check 確認前端依賴正確安裝

**完成標準**: 所有依賴安裝成功，目錄結構建立完成，編譯無錯誤。

---

## Phase 2: Foundational (基礎設施 - 阻塞性前置任務)

### 目標
實作所有 User Stories 共同依賴的基礎設施：manifest 解析、JavaScript runtime、沙箱環境、審計系統。

**依賴**: Phase 1 完成
**獨立測試標準**: 可以載入一個最小的 JavaScript 模組並在沙箱中安全執行。

### Tasks

#### 2.1 Manifest 系統 (FR-007, FR-008, FR-009)

- [X] T011 實作 ModuleManifest 資料結構 src-tauri/src/modules/manifest.rs
- [X] T012 實作 manifest.json 解析函數 (使用 serde_json)
- [X] T013 實作 manifest 驗證邏輯 (版本號 semver、欄位完整性、權限格式)
- [X] T014 實作 manifest 錯誤處理和友善錯誤訊息
- [X] T015 新增 manifest 解析的單元測試 (正常、錯誤、邊界案例)

#### 2.2 JavaScript Runtime (FR-003)

- [X] T016 實作 JsRuntime 初始化 src-tauri/src/modules/js_runtime.rs
- [X] T017 配置 deno_core RuntimeOptions (禁用內建擴展)
- [X] T018 實作 JavaScript 程式碼載入和執行
- [X] T019 實作 runtime 錯誤捕獲和轉換為 Rust Result
- [X] T020 新增 JS runtime 基本功能測試

#### 2.3 沙箱環境 (FR-027)

- [X] T021 實作 ModuleSandbox 結構 src-tauri/src/modules/sandbox.rs
- [X] T022 注入沙箱初始化腳本 (刪除 eval, Function, __proto__ 等危險 API)
- [X] T023 凍結 JavaScript 原型 (Object.prototype, Array.prototype)
- [X] T024 實作沙箱違規偵測和錯誤處理
- [X] T025 新增沙箱逃逸測試 (嘗試存取被禁止的 API)

#### 2.4 審計日誌系統 (FR-030)

- [X] T026 初始化 tracing subscriber src-tauri/src/modules/audit.rs
- [X] T027 配置 JSON 格式輸出和日誌輪轉 (每日輪轉，保留 30 天)
- [X] T028 實作 AuditLogEntry 資料結構
- [X] T029 實作審計事件記錄函數 (log_security_violation, log_network_request 等)
- [X] T030 新增審計日誌寫入和讀取測試

#### 2.5 權限系統 (FR-010, FR-011, FR-012)

- [X] T031 實作 ModulePermissions 資料結構 src-tauri/src/modules/permission.rs
- [X] T032 實作網域白名單匹配 (使用 wildmatch crate)
- [X] T033 實作權限檢查函數 (check_network_permission, check_credential_permission)
- [X] T034 實作權限違規事件記錄
- [X] T035 新增權限系統單元測試 (通配符匹配、權限拒絕)

**完成標準**: 可以載入 manifest、初始化沙箱、執行簡單的 JS 程式碼、記錄審計日誌、檢查權限。

---

## Phase 3: User Story 2 - 可擴展的模組架構 (Priority P1)

### 目標
實作核心模組系統架構，包含模組註冊、發現、載入和呼叫機制。這是 MVP 的核心。

**依賴**: Phase 2 完成
**獨立測試標準**: 可以註冊一個測試模組，系統能自動發現並呼叫它處理 URL。

### Tasks

#### 3.1 JavaScript 模組包裝器 (FR-001)

- [X] T036 [US2] 實作 SiteModule 資料結構 src-tauri/src/modules/js_module.rs
- [X] T037 [US2] 實作 JavaScript 模組類別實例化 (new MySiteModule())
- [X] T038 [US2] 實作 Rust-to-JS 函數綁定 (canHandle, extractDownloadInfo)
- [X] T039 [US2] 實作 JS-to-Rust 回傳值轉換 (DownloadInfo deserialization)
- [X] T040 [US2] 實作模組執行錯誤處理和清理

#### 3.2 模組註冊表 (FR-002, FR-004, FR-005)

- [X] T041 [US2] 擴展 ModuleRegistry 支援 JavaScript 模組 src-tauri/src/modules/registry.rs
- [X] T042 [US2] 實作 ModuleType enum (Rust vs JavaScript)
- [X] T043 [US2] 實作模組自動發現 (掃描 modules/ 目錄)
- [X] T044 [US2] 實作模組載入流程 (讀取 manifest → 驗證 → 建立沙箱 → 載入程式碼)
- [X] T045 [US2] 實作 find_module_for_url (URL 匹配邏輯)
- [X] T046 [US2] 實作模組衝突解決 (多個模組匹配同一 URL)
- [X] T047 [US2] 實作回退到通用下載模式
- [X] T048 [US2] 新增模組註冊表單元測試

#### 3.3 資源限制器 (FR-028)

- [X] T049 [P] [US2] 實作 ResourceLimits 資料結構 src-tauri/src/modules/resource_limiter.rs
- [X] T050 [P] [US2] 實作 CPU 時間限制 (tokio::time::timeout)
- [X] T051 [P] [US2] 實作記憶體使用監控 (JsRuntime heap statistics)
- [X] T052 [P] [US2] 實作資源超限處理 (強制終止模組)
- [X] T053 [P] [US2] 新增資源限制測試

#### 3.4 模組載入器整合

- [X] T054 [US2] 整合所有元件：Manifest → Sandbox → Runtime → Module
- [X] T055 [US2] 實作模組生命週期管理 (載入、執行、卸載)
- [X] T056 [US2] 實作模組狀態追蹤 (Pending → Loading → Active → Failed)
- [X] T057 [US2] 新增完整的模組載入流程測試

**完成標準**: 系統能自動掃描、載入和執行測試模組，正確處理 URL 匹配和權限檢查。

---

## Phase 4: User Story 1 - 支援 Keep2Share (K2S) 下載 (Priority P1)

### 目標
實作第一個真實的網站模組 (K2S)，驗證模組系統的實用性。

**依賴**: Phase 3 完成
**獨立測試標準**: 使用者貼上 K2S 連結，系統能完成完整下載流程。

### Tasks

#### 4.1 認證代理 API (FR-013, FR-014)

- [X] T058 [US1] 實作 CredentialProxy 結構 src-tauri/src/modules/credential_proxy.rs
- [X] T059 [US1] 實作認證資訊安全儲存 (tauri-plugin-keyring)
- [X] T060 [US1] 實作認證代理 HTTP 請求 API (inject 到 JS context)
- [X] T061 [US1] 實作認證資訊載入和驗證
- [X] T062 [US1] 新增認證代理隔離測試 (JS 無法直接存取密碼)

#### 4.2 網路監控 (FR-031, FR-032)

- [X] T063 [P] [US1] 實作 NetworkMonitor 結構 src-tauri/src/modules/network_monitor.rs
- [X] T064 [P] [US1] 實作請求頻率限制 (60 requests/min)
- [X] T065 [P] [US1] 實作回應大小限制 (10MB max)
- [X] T066 [P] [US1] 實作可疑行為偵測 (多網域請求)
- [X] T067 [P] [US1] 新增網路監控測試

#### 4.3 URL 驗證器 (FR-029, FR-032)

- [X] T068 [P] [US1] 實作 UrlValidator 結構 src-tauri/src/modules/url_validator.rs
- [X] T069 [P] [US1] 實作內網 IP 檢測 (使用 std::net::Ipv4Addr::is_private())
- [X] T070 [P] [US1] 實作 DNS 解析後驗證 (防止 DNS rebinding)
- [X] T071 [P] [US1] 實作協議檢查 (只允許 HTTP/HTTPS)
- [X] T072 [P] [US1] 新增 SSRF 防護測試

#### 4.4 Context API 實作

- [X] T073 [US1] 設計 JS context API 介面 (http, credentials, captcha, log)
- [X] T074 [US1] 實作 context.http.get/post 並整合認證代理
- [X] T075 [US1] 實作 context.credentials (返回認證資訊 proxy)
- [X] T076 [US1] 實作 context.log (橋接到 Rust tracing)
- [X] T077 [US1] 實作 context.captcha.request (Tauri event 機制)

#### 4.5 K2S 模組實作 (FR-017 to FR-022)

- [X] T078 [US1] 完善 K2S manifest.json (權限、網域白名單)
- [X] T079 [US1] 實作 K2S canHandle (URL 模式匹配)
- [X] T080 [US1] 實作 K2S 認證流程 (login API)
- [X] T081 [US1] 實作 K2S URL 解析邏輯
- [X] T082 [US1] 實作 K2S 驗證碼處理
- [X] T083 [US1] 實作 K2S 會員類型判斷 (付費/免費)
- [X] T084 [US1] 新增 K2S 模組整合測試

#### 4.6 下載整合

- [X] T085 [US1] 擴展 DownloadTask 資料結構 (新增 module_id 欄位)
- [X] T086 [US1] 修改 create_download command 整合模組系統
- [X] T087 [US1] 實作 detect_url_module 自動偵測
- [X] T088 [US1] 實作模組錯誤處理和回退到通用下載
- [X] T089 [US1] 新增端對端測試 (貼上 K2S 連結 → 完成下載)

**完成標準**: 使用者貼上 K2S 連結，系統自動識別、認證、解析並開始下載，過程透明且安全。

#### 4.7 K2S 模組優化 (基於研究分析)

**參考**: `specs/004-site-module-integration/k2s-implementation-analysis.md`

- [X] T165 [US1] [高優先級] API 端點驗證 - 測試並修正 K2S API 端點名稱
  - 驗證 `/v2/file/info` vs `/v2/getFilesInfo`
  - 驗證 `/v2/file/download` vs `/v2/getUrl`
  - 驗證 `/v2/account/info` vs `/v2/accountInfo`
  - 如果當前端點失效，切換到官方端點名稱

- [X] T166 [US1] [中優先級] CAPTCHA 處理優化 - 實作官方 ReCAPTCHA v2 流程
  - 實作 `/v2/requestReCaptcha` 端點請求
  - 實作 `/v2/reCaptcha` 端點提交
  - 處理 HTTP 406 + errorCode 33 錯誤碼
  - 保留當前 `captcha_required` 流程作為 fallback
  - 更新 Context API 的 `captcha.request()` 以支持 siteKey

- [x] T167 [US1] [中優先級] 錯誤處理和重試策略 - 添加智能重試機制
  - 在 CredentialProxy 中實作 `authenticated_request_with_retry()` 方法
  - 實作指數退避策略 (2s → 4s → 8s，上限 10s)
  - 定義可重試的錯誤類型 (429, 502, 503, 504, timeout, connection errors)
  - 配置最大重試次數和延遲參數（預設 3 次重試）
  - 添加重試邏輯測試（5 個測試全部通過）
  - Context API 自動整合重試能力

- [x] T168 [US1] [低優先級] 帳號類型偵測優化 - 處理過期的 premium 帳號
  - 檢查 `premium_expired` 標誌（已實作）
  - 處理過期 premium 帳號降級為 free（已實作）
  - 記錄 `premium_until` 日期到日誌（已實作）
  - 添加帳號狀態變更的警告訊息（已實作）
  - 存儲 `premiumExpired` 和 `premiumUntil` 屬性

- [x] T169 [US1] [低優先級] 平行下載優化 - 下載引擎層級改進
  - K2S 模組返回 `downloadHints` metadata（已實作）
  - 配置最佳區塊大小 (20MB)（已實作）
  - 配置並行連接數 (premium: 20, free: 5)（已實作）
  - 添加 `supportsRangeRequests` 標誌（已實作）
  - 添加 `requestDelay` 配置（premium: 0ms, free: 100ms）
  - 注意：DownloadEngine 的 range request 實作屬於下載引擎層級，不在本 feature 範圍

**完成標準**: K2S 模組使用官方 API 規格，具備完善的錯誤處理和重試機制，為 premium 用戶提供最佳下載性能。

---

## Phase 5: User Story 5 - 模組安全性驗證和測試 (Priority P1)

### 目標
建立全面的安全測試套件，驗證所有安全機制。

**依賴**: Phase 4 完成
**獨立測試標準**: 所有惡意測試模組的攻擊都被成功阻止，100% 成功率。

### Tasks

#### 5.1 惡意測試模組建立

- [X] T090 [P] [US5] 建立測試模組：unauthorized_domain.js (測試網域限制)
- [X] T091 [P] [US5] 建立測試模組：resource_exhaustion.js (測試資源限制)
- [X] T092 [P] [US5] 建立測試模組：ssrf_attempt.js (測試 SSRF 防護)
- [X] T093 [P] [US5] 建立測試模組：sandbox_escape.js (測試沙箱逃逸)
- [X] T094 [P] [US5] 建立測試模組：data_exfiltration.js (測試資料外洩偵測)
- [X] T095 [P] [US5] 建立測試模組：credential_access.js (測試認證隔離)
- [X] T096 [P] [US5] 建立測試模組：prototype_pollution.js (測試原型污染)
- [X] T097 [P] [US5] 建立測試模組：infinite_loop.js (測試執行逾時)

#### 5.2 安全測試套件

- [X] T098 [US5] 實作安全測試框架 src-tauri/src/modules/tests/security_tests.rs
- [X] T099 [US5] 實作測試：權限違規偵測 (SC-010: 100% 成功率)
- [X] T100 [US5] 實作測試：認證隔離 (SC-011: 100% 成功率)
- [X] T101 [US5] 實作測試：沙箱逃逸防護 (SC-012: 100% 成功率)
- [X] T102 [US5] 實作測試：資源限制執行 (SC-013: 100% 成功率)
- [X] T103 [US5] 實作測試：內網存取阻止 (SC-014: 100% 成功率)
- [X] T104 [US5] 實作測試：URL 驗證 (SC-015: 100% 成功率)
- [X] T105 [US5] 實作測試：審計日誌完整性 (SC-016: 100% 成功率)
- [X] T106 [US5] 實作測試：可疑行為偵測 (SC-017: <5% 誤報率)
- [X] T107 [US5] 執行完整安全測試套件並生成報告

**完成標準**: 所有安全測試通過，審計日誌記錄完整，無安全漏洞。

---

## Phase 6: User Story 3 - 管理網站模組認證資訊 (Priority P2)

### 目標
實作使用者友善的認證管理介面。

**依賴**: Phase 4 完成 (認證代理 API)
**獨立測試標準**: 使用者能在 UI 中儲存、查看狀態和刪除認證資訊。

### Tasks

#### 6.1 認證管理 Tauri Commands (FR-015)

- [x] T108 [P] [US3] 實作 save_module_credentials command src-tauri/src/commands/module_management.rs
- [x] T109 [P] [US3] 實作 get_module_credentials_status command
- [x] T110 [P] [US3] 實作 delete_module_credentials command
- [x] T111 [P] [US3] 新增認證管理 commands 錯誤處理（完整驗證邏輯，3 個測試通過）

#### 6.2 前端認證管理組件

- [x] T112 [US3] 實作 CredentialManager 組件 src/components/CredentialManager.tsx
- [x] T113 [US3] 實作認證輸入表單 (username, password, apiKey 等動態欄位)
- [x] T114 [US3] 實作認證狀態顯示 (已設定/缺失的 keys)
- [x] T115 [US3] 實作刪除認證確認對話框
- [x] T116 [US3] 整合 react-i18next 多語言支援

#### 6.3 認證提示流程

- [ ] T117 [US3] 實作認證缺失偵測 (下載前檢查)
- [ ] T118 [US3] 實作認證輸入提示對話框
- [ ] T119 [US3] 實作認證失效處理和重新輸入提示
- [ ] T120 [US3] 新增認證流程整合測試

**完成標準**: 使用者能透過 UI 管理認證資訊，系統自動提示缺失的認證。

---

## Phase 7: User Story 4 - 網站模組管理介面 (Priority P2)

### 目標
實作模組管理 UI，讓使用者查看和管理已安裝的模組。

**依賴**: Phase 3 完成 (模組註冊表)
**獨立測試標準**: 使用者能在設定頁面查看所有模組資訊和權限。

### Tasks

#### 7.1 模組管理 Tauri Commands (FR-023, FR-024, FR-025)

- [x] T121 [P] [US4] 實作 get_all_modules command src-tauri/src/commands/module_management.rs
- [x] T122 [P] [US4] 實作 get_module_detail command
- [x] T123 [P] [US4] 實作 reload_module command
- [x] T124 [P] [US4] 實作 unload_module command
- [x] T125 [P] [US4] 實作 scan_modules command
- [x] T126 [P] [US4] 實作 get_module_audit_logs command

#### 7.2 前端模組管理組件

- [x] T127 [US4] 實作 ModuleList 組件 src/components/ModuleList.tsx
- [x] T128 [US4] 實作 ModuleCard 組件 src/components/ModuleCard.tsx (顯示基本資訊)
- [x] T129 [US4] 實作 ModulePermissions 組件 src/components/ModulePermissions.tsx (權限和網域列表 - 整合至 ModuleCard)
- [x] T130 [US4] 實作模組詳細資訊展開面板 (使用 Collapse 實作於 ModuleCard)
- [x] T131 [US4] 實作模組狀態指示器 (Active/Failed/Suspended)
- [x] T132 [US4] 實作模組操作按鈕 (重新載入、卸載)

#### 7.3 設定頁面整合

- [x] T133 [US4] 擴展 Settings 頁面 src/pages/Settings.tsx (新增模組管理區域)
- [x] T134 [US4] 實作模組管理區域布局和導航
- [x] T135 [US4] 整合 ModuleList 和 CredentialManager 組件
- [x] T136 [US4] 新增多語言翻譯 (zh-TW, en, ja)
- [ ] T137 [US4] 新增模組管理 UI 測試

**完成標準**: 使用者能在設定頁面查看所有模組、權限、狀態，並進行基本管理操作。

---

## Phase 8: Polish & Cross-Cutting Concerns

### 目標
完善系統，處理跨功能關注點，優化效能和使用者體驗。

**依賴**: Phases 3-7 完成

### Tasks

#### 8.1 錯誤處理優化 (FR-016)

- [ ] T138 [P] 統一錯誤訊息格式 (包含模組名稱)
- [ ] T139 [P] 實作友善錯誤提示 (前端顯示)
- [ ] T140 [P] 實作錯誤恢復機制 (重試、回退)

#### 8.2 效能優化

- [ ] T141 [P] 優化模組載入時間 (lazy loading)
- [ ] T142 [P] 優化記憶體使用 (及時釋放 unused runtimes)
- [ ] T143 [P] 測試並驗證 ≤15% 效能開銷目標

#### 8.3 URL 偵測整合 (FR-026)

- [x] T144 實作 detect_url_module Tauri command
- [x] T145 整合到前端 URL 輸入流程
- [x] T146 實作 URL 偵測指示器 (顯示使用的模組，僅在失敗時)

#### 8.4 審計日誌 UI

- [ ] T147 [P] 實作審計日誌查看組件 src/components/AuditLogViewer.tsx
- [ ] T148 [P] 實作日誌過濾和搜尋功能
- [ ] T149 [P] 整合到模組詳細資訊面板

#### 8.5 模組開發者體驗

- [X] T150 建立 TypeScript 型別定義檔 modules/@types/zoo-downloader/index.d.ts
  - 定義 Context, HttpClient, Logger, CaptchaClient 等接口
  - 定義 DownloadInfo, HttpResponse, Credentials 等資料型別
  - 包含完整的 JSDoc 註解和使用範例
  - 提供全域型別別名 (ModuleContext, ModuleDownloadInfo 等)
- [X] T151 建立 TypeScript 型別定義 package.json (modules/@types/zoo-downloader/package.json)
- [X] T152 建立 jsconfig.json (modules/jsconfig.json) 全域配置
- [X] T153 建立 VS Code 設定 (modules/.vscode/settings.json)
- [X] T154 建立 VS Code 程式碼片段 (modules/.vscode/zoo-downloader.code-snippets)
  - zoo-module: 完整模組模板
  - zoo-http-get: HTTP GET 請求
  - zoo-http-post: HTTP POST 請求
  - zoo-captcha: 驗證碼處理
  - zoo-login: 登入流程範例
- [X] T155 完善 modules/_template/ 模板檔案（使用型別定義）
- [ ] T156 更新 modules/k2s/ 範例使用型別定義和 JSDoc
- [ ] T157 撰寫模組開發文件 (參考 quickstart.md)
  - 說明如何使用型別定義獲得 IDE 支援
  - 提供 JSDoc 使用範例
  - 說明 code snippets 使用方法
- [ ] T158 建立模組測試工具和驗證腳本

#### 8.6 國際化完善

- [ ] T159 [P] 新增所有模組系統相關的翻譯鍵
- [ ] T160 [P] 驗證 zh-TW, en, ja 翻譯完整性

#### 8.7 整合測試與文件

- [ ] T161 執行完整的端對端測試 (所有 User Stories)
- [ ] T162 生成測試報告和覆蓋率分析
- [ ] T163 更新 README.md 和使用者文件
- [ ] T164 建立模組開發教學和範例

**完成標準**: 系統穩定、效能符合目標、使用者體驗流暢、文件完整。

---

## Dependencies (User Story 完成順序)

```
Phase 1 (Setup)
    ↓
Phase 2 (Foundational) ← 所有 Stories 的前置條件
    ↓
    ├─→ Phase 3 (US2: 模組架構) ← MVP 核心
    │       ↓
    │   ├─→ Phase 4 (US1: K2S 模組)
    │   │       ↓
    │   │   └─→ Phase 5 (US5: 安全測試)
    │   │
    │   ├─→ Phase 6 (US3: 認證管理) ← 依賴 Phase 4 的認證代理
    │   │
    │   └─→ Phase 7 (US4: 模組管理 UI)
    │
    └─→ Phase 8 (Polish) ← 所有 Stories 完成後
```

**關鍵依賴說明：**
- **Phase 2 是所有 Stories 的阻塞性前置條件** - 必須先完成
- **US2 (Phase 3) 是 MVP 核心** - 必須第二完成
- **US1 (Phase 4) 依賴 US2** - 需要模組架構才能實作 K2S
- **US5 (Phase 5) 依賴 US1** - 需要真實模組才能測試安全性
- **US3 (Phase 6) 依賴 US1** - 需要認證代理 API
- **US4 (Phase 7) 只依賴 US2** - 可以在 US1 完成前開始
- **Phase 8 依賴所有 Stories** - 最後的優化和整合

---

## Parallel Execution Opportunities

### Phase 2 (Foundational)
可並行執行：**T011-T015 (Manifest)**, **T016-T020 (Runtime)**, **T021-T025 (Sandbox)**, **T026-T030 (Audit)**, **T031-T035 (Permission)**
這 5 個子系統相對獨立，可由不同開發者同時進行。

### Phase 3 (US2)
可並行執行：**T049-T053 (Resource Limiter)** 與 **T036-T048 (Module Wrapper + Registry)**
資源限制器與模組載入邏輯可以並行開發。

### Phase 4 (US1)
可並行執行：
- **T063-T067 (Network Monitor)**
- **T068-T072 (URL Validator)**
- **T078-T083 (K2S Module)** (在 Context API 完成後)

### Phase 5 (US5)
可並行執行：**T090-T097 (所有惡意測試模組)**
8 個測試模組可以同時建立。

### Phase 6-7 (US3, US4)
**US3 和 US4 可以並行開發** - 它們都只依賴 Phase 3，不互相依賴。

### Phase 8 (Polish)
大部分任務 (T138-T154) 可以並行執行。

---

## Implementation Strategy

### MVP 定義 (最小可行產品)

**MVP = Phase 1 + Phase 2 + Phase 3 (US2 核心)**

包含：
- ✅ 模組系統基礎架構 (manifest, runtime, sandbox, audit, permission)
- ✅ 模組註冊、發現、載入機制
- ✅ 基本的安全防護 (沙箱、權限檢查)
- ✅ 可以載入和執行測試模組
- ❌ 尚未包含 K2S 實作
- ❌ 尚未包含認證管理 UI
- ❌ 尚未包含模組管理 UI

**MVP 完成後即可驗證架構可行性**，然後再增量添加功能。

### 建議實作順序

1. **Week 1-2**: Phase 1 + Phase 2 (Setup + Foundational)
2. **Week 3**: Phase 3 (US2 - 模組架構) → **MVP 完成**
3. **Week 4**: Phase 4 (US1 - K2S 模組)
4. **Week 5**: Phase 5 (US5 - 安全測試)
5. **Week 6**: Phase 6 + Phase 7 (US3 + US4 並行)
6. **Week 7**: Phase 8 (Polish)

### 驗證點

- **MVP 驗證** (Phase 3 完成後)：載入測試模組並執行
- **K2S 驗證** (Phase 4 完成後)：成功下載 K2S 檔案
- **安全驗證** (Phase 5 完成後)：所有安全測試通過
- **UI 驗證** (Phase 7 完成後)：使用者能管理模組和認證
- **最終驗證** (Phase 8 完成後)：所有 User Stories 完整測試

---

## Notes

### 任務標記說明

- **[P]**: 可並行執行的任務（不同檔案，無依賴）
- **[US1]** - **[US5]**: User Story 標籤，對應 spec.md 中的 User Stories
- **無標籤**: 設定和基礎任務

### 檔案路徑慣例

- **Rust 後端**: `src-tauri/src/modules/*.rs`
- **前端組件**: `src/components/*.tsx`
- **API 客戶端**: `src/api/modules.ts`
- **型別定義**: `src/types/module.ts`
- **模組檔案**: `modules/{module-id}/manifest.json`, `modules/{module-id}/index.js`

### 開發建議

1. **嚴格遵循 MVP-first 策略** - 不要跳過 Phase 2 和 Phase 3
2. **每個 Phase 完成後執行測試** - 確保獨立測試標準達成
3. **善用並行機會** - 充分利用標記為 [P] 的任務
4. **頻繁提交** - 小步驟提交，方便回滾和審查
5. **安全優先** - Phase 5 的安全測試絕不能跳過

### 參考文件

- **設計文件**: `specs/004-site-module-integration/plan.md`
- **資料模型**: `specs/004-site-module-integration/data-model.md`
- **API 契約**: `specs/004-site-module-integration/contracts/tauri-commands.md`
- **研究報告**: `specs/004-site-module-integration/research.md`
- **快速入門**: `specs/004-site-module-integration/quickstart.md`

---

**Last Updated**: 2025-12-07
**Total Tasks**: 169
**Estimated Duration**: 7 weeks
**MVP Milestone**: Phase 3 完成 (約 3 weeks)
**Developer Experience**: TypeScript 型別定義 + VS Code snippets (Phase 8)
