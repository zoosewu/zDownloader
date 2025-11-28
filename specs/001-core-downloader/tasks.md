# Tasks: Zoo Downloader - 跨平台下載器核心功能

**Input**: Design documents from `/nodejs/zoo-downloader/specs/001-core-downloader/`
**Prerequisites**: plan.md ✅, spec.md ✅, PLANNING_RECORD.md ✅

**Tests**: 測試任務將在 Phase N: Polish 階段執行（非 TDD 方式）

**Organization**: 任務按使用者故事（P1-P8）組織，每個故事可獨立實作和測試

## Format: `[ID] [P?] [Story] Description`

- **[P]**: 可並行執行（不同檔案，無依賴）
- **[Story]**: 此任務所屬的使用者故事（例如 US1, US2, US3）
- 包含明確的檔案路徑

## Path Conventions

專案使用 Tauri 結構：
- **Rust 後端**: `src-tauri/src/`
- **React 前端**: `src/`
- **規格文件**: `specs/001-core-downloader/`

---

## Phase 1: Setup (專案初始化)

**目的**: 建立專案基礎結構和依賴安裝

- [X] T001 使用 Tauri CLI 建立專案結構：執行 `npm create tauri-app` 並配置為 React + TypeScript
- [X] T002 配置 Rust 依賴在 src-tauri/Cargo.toml：tauri 2.x, tokio, reqwest, serde, uuid, chrono, libloading, headless_chrome, keyring
- [X] T003 [P] 配置 React 依賴在 package.json：React 19, Material UI 6, Jotai, react-i18next, i18next
- [X] T004 [P] 建立目錄結構：src-tauri/src/{commands,download,proxy,modules,storage,utils,models}
- [X] T005 [P] 建立前端目錄結構：src/{components,pages,stores,hooks,locales,styles,types}
- [X] T006 [P] 配置 Tauri 應用程式設定在 src-tauri/tauri.conf.json：SystemTray, Notification 功能
- [X] T007 [P] 配置 Rust linting：cargo clippy 和 rustfmt
- [X] T008 [P] 配置 TypeScript 嚴格模式和 ESLint

---

## Phase 2: Foundational (阻塞性先決條件)

**目的**: 核心基礎設施，所有使用者故事的前置條件

**⚠️ 關鍵**: 必須完成此階段後才能開始任何使用者故事

### 資料模型與儲存層

- [X] T009 [P] 定義 DownloadTask 結構體在 src-tauri/src/models/download_task.rs：包含 10 種狀態、狀態轉換邏輯
- [X] T010 [P] 定義 DownloadSegment 結構體在 src-tauri/src/models/download_segment.rs
- [X] T011 [P] 定義 ProxyInfo 結構體在 src-tauri/src/models/proxy_info.rs：包含健康度評分欄位
- [X] T012 [P] 定義 AppSettings 結構體在 src-tauri/src/models/settings.rs：5 大類設定
- [X] T013 [P] 實作 JSON 儲存層在 src-tauri/src/storage/mod.rs：讀寫 config.json, history.json
- [X] T014 實作跨平台路徑處理在 src-tauri/src/storage/paths.rs：Windows/macOS/Linux 資料目錄
- [X] T015 實作 DownloadTask 持久化在 src-tauri/src/storage/tasks.rs：儲存到 downloads/{task_id}.json

### Tauri 命令基礎

- [X] T016 建立 Tauri 命令模組結構在 src-tauri/src/commands/mod.rs：匯出 download, proxy, settings 命令
- [X] T017 [P] 建立錯誤處理系統在 src-tauri/src/utils/error.rs：定義統一的 Result<T, AppError> 類型
- [X] T018 [P] 建立事件發送工具在 src-tauri/src/utils/events.rs：封裝 emit() 函數
- [X] T019 在 src-tauri/src/main.rs 註冊所有 Tauri 命令並啟動應用程式

### 前端基礎

