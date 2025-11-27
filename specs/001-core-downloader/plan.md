# Implementation Plan: Zoo Downloader - 跨平台下載器

**Branch**: `001-core-downloader` | **Date**: 2025-01-27 | **Spec**: [spec.md](./spec.md)

> 📘 **完整規劃記錄**: 請參閱 [PLANNING_RECORD.md](./PLANNING_RECORD.md) 取得所有技術研究、資料模型、API 合約的詳細內容

---

## Summary

**專案目標**: 建立一個跨平台（Windows、Linux、macOS）的 HTTP/HTTPS 下載器，專為大檔案設計，支援多線程分段下載、斷點續傳、動態 Proxy 池管理、模組化網站支援。

**核心價值主張**:
1. **高效能下載**: 多線程分段下載相比單線程提升 50% 速度
2. **可靠性**: 智能斷點續傳（容差 ±1 byte），成功率 > 95%
3. **模組化**: 執行時動態載入網站支援模組，社群可擴充
4. **自動化**: Proxy 自動驗證/切換、剪貼簿監聽、自動重試
5. **跨平台**: Windows/macOS/Linux 一致體驗

**技術方法**:
- 後端: Rust + Tauri 2.x (高效能、記憶體安全、跨平台)
- 前端: React 19 + Material UI 6 + Jotai (現代、響應式、扁平化)
- 模組系統: libloading + Trait 介面 (執行時動態載入)
- 儲存: JSON 檔案 (人類可讀、跨平台移植)

---

## Technical Context

### 🔧 核心技術棧

| 類別 | 技術 | 版本 | 用途 |
|------|------|------|------|
| **後端語言** | Rust | 1.75+ | Tauri 核心、下載引擎、Proxy 管理 |
| **前端框架** | React | 19+ | UI 開發 |
| **桌面框架** | Tauri | 2.x | 跨平台桌面應用 |
| **UI 庫** | Material UI | 6+ | 扁平化設計、元件庫 |
| **狀態管理** | Jotai | 2.x | 原子化狀態管理 |
| **國際化** | react-i18next | 14+ | 多語言支援（繁中/英/日） |
| **HTTP 客戶端** | reqwest | 0.11+ | HTTP 請求、Range Request |
| **非同步執行時** | tokio | 1.x | 多線程、非同步 I/O |
| **模組載入** | libloading | 0.8+ | 動態載入 `.so`/`.dll`/`.dylib` |
| **Headless Browser** | headless_chrome | 1.0+ | JavaScript 執行（網站模組） |
| **憑證加密** | keyring | 2.0+ | OS Keychain 整合 |

### 📦 主要依賴

**Rust (src-tauri/Cargo.toml)**:
```toml
[dependencies]
tauri = { version = "2.0", features = ["system-tray", "notification"] }
tokio = { version = "1", features = ["full"] }
reqwest = { version = "0.11", features = ["json", "stream"] }
serde = { version = "1", features = ["derive"] }
serde_json = "1"
uuid = { version = "1", features = ["v4", "serde"] }
chrono = { version = "0.4", features = ["serde"] }
libloading = "0.8"
headless_chrome = "1.0"
keyring = "2.0"
```

**React (package.json)**:
```json
{
  "dependencies": {
    "react": "^19.0.0",
    "react-dom": "^19.0.0",
    "@mui/material": "^6.0.0",
    "@emotion/react": "^11.0.0",
    "@emotion/styled": "^11.0.0",
    "jotai": "^2.0.0",
    "react-i18next": "^14.0.0",
    "i18next": "^23.0.0",
    "@tauri-apps/api": "^2.0.0"
  }
}
```

### 💾 資料儲存

**類型**: JSON 檔案（本地儲存）

