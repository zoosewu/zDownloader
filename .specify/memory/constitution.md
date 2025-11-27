<!--
Sync Impact Report:
- Version change: (new constitution) → 1.0.0
- Initial constitution creation for Zoo Downloader
- Principles defined:
  1. 模組化優先 (Modularity First)
  2. 效能與可靠性 (Performance & Reliability)
  3. 使用者體驗至上 (User Experience First)
  4. 跨平台一致性 (Cross-Platform Consistency)
  5. 安全與隱私 (Security & Privacy)
- Templates status:
  ⚠ plan-template.md - pending initial creation
  ⚠ spec-template.md - pending initial creation
  ⚠ tasks-template.md - pending initial creation
- Follow-up: Create initial spec using /speckit.specify
-->

# Zoo Downloader Constitution

## Core Principles

### I. 模組化優先（Modularity First）

**核心規則（NON-NEGOTIABLE）：**
- 所有擴充功能（網站支援模組、Proxy List 來源模組）**必須**透過模組系統實作
- 模組**必須**獨立、可測試、支援執行時動態載入
- 模組介面**必須**清晰定義且維持向後相容性
- 核心下載器**不得**包含特定網站的硬編碼邏輯

**理由：**
專案設計目標是可擴充的下載器。模組化確保未來可以輕鬆新增對更多網站和 Proxy 來源的支援，而不需修改核心程式碼。這也使社群貢獻者可以獨立開發和分享模組。

### II. 效能與可靠性（Performance & Reliability）

**核心規則（NON-NEGOTIABLE）：**
- 多線程分段下載**必須**執行緒安全且高效
- 斷點續傳**必須**可靠，支援智能檢查（容差 ±1 byte）
- Proxy 失敗時**必須**自動切換到下一個可用 Proxy
- 錯誤處理**必須**優雅降級，避免資料損失
- 分段大小固定為 10MB（可由使用者調整）

**理由：**
這是一個專為大檔案設計的下載器。效能和可靠性直接影響使用者體驗。參考 k2s-Downloaderm 的設計，動態 Proxy 池管理和智能斷點續傳是保證下載成功率的關鍵。

### III. 使用者體驗至上（User Experience First）

**核心規則（NON-NEGOTIABLE）：**
- UI **必須**簡潔、扁平化、響應迅速（Material UI 設計語言）
- 錯誤訊息**必須**友善且提供可操作的建議
- **必須**支援多語言：繁體中文、英文、日文（使用 JSON 格式管理）
- 自動化功能：
  - 剪貼簿監聽**必須**自動偵測 URL 並提示使用者
  - 下載失敗**必須**自動重試
  - 應用程式**必須**支援自動更新
- 檔案名稱衝突**必須**自動重命名（加編號）
- **必須**支援淺色/深色模式切換

**理由：**
個人使用工具的成功取決於使用的流暢度。自動化減少手動操作，多語言支援擴大使用者群，友善的錯誤處理降低學習曲線。

### IV. 跨平台一致性（Cross-Platform Consistency）

**核心規則（NON-NEGOTIABLE）：**
- Windows、Linux、macOS 的核心功能**必須**行為一致
- 系統整合（托盤、通知、瀏覽器擴充）**應該**針對各平台優化
- 設定檔**必須**可跨平台移植（JSON 格式）
- UI 佈局**應該**適應不同螢幕尺寸和 DPI

**理由：**
Tauri 提供跨平台能力，但需要謹慎處理平台差異。一致的核心行為確保文檔和支援的可維護性，而平台特定的優化提升本地化體驗。

### V. 安全與隱私（Security & Privacy）

**核心規則（NON-NEGOTIABLE）：**
- 使用者憑證（帳號密碼）**必須**加密儲存
- HTTPS 連線**必須**驗證憑證
- Proxy 使用**必須**透明且使用者可控
- **不得**包含任何遙測或使用者追蹤功能
- **必須**支援 Proxy 驗證和效能評估

**理由：**
下載器可能處理敏感檔案和憑證。安全性不是可選功能。Proxy 的透明使用確保使用者理解其網路流量走向。

## 技術標準（Technical Standards）

### 技術棧規範

**前端：**
- React 18+
- Material UI（UI 框架）
- Jotai（狀態管理）
- react-i18next（國際化）

**後端：**
- Tauri 2.x（Rust 核心）
- reqwest + tokio（HTTP 和非同步）
- 需要時整合 headless browser（用於網站模組的 JavaScript 執行）

**資料儲存：**
- JSON 檔案（設定、歷史、Proxy 快取）
- 檔案格式**必須**人類可讀且易於除錯

**程式碼規範：**
- 程式碼和註解**必須**使用英文
- UI 文字**必須**透過 i18n 系統管理，支援多語言
- Rust 程式碼**必須**通過 `cargo clippy` 檢查
- TypeScript 程式碼**必須**啟用嚴格模式

### 借鏡設計模式（k2s-Downloaderm）

基於對 k2s-Downloaderm 專案的分析，以下設計模式**必須**遵循：

**Proxy 管理：**
- 批次並行驗證（每批 50 個）
- Proxy 快取機制（`proxies.txt`）
- 動態 Proxy 池管理（為每個分段分配不同 Proxy）
- 執行緒安全的 Proxy 鎖機制
- 優雅降級（無可用 Proxy 時使用直接連線）

**網站模組：**
- CAPTCHA 雙層回調設計（預設 UI + 可自訂）
- 禮貌性延遲機制（遵守 API 速率限制）
- Proxy 輪替應對速率限制

**下載器核心：**
- Range Request 分段下載
- 智能斷點續傳（容差 ±1 byte）
- 多層進度追蹤（位元組級 + 分段級）

## 開發流程（Development Workflow）

### Spec Kit 工作流程

**規範：**
- 新功能開發**必須**先使用 `/speckit.specify` 建立規格
- 重大架構變更**必須**使用 `/speckit.plan` 建立設計文件
- 實作任務**必須**使用 `/speckit.tasks` 生成任務清單
- 規格變更**必須**觸發相關文件的更新

### 品質標準

**測試要求：**
- 核心下載邏輯**必須**有單元測試
- 模組介面**必須**有整合測試
- 關鍵使用者流程**應該**有端對端測試

**程式碼審查：**
- 所有變更**必須**經過審查
- 安全相關變更**必須**經過額外安全審查

## Governance

### 憲法修訂程序

- 憲法修訂**必須**記錄在 Sync Impact Report 中
- 版本號遵循語義化版本：
  - **MAJOR**：不向後相容的原則移除或重新定義
  - **MINOR**：新增原則或重大擴充
  - **PATCH**：澄清、措辭修正、非語義調整
- 修訂**必須**更新 `LAST_AMENDED_DATE`

### 合規性

- 所有程式碼審查**必須**驗證是否符合憲法原則
- 違反核心原則的程式碼**不得**合併
- 複雜度增加**必須**提供明確理由

### 執行指引

- 執行時開發指引遵循本憲法
- 模組開發者**應該**參考 `.specify/templates/` 中的模組模板
- 遇到憲法衝突時，本憲法優先於其他慣例

**Version**: 1.0.0 | **Ratified**: 2025-01-27 | **Last Amended**: 2025-01-27