- [X] T020 [P] 定義 TypeScript 類型在 src/types/download.ts, src/types/proxy.ts, src/types/settings.ts
- [X] T021 [P] 建立 Material UI 主題配置在 src/styles/theme.ts：淺色/深色模式
- [X] T022 [P] 配置 i18n 在 src/locales/：建立 zh-TW.json, en.json, ja.json 骨架
- [X] T023 建立主應用元件在 src/App.tsx：路由、主題提供者、i18n 提供者
- [X] T024 [P] 建立 Jotai 狀態原子在 src/stores/settingsStore.ts：全域設定狀態

**Checkpoint**: 基礎設施完成 - 使用者故事實作可以開始

---

## Phase 3: User Story 1 - 基本下載功能 (Priority: P1) 🎯 MVP

**目標**: 使用者可以貼上 URL 或從剪貼簿自動偵測來下載單一檔案，即時查看進度並控制下載

**獨立測試**: 貼上一個 HTTP/HTTPS URL，點擊下載按鈕，可以看到下載進度並成功下載檔案。可以暫停、繼續或取消下載

### 核心下載引擎

- [X] T025 [P] [US1] 實作 HTTP 客戶端封裝在 src-tauri/src/download/client.rs：使用 reqwest 處理 HTTP/HTTPS
- [X] T026 [US1] 實作下載引擎核心在 src-tauri/src/download/engine.rs：單線程下載、進度追蹤、狀態管理
- [X] T027 [US1] 實作下載佇列管理在 src-tauri/src/download/queue.rs：任務佇列、狀態同步
- [X] T028 [US1] 實作檔案寫入邏輯在 src-tauri/src/download/writer.rs：處理檔名衝突（自動重命名）

### Tauri 命令實作

- [X] T029 [P] [US1] 實作 create_download 命令在 src-tauri/src/commands/download.rs：建立下載任務
- [X] T030 [P] [US1] 實作 start_download 命令在 src-tauri/src/commands/download.rs：開始下載
- [X] T031 [P] [US1] 實作 pause_download 命令在 src-tauri/src/commands/download.rs：暫停下載
- [X] T032 [P] [US1] 實作 cancel_download 命令在 src-tauri/src/commands/download.rs：取消下載並清理
- [X] T033 [P] [US1] 實作 get_download_list 命令在 src-tauri/src/commands/download.rs：取得下載列表
- [X] T034 [US1] 實作進度事件發送在 src-tauri/src/download/engine.rs：發送 progress-updated 和 status-changed 事件

### 剪貼簿監聽

- [X] T035 [US1] 實作剪貼簿監聽在 src-tauri/src/utils/clipboard.rs：200ms 輪詢 + URL 正則驗證
- [X] T036 [US1] 發送 clipboard-detected 事件並整合到下載引擎

### 前端 UI（User Story 1）

- [X] T037 [P] [US1] 建立下載列表元件在 src/components/DownloadList.tsx：顯示所有下載任務
- [X] T038 [P] [US1] 建立下載項目元件在 src/components/DownloadItem.tsx：單一下載任務 UI（進度、速度、控制按鈕）
- [X] T039 [P] [US1] 建立進度條元件在 src/components/ProgressBar.tsx：Material UI LinearProgress
- [X] T040 [P] [US1] 建立 URL 輸入元件在 src/components/UrlInput.tsx：貼上 URL、顯示剪貼簿提示
- [X] T041 [US1] 建立下載狀態管理在 src/stores/downloadStore.ts：Jotai atoms 管理下載列表和進度
- [X] T042 [US1] 建立 Tauri 事件監聽 hook 在 src/hooks/useTauriEvents.ts：監聽 progress-updated, status-changed, clipboard-detected
- [X] T043 [US1] 建立主頁面在 src/pages/MainPage.tsx：整合 UrlInput + DownloadList
- [X] T044 [US1] 實作下載控制邏輯在 src/hooks/useDownloads.ts：呼叫 Tauri 命令（create, start, pause, cancel）

### 系統通知

- [X] T045 [US1] 實作下載完成通知在 src-tauri/src/utils/notification.rs：使用 Tauri Notification API

**Checkpoint**: User Story 1 完成 - MVP 可運作！使用者可以下載單一檔案、查看進度、暫停/繼續/取消

---

## Phase 4: User Story 2 - 批次下載管理 (Priority: P2)

**目標**: 使用者可以匯入文字檔案批次新增多個 URL，並管理下載佇列