**位置**:
- Windows: `%APPDATA%\zoo-downloader\`
- macOS: `~/Library/Application Support/zoo-downloader/`
- Linux: `~/.config/zoo-downloader/`

**檔案結構**:
- `config.json` - 應用程式設定 (AppSettings)
- `downloads/{task_id}.json` - 個別下載任務資料
- `proxies.txt` - Proxy 快取
- `history.json` - 下載歷史記錄

### 🧪 測試策略

**Rust 後端**:
- **框架**: cargo test (內建)
- **類型**: 單元測試、整合測試
- **重點**: 下載引擎、Proxy 池、模組載入器

**React 前端**:
- **框架**: Jest + React Testing Library
- **類型**: 元件測試、整合測試
- **重點**: UI 互動、狀態管理、事件處理

**端對端測試**:
- **框架**: Tauri WebDriver
- **重點**: 關鍵使用者流程（下載、暫停、恢復、Proxy 切換）

### 🎯 目標平台

| 平台 | 最低版本 | 測試優先級 |
|------|----------|------------|
| Windows | 10 (1809+) | 高 |
| macOS | 11.0 (Big Sur) | 高 |
| Linux | Ubuntu 20.04 / Debian 11 | 中 |

### 📊 效能目標（基於 Success Criteria）

| 指標 | 目標值 | 驗證方法 | 規格來源 |
|------|--------|----------|----------|
| **UI 響應時間** | < 100ms | 點擊到 UI 更新的延遲測量 | SC-009 |
| **啟動時間** | < 3 秒 | 冷啟動到主視窗顯示 | SC-008 |
| **下載速度提升** | +50% | 多線程 vs 單線程基準測試 | SC-002 |
| **並發下載數** | 100 個 | 負載測試（無效能降級） | SC-003 |
| **記憶體使用** | < 2GB | 滿載時記憶體監控 | SC-003 |
| **CPU 使用率** | < 80% | 滿載時 CPU 監控 | SC-003 |
| **斷點續傳成功率** | > 95% | 網路中斷恢復測試 100 次 | SC-004 |
| **Proxy 驗證準確率** | > 90% | 驗證結果 vs 實際下載成功率 | SC-005 |
| **Proxy 切換時間** | < 5 秒 | 失敗偵測到切換完成 | SC-006 |

### ⚠️ 約束條件

**資源限制**:
- 記憶體: < 2GB (100 個並發下載)
- CPU: < 80% (滿載下載時)
- 磁碟 I/O: 支援 SSD 和 HDD

**技術約束**:
- **Proxy**: 僅支援 HTTP Proxy（不支援 SOCKS4/SOCKS5）
- **協定**: 僅支援 HTTP/HTTPS（不支援 FTP/Torrent）
- **模組安全**: 不提供模組簽章驗證（假設使用者信任模組）

**平台特定約束**:
- Chrome Extension 僅支援 Chrome 90+（不支援 Firefox/Safari）
- 系統托盤 API 依賴 Tauri 2.x 的跨平台抽象

### 📏 規模與範圍

**預期使用規模**:
- **下載任務**: 100 個並發、10,000 個歷史記錄
- **Proxy 池**: 最多 500 個 Proxy
- **模組數量**: 10-20 個網站支援模組
- **使用者數**: 個人使用（不設計為多使用者系統）

**功能範圍** (詳見 spec.md):
- ✅ 83 個功能需求 (FR-001 ~ FR-083)
- ✅ 8 個優先級使用者故事 (P1-P8)
- ✅ 20 個可測量成功標準 (SC-001 ~ SC-020)

---

## Constitution Check

> ⚠️ **GATE**: 必須通過所有檢查才能進入 Phase 0 研究。Phase 1 設計後需重新驗證。

### ✅ 原則 I: 模組化優先（Modularity First）

**要求**:
- 所有擴充功能（網站支援、Proxy List）必須透過模組系統實作
- 模組必須獨立、可測試、支援執行時動態載入
- 核心不得包含特定網站硬編碼邏輯

**合規性驗證**:
- ✅ **模組系統設計**: 使用 `libloading` 實現執行時動態載入
- ✅ **Trait 介面**: 定義 `WebsiteModule` 和 `ProxyListModule` trait
- ✅ **核心分離**: 下載引擎完全獨立於特定網站邏輯
- ✅ **測試隔離**: 每個模組可獨立測試，不依賴核心實作

**證據**:
- 專案結構中 `src-tauri/src/modules/loader.rs` 負責動態載入
- `contracts/module-interface.md` 定義清晰的模組 API
- 社群可建立自訂模組並放置在 `~/.zoo-downloader/modules/`

---

### ✅ 原則 II: 效能與可靠性（Performance & Reliability）

**要求**:
- 多線程分段下載必須執行緒安全且高效
- 斷點續傳必須可靠（容差 ±1 byte）
- Proxy 失敗必須自動切換
- 錯誤處理必須優雅降級

**合規性驗證**:
- ✅ **執行緒安全**: 使用 Rust 的 `Arc<Mutex<>>` 確保共享狀態安全
- ✅ **分段下載**: reqwest + tokio 實現並行下載，預設 8 執行緒
- ✅ **斷點續傳**: 實作智能檔案完整性檢查（`data-model.md` 定義）
- ✅ **Proxy 切換**: 健康度評分算法 + 自動 fallback（`research.md` 詳述）
- ✅ **降級處理**: Proxy 全失效時詢問使用者是否使用直接連線

**效能指標達成**:
- 下載速度提升 +50% (SC-002)
- 斷點續傳成功率 > 95% (SC-004)
- Proxy 切換時間 < 5 秒 (SC-006)

---

### ✅ 原則 III: 使用者體驗至上（User Experience First）

**要求**:
- UI 必須簡潔、扁平化、響應迅速（Material UI）
- 錯誤訊息必須友善且可操作
- 必須支援多語言（繁中/英/日）
- 自動化功能（剪貼簿、重試、更新）

**合規性驗證**:
- ✅ **Material UI 6**: 現代扁平化設計，遵循 Material Design 3
- ✅ **友善錯誤**: 錯誤訊息包含具體建議（例如：「Proxy 失效，是否使用直接連線？」）
- ✅ **多語言**: react-i18next + JSON 管理翻譯（`src/locales/`）
- ✅ **剪貼簿監聽**: 200ms 輪詢 + URL 正則驗證，自動提示
- ✅ **自動重試**: 可配置次數/間隔（預設 3 次，間隔 5 秒）
- ✅ **檔名衝突**: 自動重命名加編號（`file.zip` → `file (1).zip`）

**UX 指標達成**:
- UI 響應 < 100ms (SC-009)
- 語言切換 < 500ms (SC-011)
- 主題切換 < 300ms (SC-012)
- 首次使用 5 分鐘內完成下載 (SC-016)

---

### ✅ 原則 IV: 跨平台一致性（Cross-Platform Consistency）

**要求**:
- Windows/Linux/macOS 核心功能必須行為一致
- 系統整合應針對各平台優化
- 設定檔必須可跨平台移植（JSON）

**合規性驗證**:
- ✅ **核心一致**: Rust 後端完全跨平台，無平台特定邏輯
- ✅ **系統整合**: Tauri 抽象層處理平台差異（托盤、通知、自動啟動）
- ✅ **路徑處理**: 使用 `PathBuf` 抽象 `\` vs `/` 差異
- ✅ **設定移植**: JSON 格式，可直接複製到其他平台使用

**平台優化**:
- Windows: Credential Manager（憑證）、Windows 通知
- macOS: Keychain（憑證）、macOS 通知中心
- Linux: Secret Service（憑證）、libnotify

**測試策略**:
- 在三個平台上執行相同的整合測試套件
- 設定檔跨平台移植測試（SC-014: 100% 成功率）

---

### ✅ 原則 V: 安全與隱私（Security & Privacy）

**要求**:
- 憑證必須加密儲存
- HTTPS 連線必須驗證憑證
- Proxy 使用必須透明
- 不得包含遙測

**合規性驗證**:
- ✅ **憑證加密**: 使用 OS Keychain（`keyring` crate），最高安全等級
- ✅ **HTTPS 驗證**: reqwest 預設啟用憑證驗證
- ✅ **Proxy 透明**: UI 明確顯示「使用 Proxy: 127.0.0.1:8080」
- ✅ **無遙測**: 專案憲法明確禁止，程式碼審查強制執行

**安全措施**:
- 憑證儲存: Windows Credential Manager / macOS Keychain / Linux Secret Service
- 網路安全: HTTPS 強制、憑證固定（可選）
- 模組隔離: 模組崩潰不影響主程序（process isolation）

---

### 📋 Constitution Check 結論

**狀態**: ✅ **全部通過**

所有五大核心原則均符合要求，無違規項目。設計決策均有明確理由並符合專案目標。

**下一步**: 可以進入 **Phase 0: 技術研究**

---

## Project Structure

### 📁 文件結構（本功能規格）

```
specs/001-core-downloader/
├── spec.md                         # 功能規格（已完成）
├── plan.md                         # 本文件（實施計畫主文件）
├── PLANNING_RECORD.md              # 完整規劃記錄（詳細參考）
├── checklists/
│   └── requirements.md             # 規格品質檢查清單（已通過）
│
├── research.md                     # Phase 0 輸出（待建立）
├── data-model.md                   # Phase 1 輸出（待建立）
├── quickstart.md                   # Phase 1 輸出（待建立）
├── contracts/                      # Phase 1 輸出（待建立）
│   ├── tauri-commands.md           # Tauri 前後端 API 合約
│   ├── module-interface.md         # 模組開發規範
│   └── storage-schema.md           # JSON 儲存結構
│
└── tasks.md                        # Phase 2 輸出（/speckit.tasks 生成）
```

> 📘 **注意**: `research.md`、`data-model.md`、`quickstart.md`、`contracts/` 的詳細內容已記錄在 [PLANNING_RECORD.md](./PLANNING_RECORD.md) 中

### 🏗️ 原始碼結構（repository root）

```
zoo-downloader/
│
├── src-tauri/                      # ⚙️ Rust 後端（Tauri）
│   ├── src/
│   │   ├── main.rs                 # 應用程式進入點、Tauri 設定
│   │   │
│   │   ├── commands/               # 🔌 Tauri 命令處理器（前後端橋接）
│   │   │   ├── mod.rs              # 命令模組匯出
│   │   │   ├── download.rs         # 下載管理命令 ⭐
│   │   │   ├── proxy.rs            # Proxy 管理命令
│   │   │   └── settings.rs         # 設定管理命令
│   │   │
│   │   ├── download/               # 📥 下載引擎核心 ⭐⭐⭐
│   │   │   ├── mod.rs
│   │   │   ├── engine.rs           # 主下載引擎（分段、進度追蹤）⭐
│   │   │   ├── segment.rs          # 分段下載邏輯
│   │   │   ├── resume.rs           # 斷點續傳實作
│   │   │   └── queue.rs            # 下載佇列管理
│   │   │
│   │   ├── proxy/                  # 🌐 Proxy 池管理 ⭐⭐⭐
│   │   │   ├── mod.rs
│   │   │   ├── pool.rs             # Proxy 池核心（驗證、評分）⭐
│   │   │   ├── validator.rs        # Proxy 驗證邏輯
│   │   │   └── health_scorer.rs    # 健康度評分算法
│   │   │
│   │   ├── modules/                # 🧩 模組系統 ⭐⭐
│   │   │   ├── mod.rs
│   │   │   ├── loader.rs           # 動態模組載入器 ⭐
│   │   │   ├── interface.rs        # WebsiteModule Trait 定義
│   │   │   └── registry.rs         # 模組註冊表
│   │   │
│   │   ├── storage/                # 💾 JSON 儲存層
│   │   │   ├── mod.rs
│   │   │   ├── config.rs           # AppSettings 讀寫
│   │   │   ├── tasks.rs            # DownloadTask 持久化
│   │   │   └── history.rs          # DownloadHistory 管理
│   │   │
│   │   ├── utils/                  # 🛠️ 工具函式
│   │   │   ├── crypto.rs           # 憑證加密（keyring 封裝）
│   │   │   ├── clipboard.rs        # 剪貼簿監聽
│   │   │   └── platform.rs         # 平台特定處理
│   │   │
│   │   └── models/                 # 📊 資料結構定義
│   │       ├── download_task.rs
│   │       ├── proxy_info.rs
│   │       └── settings.rs
│   │
│   ├── Cargo.toml                  # Rust 依賴管理
│   ├── tauri.conf.json             # Tauri 應用程式配置
│   └── build.rs                    # 建構腳本
│
├── src/                            # ⚛️ React 前端
│   ├── main.tsx                    # React 進入點
│   ├── App.tsx                     # 主應用元件
│   │
│   ├── components/                 # 🧱 UI 元件（Material UI）
│   │   ├── DownloadList.tsx        # 下載列表元件
│   │   ├── DownloadItem.tsx        # 單一下載項元件
│   │   ├── ProgressBar.tsx         # 進度條元件
│   │   ├── ProxySettings.tsx       # Proxy 設定面板
│   │   └── SettingsDialog.tsx      # 設定對話框
│   │
│   ├── pages/                      # 📄 頁面元件
│   │   ├── MainPage.tsx            # 主頁面（下載列表）
│   │   ├── HistoryPage.tsx         # 歷史記錄頁
│   │   └── SettingsPage.tsx        # 設定頁面
│   │
│   ├── stores/                     # 🗃️ Jotai 狀態管理 ⭐
│   │   ├── downloadStore.ts        # 下載任務狀態 ⭐
│   │   ├── proxyStore.ts           # Proxy 狀態
│   │   └── settingsStore.ts        # 設定狀態
│   │
│   ├── hooks/                      # 🪝 自訂 React Hooks
│   │   ├── useDownloads.ts         # 下載相關 hooks
│   │   ├── useTauriEvents.ts       # Tauri 事件監聽
│   │   └── useClipboard.ts         # 剪貼簿監聽 hook
│   │
│   ├── locales/                    # 🌍 國際化 JSON
│   │   ├── zh-TW.json              # 繁體中文（預設）
│   │   ├── en.json                 # 英文
│   │   └── ja.json                 # 日文
│   │
│   ├── styles/                     # 🎨 全域樣式
│   │   └── theme.ts                # Material UI 主題配置
│   │
│   └── types/                      # 📝 TypeScript 類型定義
│       ├── download.ts
│       ├── proxy.ts
│       └── settings.ts
│
├── chrome-extension/               # 🔌 Chrome 擴充套件（選用）
│   ├── manifest.json               # 擴充套件清單
│   ├── background.js               # 背景腳本（偵測下載）
│   └── native-messaging/           # Native Messaging 配置
│
├── modules/                        # 🧩 預建模組範例（不在 src-tauri 內）
│   └── example-downloader/         # 範例網站支援模組
│       ├── Cargo.toml
│       └── src/lib.rs
│
├── tests/                          # 🧪 整合測試
│   ├── download_test.rs
│   ├── proxy_test.rs
│   └── e2e/                        # 端對端測試（Tauri WebDriver）
│
├── .specify/                       # 📋 Spec Kit 設定（已有）
├── specs/                          # 📋 功能規格（已有）
├── package.json                    # Node.js 依賴管理
├── vite.config.ts                  # Vite 建構配置
└── README.md                       # 專案說明
```

#### ⭐ 關鍵檔案標記說明

- **⭐⭐⭐** (最關鍵): 專案核心邏輯，實作優先順序最高
  - `src-tauri/src/download/engine.rs` - 下載引擎主邏輯
  - `src-tauri/src/proxy/pool.rs` - Proxy 池管理核心

- **⭐⭐** (重要): 關鍵功能模組
  - `src-tauri/src/modules/loader.rs` - 模組載入器
  - `src-tauri/src/commands/download.rs` - Tauri 命令處理

- **⭐** (基礎): 必要基礎設施
  - `src/stores/downloadStore.ts` - 前端狀態管理

### 📐 結構決策

**選擇**: Option 2 - Web Application 結構（Tauri 變體）

**理由**:
1. **前後端分離**: Rust 後端（`src-tauri/`）+ React 前端（`src/`）清晰分離
2. **Tauri 慣例**: 遵循 Tauri 官方專案結構建議
3. **模組獨立**: `modules/` 目錄獨立於主專案，方便社群貢獻
4. **測試分層**: 單元測試（Rust/React）+ 整合測試（`tests/`）+ E2E（`tests/e2e/`）

**與傳統 Web App 的差異**:
- 沒有 `backend/` 而是 `src-tauri/`（Tauri 特有）
- 沒有 API 伺服器，前後端透過 Tauri 命令通訊
- 編譯產物是桌面應用程式（`.exe`/`.app`/`.AppImage`）而非 Web 伺服器

---

## Complexity Tracking

> 🔍 **目的**: 僅在 Constitution Check 有違規時填寫，記錄違規理由和替代方案評估

**狀態**: ✅ **無違規項目**

本專案設計完全符合專案憲法的五大核心原則，無需複雜度理由說明。

---

## Phase 0: Outline & Research

> 📘 **完整研究內容**: 請參閱 [PLANNING_RECORD.md § Phase 0: 技術研究](./PLANNING_RECORD.md#phase-0-技術研究)

### 研究摘要

已完成 8 個關鍵技術領域的研究：

1. **✅ Headless Browser 整合**
   - 決策: `headless_chrome` crate
   - 理由: 成熟穩定、類 Puppeteer API、完整 CDP 支援

2. **✅ 動態模組載入**
   - 決策: `libloading` crate
   - 理由: 執行時動態載入、跨平台、native 效能

3. **✅ 多執行緒下載策略**
   - 決策: reqwest + tokio + Arc<Mutex<State>>
   - 設計: 預設 8 執行緒、10MB 分段、健康度評分

4. **✅ Proxy 驗證策略**
   - 決策: 主端點 `api.myip.com` + 備用端點
   - 流程: 批次 50 個、並行驗證、5 秒 timeout

5. **✅ 憑證加密儲存**
   - 決策: OS Keychain（`keyring` crate）
   - 對應: Windows Credential Manager / macOS Keychain / Linux Secret Service

6. **✅ 系統托盤整合**
   - 決策: Tauri 內建 SystemTray API
   - 功能: 進度顯示、右鍵選單、通知

7. **✅ 剪貼簿監聽**
   - 決策: tauri-plugin-clipboard + 200ms 輪詢
   - 驗證: URL 正則 + 自動提示

8. **✅ 國際化實作**
   - 決策: react-i18next + JSON
   - 語言: 繁中/英/日，自動偵測系統語言

### 研究產出

**文件**: `research.md`（待建立，內容已規劃於 PLANNING_RECORD.md）

**內容包含**:
- 每個技術領域的決策理由
- 替代方案評估
- 實作範例程式碼
- 最佳實踐建議

---

## Phase 1: Design & Contracts

> 📘 **完整設計內容**: 請參閱 [PLANNING_RECORD.md § Phase 1](./PLANNING_RECORD.md#phase-1-資料模型)

### 資料模型設計

**文件**: `data-model.md`（待建立，內容已規劃於 PLANNING_RECORD.md）

**定義的實體** (7 個):
1. **DownloadTask** - 下載任務（10 種狀態、狀態轉換規則）
2. **DownloadSegment** - 下載分段（完整性驗證算法）
3. **ProxyInfo** - Proxy 資訊（健康度評分公式）
4. **WebsiteModule** - 網站模組（Trait 定義）
5. **AppSettings** - 應用設定（5 大類設定）
6. **DownloadHistory** - 下載歷史（查詢/篩選方法）
7. **時序圖** - 下載生命週期 + Proxy 池管理流程（Mermaid 圖）

### API 合約設計

**文件**: `contracts/`（待建立，內容已規劃於 PLANNING_RECORD.md）

**檔案清單**:
- `tauri-commands.md` - 14 個 Tauri 命令 + 4 個前端事件
- `module-interface.md` - WebsiteModule Trait + 模組範例
- `storage-schema.md` - 4 個 JSON 檔案結構 + 跨平台路徑

### 開發者入門

**文件**: `quickstart.md`（待建立，內容已規劃於 PLANNING_RECORD.md）

**內容包含**:
- 環境設定（Rust, Node.js, Tauri CLI）
- 專案結構導覽
- **建立網站支援模組**（完整 4 步驟範例）
- 常見開發任務（新增命令、修改 UI、測試）
- 除錯技巧

---

## Next Steps

### 📝 待辦事項（本次規劃後）

1. **決定詳細文件建立範圍**
   - 選項 A: 立即建立所有詳細文件（`research.md`, `data-model.md`, `contracts/`, `quickstart.md`）
   - 選項 B: 僅參考 `PLANNING_RECORD.md`，直接進入 Phase 2 生成任務

2. **Phase 2: 實作任務生成**
   - 執行 `/speckit.tasks` 命令
   - 生成 `tasks.md`（依賴排序、複雜度估算）

3. **Phase 3: 開始實作**
   - 根據 `tasks.md` 順序執行
   - 優先實作關鍵檔案（標記 ⭐⭐⭐）

### 🎯 實作優先順序建議

**階段 1: 基礎架構** (Week 1-2)
- Tauri 專案初始化
- 目錄結構建立
- 依賴安裝與配置

**階段 2: 核心下載引擎** (Week 3-5) ⭐⭐⭐
- `download/engine.rs` - 多線程分段下載
- `download/resume.rs` - 斷點續傳
- `download/queue.rs` - 下載佇列

**階段 3: Proxy 池管理** (Week 6-7) ⭐⭐⭐
- `proxy/pool.rs` - Proxy 驗證與評分
- `proxy/validator.rs` - 批次並行驗證
- `proxy/health_scorer.rs` - 健康度算法

**階段 4: 模組系統** (Week 8-9) ⭐⭐
- `modules/loader.rs` - 動態載入
- `modules/interface.rs` - Trait 定義
- 範例模組（`modules/example-downloader/`）

**階段 5: Tauri 整合** (Week 10-11) ⭐
- `commands/*.rs` - 所有 Tauri 命令
- 前端事件發送
- 錯誤處理與驗證

**階段 6: React UI** (Week 12-14)
- Material UI 元件建立
- Jotai 狀態管理
- Tauri 事件監聽

**階段 7: 系統整合** (Week 15-16)
- 系統托盤
- 剪貼簿監聽
- 通知系統
- 多語言 (i18n)

**階段 8: 測試與優化** (Week 17-18)
- 單元測試（Rust + React）
- 整合測試
- 效能測試（驗證 Success Criteria）
- 跨平台測試（Windows/macOS/Linux）

---

## References

### 📚 內部文件

- [功能規格 (spec.md)](./spec.md) - 83 個功能需求、20 個成功標準
- [品質檢查清單 (checklists/requirements.md)](./checklists/requirements.md) - 規格品質驗證
- **[完整規劃記錄 (PLANNING_RECORD.md)](./PLANNING_RECORD.md)** - 所有技術研究、資料模型、API 合約詳細內容
- [專案憲法 (../../.specify/memory/constitution.md)](../../.specify/memory/constitution.md) - 五大核心原則

### 🔗 外部資源

**Tauri**:
- [Tauri 官方文件](https://tauri.app/v1/guides/)
- [Tauri 2.0 遷移指南](https://v2.tauri.app/start/migrate/from-tauri-1/)

**Rust**:
- [reqwest 文件](https://docs.rs/reqwest/)
- [tokio 文件](https://tokio.rs/)
- [libloading 文件](https://docs.rs/libloading/)

**React**:
- [Material UI 文件](https://mui.com/material-ui/)
- [Jotai 文件](https://jotai.org/)
- [react-i18next 文件](https://react.i18next.com/)

**參考專案**:
- [k2s-Downloaderm](https://github.com/lu8787ouo/k2s-Downloaderm) - Proxy 池管理參考

---

**文件版本**: 1.0.0
**建立日期**: 2025-01-27
**狀態**: ✅ Phase 0-1 規劃完成，等待 Phase 2 任務生成
**下一步**: 執行 `/speckit.tasks` 生成實作任務清單