**獨立測試**: 準備一個包含多個 URL 的文字檔（每行一個），匯入後可以看到所有下載任務排隊，並能管理佇列

### 後端批次匯入

- [X] T046 [US2] 實作批次匯入命令在 src-tauri/src/commands/download.rs：import_batch_downloads(file_path)
- [X] T047 [US2] 實作文字檔解析在 src-tauri/src/utils/parser.rs：解析換行分隔的 URL 列表
- [X] T048 [US2] 擴充下載佇列管理在 src-tauri/src/download/queue.rs：支援同時下載數量限制

### 前端批次管理 UI

- [X] T049 [P] [US2] 建立批次匯入按鈕元件在 src/components/BatchImportButton.tsx：開啟檔案選擇對話框
- [X] T050 [P] [US2] 建立佇列控制元件在 src/components/QueueControls.tsx：全部暫停、全部開始、清除已完成
- [X] T051 [US2] 擴充 downloadStore 在 src/stores/downloadStore.ts：支援批次操作
- [X] T052 [US2] 更新 MainPage 在 src/pages/MainPage.tsx：整合批次匯入和佇列控制

**Checkpoint**: User Story 2 完成 - 使用者可以批次匯入 URL 並管理下載佇列

---

## Phase 5: User Story 3 - 多線程分段下載與斷點續傳 (Priority: P3)

**目標**: 大檔案自動使用多線程分段下載加速，網路中斷後能恢復下載

**獨立測試**: 下載大檔案（> 100MB），系統使用多個線程加速。模擬網路中斷後，重新啟動應用程式能恢復下載

### 分段下載引擎

- [X] T053 [US3] 實作 Range Request 檢測在 src-tauri/src/download/client.rs：HEAD 請求檢查 Accept-Ranges
- [X] T054 [US3] 實作分段策略在 src-tauri/src/download/segment.rs：固定 10MB 分段、可配置
- [ ] T055 [US3] 重構下載引擎在 src-tauri/src/download/engine.rs：支援多線程分段下載（tokio::spawn）
- [ ] T056 [US3] 實作分段合併在 src-tauri/src/download/merger.rs：所有分段完成後合併為完整檔案
- [ ] T057 [US3] 實作分段進度追蹤在 src-tauri/src/download/engine.rs：彙總所有分段的進度

### 斷點續傳

- [ ] T058 [US3] 實作斷點續傳邏輯在 src-tauri/src/download/resume.rs：檢查部分檔案完整性（±1 byte 容差）
- [ ] T059 [US3] 實作未完成下載偵測在 src-tauri/src/main.rs：應用程式啟動時掃描未完成的下載
- [ ] T060 [US3] 實作恢復下載 UI 提示在 src/components/ResumeDialog.tsx：詢問使用者是否恢復下載

### 前端分段進度顯示

- [ ] T061 [US3] 擴充 DownloadItem 元件在 src/components/DownloadItem.tsx：顯示分段進度（選用）
- [ ] T062 [US3] 更新 downloadStore 在 src/stores/downloadStore.ts：支援分段資訊

**Checkpoint**: User Story 3 完成 - 多線程分段下載和斷點續傳運作正常

---

## Phase 6: User Story 4 - Proxy 支援與管理 (Priority: P4)

**目標**: 使用者可以配置 HTTP Proxy，系統自動驗證 Proxy 可用性，並在下載時動態分配和輪替 Proxy

**獨立測試**: 匯入 Proxy 列表，系統自動驗證並使用可用 Proxy 進行下載。當某個 Proxy 失敗時，自動切換

### Proxy 池管理

- [ ] T063 [P] [US4] 實作 Proxy 池核心在 src-tauri/src/proxy/pool.rs：管理 Proxy 列表、健康度評分
- [ ] T064 [P] [US4] 實作 Proxy 驗證器在 src-tauri/src/proxy/validator.rs：批次驗證（每批 50 個）、5 秒 timeout
- [ ] T065 [P] [US4] 實作健康度評分算法在 src-tauri/src/proxy/health_scorer.rs：成功率 70% + 延遲 20% + 最近使用 10%
- [ ] T066 [US4] 實作 Proxy 快取在 src-tauri/src/proxy/cache.rs：讀寫 proxies.txt
- [ ] T067 [US4] 實作 Proxy 自動切換在 src-tauri/src/download/engine.rs：失敗時切換到下一個可用 Proxy
- [ ] T068 [US4] 實作優雅降級在 src-tauri/src/download/engine.rs：所有 Proxy 失效時詢問使用者是否使用直接連線

### Proxy 管理命令

- [ ] T069 [P] [US4] 實作 import_proxy_list 命令在 src-tauri/src/commands/proxy.rs
- [ ] T070 [P] [US4] 實作 get_proxy_list 命令在 src-tauri/src/commands/proxy.rs
- [ ] T071 [P] [US4] 實作 remove_proxy 命令在 src-tauri/src/commands/proxy.rs

### 前端 Proxy 管理 UI

- [ ] T072 [P] [US4] 建立 Proxy 設定面板在 src/components/ProxySettings.tsx：匯入列表、顯示 Proxy 狀態
- [ ] T073 [P] [US4] 建立 Proxy 狀態管理在 src/stores/proxyStore.ts：Jotai atoms
- [ ] T074 [US4] 建立設定頁面在 src/pages/SettingsPage.tsx：整合 ProxySettings
- [ ] T075 [US4] 更新 DownloadItem 在 src/components/DownloadItem.tsx：顯示正在使用的 Proxy

**Checkpoint**: User Story 4 完成 - Proxy 池管理和動態切換運作正常

---

## Phase 7: User Story 5 - 模組化網站支援 (Priority: P5)

**目標**: 使用者可以輸入特定網站的頁面 URL，系統透過網站模組自動解析並取得下載連結

**獨立測試**: 輸入一個檔案分享網站的頁面 URL，系統使用對應模組解析頁面並自動開始下載

### 模組系統核心

- [ ] T076 [P] [US5] 定義 WebsiteModule Trait 在 src-tauri/src/modules/interface.rs：metadata, can_handle, extract_download_info, handle_captcha
- [ ] T077 [US5] 實作模組載入器在 src-tauri/src/modules/loader.rs：使用 libloading 動態載入 .so/.dll/.dylib
- [ ] T078 [US5] 實作模組註冊表在 src-tauri/src/modules/registry.rs：掃描模組目錄、註冊模組
- [ ] T079 [US5] 整合 headless browser 在 src-tauri/src/modules/browser.rs：使用 headless_chrome 執行 JavaScript

### CAPTCHA 處理

- [ ] T080 [US5] 實作 CAPTCHA 事件在 src-tauri/src/modules/interface.rs：發送 captcha-required 事件
- [ ] T081 [US5] 實作 CAPTCHA 回調在 src-tauri/src/modules/interface.rs：接收使用者輸入

### 憑證管理

- [ ] T082 [US5] 實作憑證加密在 src-tauri/src/utils/crypto.rs：使用 keyring crate（OS Keychain）
- [ ] T083 [US5] 實作憑證儲存/讀取在 src-tauri/src/modules/auth.rs

### 範例模組

- [ ] T084 [US5] 建立範例模組專案在 modules/example-downloader/：Cargo.toml + src/lib.rs
- [ ] T085 [US5] 實作範例模組在 modules/example-downloader/src/lib.rs：示範如何實作 WebsiteModule Trait

### 前端 CAPTCHA UI

- [ ] T086 [US5] 建立 CAPTCHA 對話框在 src/components/CaptchaDialog.tsx：顯示 CAPTCHA 圖片、接收輸入
- [ ] T087 [US5] 建立 CAPTCHA 事件監聽在 src/hooks/useTauriEvents.ts：監聽 captcha-required
- [ ] T088 [US5] 建立登入對話框在 src/components/LoginDialog.tsx：輸入帳號密碼

**Checkpoint**: User Story 5 完成 - 模組化網站支援運作正常，可以擴充支援更多網站

---

## Phase 8: User Story 6 - Chrome 瀏覽器整合 (Priority: P6)

**目標**: 使用者在 Chrome 瀏覽器中瀏覽時，擴充套件自動偵測頁面中的下載連結並可直接傳送到下載器

**獨立測試**: 安裝 Chrome 擴充套件，瀏覽包含下載連結的網頁，擴充套件自動偵測並顯示圖示，點擊後連結傳送到下載器

### Chrome 擴充套件

- [ ] T089 [P] [US6] 建立擴充套件結構在 chrome-extension/：manifest.json, background.js, content.js
- [ ] T090 [P] [US6] 實作連結偵測在 chrome-extension/content.js：偵測頁面中的下載連結
- [ ] T091 [US6] 實作背景腳本在 chrome-extension/background.js：與應用程式通訊

### Native Messaging

- [ ] T092 [US6] 實作 Native Messaging host 在 src-tauri/src/native_messaging/：接收來自 Chrome 的訊息
- [ ] T093 [US6] 配置 Native Messaging manifest 在 chrome-extension/native-messaging/：Windows/macOS/Linux 路徑
- [ ] T094 [US6] 整合到下載引擎在 src-tauri/src/main.rs：處理從 Chrome 傳來的 URL

**Checkpoint**: User Story 6 完成 - Chrome 瀏覽器整合運作正常

---

## Phase 9: User Story 7 - 排程與自動化下載 (Priority: P7)

**目標**: 使用者可以設定排程下載任務，在指定時間自動開始下載，並配置自動重試機制

**獨立測試**: 設定一個下載任務在未來某個時間開始，到了時間系統自動開始下載。如果下載失敗，系統自動重試

### 排程系統

- [ ] T095 [US7] 實作排程管理在 src-tauri/src/download/scheduler.rs：任務排程、時間觸發
- [ ] T096 [US7] 實作自動重試在 src-tauri/src/download/retry.rs：可配置次數和間隔
- [ ] T097 [US7] 實作背景服務在 src-tauri/src/background_service.rs：應用程式關閉時仍可執行排程

### 排程 UI

- [ ] T098 [US7] 建立排程對話框在 src/components/ScheduleDialog.tsx：選擇日期和時間
- [ ] T099 [US7] 更新 DownloadItem 在 src/components/DownloadItem.tsx：顯示排程時間
- [ ] T100 [US7] 建立重試設定在 src/components/RetrySettings.tsx：配置重試次數和間隔

**Checkpoint**: User Story 7 完成 - 排程下載和自動重試運作正常

---

## Phase 10: User Story 8 - 進階功能（分類、通知、設定） (Priority: P8)

**目標**: 使用者可以透過分類管理下載檔案、接收下載完成通知、自訂應用程式設定

**獨立測試**: 下載不同類型的檔案，系統自動分類到不同資料夾。下載完成後收到系統通知。可以在設定中調整語言、主題等

### 自動分類

- [ ] T101 [US8] 實作檔案類型偵測在 src-tauri/src/utils/file_type.rs：根據副檔名分類（影片、音訊、文件、壓縮檔等）
- [ ] T102 [US8] 實作自動分類在 src-tauri/src/download/writer.rs：下載完成後移動到對應子資料夾

### 系統托盤

- [ ] T103 [US8] 實作系統托盤在 src-tauri/src/main.rs：使用 Tauri SystemTray API
- [ ] T104 [US8] 實作托盤圖示進度在 src-tauri/src/utils/tray.rs：顯示總下載進度
- [ ] T105 [US8] 實作托盤選單在 src-tauri/src/utils/tray.rs：顯示/隱藏、暫停全部、退出

### 多語言與主題

- [ ] T106 [P] [US8] 完善翻譯檔案在 src/locales/zh-TW.json：所有 UI 文字的繁體中文翻譯
- [ ] T107 [P] [US8] 完善翻譯檔案在 src/locales/en.json：所有 UI 文字的英文翻譯
- [ ] T108 [P] [US8] 完善翻譯檔案在 src/locales/ja.json：所有 UI 文字的日文翻譯
- [ ] T109 [US8] 實作語言切換在 src/App.tsx：根據設定動態切換語言
- [ ] T110 [US8] 實作主題切換在 src/App.tsx：淺色/深色模式切換

### 設定管理

- [ ] T111 [P] [US8] 實作 get_settings 命令在 src-tauri/src/commands/settings.rs
- [ ] T112 [P] [US8] 實作 update_settings 命令在 src-tauri/src/commands/settings.rs
- [ ] T113 [P] [US8] 實作 export_settings 命令在 src-tauri/src/commands/settings.rs
- [ ] T114 [P] [US8] 實作 import_settings 命令在 src-tauri/src/commands/settings.rs
- [ ] T115 [US8] 建立設定對話框在 src/components/SettingsDialog.tsx：所有設定項目的 UI
- [ ] T116 [US8] 完善設定頁面在 src/pages/SettingsPage.tsx：整合所有設定元件

### 下載歷史

- [ ] T117 [US8] 實作下載歷史管理在 src-tauri/src/storage/history.rs：讀寫 history.json
- [ ] T118 [US8] 建立歷史頁面在 src/pages/HistoryPage.tsx：顯示所有歷史下載、篩選、搜尋

### 自動更新

- [ ] T119 [US8] 配置 Tauri Updater 在 src-tauri/tauri.conf.json：自動更新設定
- [ ] T120 [US8] 實作更新檢查在 src-tauri/src/main.rs：定期檢查新版本
- [ ] T121 [US8] 建立更新通知在 src/components/UpdateNotification.tsx：提示使用者下載更新

**Checkpoint**: User Story 8 完成 - 所有進階功能運作正常

---

## Phase N: Polish & Cross-Cutting Concerns

**目的**: 優化、測試和文件完善

### 測試

- [ ] T122 [P] 撰寫 Rust 單元測試在 src-tauri/src/download/engine_test.rs：測試下載引擎核心功能
- [ ] T123 [P] 撰寫 Rust 單元測試在 src-tauri/src/proxy/pool_test.rs：測試 Proxy 池管理
- [ ] T124 [P] 撰寫 Rust 整合測試在 tests/download_test.rs：端對端下載流程測試
- [ ] T125 [P] 撰寫 React 元件測試在 src/components/__tests__/：使用 Jest + React Testing Library

### 效能優化與驗證

- [ ] T126 驗證下載速度提升在測試環境：多線程 vs 單線程對比測試，確認 > 50% 提升（SC-002）
- [ ] T127 驗證並發下載在測試環境：測試 100 個並發下載，確認 CPU < 80%, 記憶體 < 2GB（SC-003）
- [ ] T128 驗證斷點續傳成功率：網路中斷測試 100 次，確認成功率 > 95%（SC-004）
- [ ] T129 驗證 Proxy 驗證準確率：對比驗證結果與實際下載成功率，確認 > 90%（SC-005）
- [ ] T130 驗證 UI 響應時間：測量點擊到 UI 更新延遲，確認 < 100ms（SC-009）
- [ ] T131 驗證啟動時間：測量冷啟動到主視窗顯示，確認 < 3 秒（SC-008）

### 跨平台測試

- [ ] T132 [P] 在 Windows 10+ 上測試所有功能：路徑處理、系統托盤、通知、憑證儲存
- [ ] T133 [P] 在 macOS 11+ 上測試所有功能：路徑處理、系統托盤、通知、憑證儲存
- [ ] T134 [P] 在 Ubuntu 20.04+ 上測試所有功能：路徑處理、系統托盤、通知、憑證儲存
- [ ] T135 驗證設定檔跨平台移植：匯出設定並在不同平台匯入，確認 100% 成功（SC-014）

### 文件與開發者體驗

- [ ] T136 [P] 從 PLANNING_RECORD.md 建立 research.md 在 specs/001-core-downloader/：技術研究詳細內容
- [ ] T137 [P] 從 PLANNING_RECORD.md 建立 data-model.md 在 specs/001-core-downloader/：資料模型詳細內容
- [ ] T138 [P] 從 PLANNING_RECORD.md 建立 contracts/ 目錄在 specs/001-core-downloader/：所有 API 合約
- [ ] T139 [P] 從 PLANNING_RECORD.md 建立 quickstart.md 在 specs/001-core-downloader/：開發者入門指南
- [ ] T140 [P] 撰寫 README.md 在專案根目錄：專案介紹、安裝、使用說明
- [ ] T141 [P] 撰寫模組開發指南在 docs/module-development.md：教導開發者如何建立網站模組
- [ ] T142 執行 quickstart.md 驗證：按照 quickstart.md 步驟從零開始建立開發環境，確認無誤

### 安全與品質

- [ ] T143 [P] 執行 cargo clippy 並修正所有警告
- [ ] T144 [P] 執行 cargo fmt 格式化所有 Rust 程式碼
- [ ] T145 [P] 執行 ESLint 並修正所有警告
- [ ] T146 安全性審查：檢查憑證儲存、HTTPS 驗證、Proxy 使用透明性
- [ ] T147 錯誤訊息友善性審查：確保所有錯誤訊息提供可操作的建議

### 建構與發布

- [ ] T148 配置建構腳本在 package.json：npm run tauri build
- [ ] T149 測試建構產物：Windows (.exe), macOS (.app), Linux (.AppImage)
- [ ] T150 準備發布：建立 GitHub Release、撰寫 CHANGELOG

---

## Dependencies & Execution Order

### Phase 依賴關係

- **Setup (Phase 1)**: 無依賴 - 可立即開始
- **Foundational (Phase 2)**: 依賴 Setup 完成 - **阻塞所有使用者故事**
- **User Stories (Phase 3-10)**: 全部依賴 Foundational 完成
  - User Story 1-8 可以並行開發（如果有多個開發者）
  - 或按優先級順序開發（P1 → P2 → P3 → ... → P8）
- **Polish (Phase N)**: 依賴所有需要的使用者故事完成

### User Story 依賴關係

- **User Story 1 (P1)**: 可在 Foundational 完成後立即開始 - 無其他依賴
- **User Story 2 (P2)**: 可在 Foundational 完成後立即開始 - 輕度依賴 US1（擴充佇列管理）
- **User Story 3 (P3)**: 依賴 US1（重構下載引擎）
- **User Story 4 (P4)**: 可在 Foundational 完成後立即開始 - 與 US3 整合（Proxy 分配到分段）
- **User Story 5 (P5)**: 可在 Foundational 完成後立即開始 - 獨立於其他故事
- **User Story 6 (P6)**: 依賴 US1（呼叫 create_download）
- **User Story 7 (P7)**: 依賴 US1（排程使用下載引擎）
- **User Story 8 (P8)**: 可在 Foundational 完成後立即開始 - 跨故事整合功能

### 每個 User Story 內部順序

- 後端模型 → 後端服務 → Tauri 命令 → 前端狀態管理 → 前端 UI
- 標記 [P] 的任務可並行執行（不同檔案，無依賴）

### 並行機會

- Setup 階段所有標記 [P] 的任務可並行
- Foundational 階段所有標記 [P] 的任務可並行
- 每個 User Story 內部標記 [P] 的任務可並行
- 不同 User Story 可由不同開發者並行開發（完成 Foundational 後）

---

## Parallel Example: User Story 1

```bash
# 並行執行 US1 的核心下載引擎任務（不同檔案）:
Task T025: "實作 HTTP 客戶端封裝在 src-tauri/src/download/client.rs"
Task T029: "實作 create_download 命令在 src-tauri/src/commands/download.rs"
Task T030: "實作 start_download 命令在 src-tauri/src/commands/download.rs"
Task T031: "實作 pause_download 命令在 src-tauri/src/commands/download.rs"

# 並行執行 US1 的前端 UI 任務:
Task T037: "建立下載列表元件在 src/components/DownloadList.tsx"
Task T038: "建立下載項目元件在 src/components/DownloadItem.tsx"
Task T039: "建立進度條元件在 src/components/ProgressBar.tsx"
Task T040: "建立 URL 輸入元件在 src/components/UrlInput.tsx"
```

---

## Implementation Strategy

### MVP First (僅 User Story 1)

1. 完成 Phase 1: Setup
2. 完成 Phase 2: Foundational（**關鍵** - 阻塞所有故事）
3. 完成 Phase 3: User Story 1
4. **停止並驗證**: 測試 User Story 1 是否獨立運作
5. 準備 Demo 或部署 MVP

### Incremental Delivery（增量交付）

1. 完成 Setup + Foundational → 基礎準備完成
2. 新增 User Story 1 → 獨立測試 → 部署/Demo（**MVP!**）
3. 新增 User Story 2 → 獨立測試 → 部署/Demo
4. 新增 User Story 3 → 獨立測試 → 部署/Demo
5. 新增 User Story 4 → 獨立測試 → 部署/Demo
6. 以此類推...每個故事增加價值而不破壞之前的故事

### Parallel Team Strategy（多人並行）

如果有多個開發者：

1. 團隊一起完成 Setup + Foundational
2. Foundational 完成後：
   - 開發者 A: User Story 1（MVP 優先）
   - 開發者 B: User Story 2（批次下載）
   - 開發者 C: User Story 4（Proxy 池 - 可獨立開發）
3. 各故事獨立完成並整合

---

## Summary

### 任務統計

- **總任務數**: 150 個任務
- **Phase 1 (Setup)**: 8 個任務
- **Phase 2 (Foundational)**: 16 個任務（**關鍵阻塞點**）
- **Phase 3 (US1 - MVP)**: 21 個任務
- **Phase 4 (US2)**: 7 個任務
- **Phase 5 (US3)**: 10 個任務
- **Phase 6 (US4)**: 13 個任務
- **Phase 7 (US5)**: 13 個任務
- **Phase 8 (US6)**: 6 個任務
- **Phase 9 (US7)**: 6 個任務
- **Phase 10 (US8)**: 21 個任務
- **Phase N (Polish)**: 29 個任務

### 並行機會識別

- **Setup 階段**: 6 個任務可並行（標記 [P]）
- **Foundational 階段**: 9 個任務可並行
- **每個 User Story**: 平均 30-40% 的任務可並行
- **跨 User Story**: US1, US2, US4, US5 可並行開發（完成 Foundational 後）

### 各 User Story 任務數

- US1 (基本下載): 21 個任務 - **MVP 核心**
- US2 (批次下載): 7 個任務
- US3 (分段/斷點續傳): 10 個任務
- US4 (Proxy 管理): 13 個任務
- US5 (模組化網站): 13 個任務
- US6 (Chrome 整合): 6 個任務
- US7 (排程下載): 6 個任務
- US8 (進階功能): 21 個任務

### 建議 MVP 範圍

**僅 User Story 1 (P1)** - 基本下載功能:
- 單一檔案下載
- 即時進度顯示
- 暫停/繼續/取消
- 剪貼簿自動偵測
- 下載完成通知

**預估時間**: 2-3 週（1 個全職開發者）

### 獨立測試標準

每個 User Story 完成時應通過以下測試：

- ✅ **US1**: 可以下載單一檔案、查看進度、暫停/繼續/取消
- ✅ **US2**: 可以批次匯入 URL、管理下載佇列
- ✅ **US3**: 大檔案自動分段下載、網路中斷後恢復
- ✅ **US4**: Proxy 自動驗證、動態分配、失敗切換
- ✅ **US5**: 載入網站模組、解析頁面 URL、處理 CAPTCHA
- ✅ **US6**: Chrome 擴充套件偵測連結並傳送到應用程式
- ✅ **US7**: 排程下載在指定時間執行、自動重試
- ✅ **US8**: 自動分類、多語言切換、主題切換、系統托盤

### 格式驗證

✅ **所有任務均遵循清單格式**: `- [ ] [ID] [P?] [Story?] Description with file path`
✅ **所有任務均包含明確的檔案路徑**
✅ **所有 User Story 任務均標記 [Story] 標籤**（例如 [US1], [US2]）
✅ **所有可並行任務均標記 [P]**

---

## Notes

- 標記 [P] 的任務 = 不同檔案、無依賴、可並行
- 標記 [Story] 的任務 = 屬於特定使用者故事，方便追蹤
- 每個使用者故事應獨立完成並可測試
- 在每個 Checkpoint 停止並驗證故事是否獨立運作
- 避免：模糊任務、相同檔案衝突、跨故事依賴破壞獨立性
- 專案憲法合規性已在 plan.md 驗證通過 ✅
- 提交策略：每完成一個任務或邏輯組提交一次
