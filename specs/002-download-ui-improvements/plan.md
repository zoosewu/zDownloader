# Implementation Plan: Download UI Improvements

**Branch**: `002-download-ui-improvements` | **Date**: 2025-12-01 | **Spec**: [spec.md](./spec.md)
**Input**: Feature specification from `/specs/002-download-ui-improvements/spec.md`

**Note**: This template is filled in by the `/speckit.plan` command. See `.specify/templates/commands/plan.md` for the execution workflow.

## Summary

此功能重新設計下載管理介面以改善使用者體驗：

1. **主畫面簡化**：預設僅顯示下載列表，將新增下載功能移至獨立頁面，透過右上角按鈕存取
2. **智能剪貼簿監控**：自動偵測剪貼簿中的下載連結並自動導航至新增下載頁面並預填網址
3. **彈性路徑管理**：新增下載時提供路徑自訂功能，預設顯示使用者設定的下載資料夾
4. **模組化 URL 檢測**：支援兩種檢測模式 - 基於附檔名的直接檔案連結檢測和網站模組客製化檢測邏輯

技術方案採用 React Router 實現頁面切換、強化現有剪貼簿監控功能、並建立可擴充的 URL 檢測模組系統。

## Technical Context

**Language/Version**:
- Frontend: TypeScript 5.8.3 + React 19.1.0
- Backend: Rust 1.75+ (Tauri 2.x)

**Primary Dependencies**:
- Frontend: Material UI 6.0.0, Jotai 2.0.0, react-i18next 14.0.0, NEEDS CLARIFICATION (routing solution)
- Backend: Tauri 2.x, tauri-plugin-clipboard-manager, tauri-plugin-dialog

**Storage**: JSON files (settings, download history) + file system (download data)

**Testing**: NEEDS CLARIFICATION (frontend testing framework), cargo test (Rust)

**Target Platform**: Cross-platform desktop (Windows, Linux, macOS via Tauri)

**Project Type**: Desktop application (Tauri) with React frontend

**Performance Goals**:
- Clipboard detection within 500ms (SC-003)
- Complete add-download flow in <5 seconds (SC-005)
- UI responsiveness maintained during URL detection

**Constraints**:
- Must work across different OS clipboard APIs
- URL detection must be non-blocking to avoid UI freezing
- Must maintain backward compatibility with existing download task format

**Scale/Scope**:
- Single-user desktop application
- ~20 React components total (currently ~15 components)
- Add 2-3 new pages/views for navigation
- Support common file extensions (12+ types defined in spec assumptions)

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

### I. 模組化優先（Modularity First）

✅ **PASS** - URL 檢測模組化設計
- FR-012: 系統支援模組化的 URL 檢測機制
- FR-013: URL 檢測區分兩種模式，允許網站模組提供客製化邏輯
- 符合憲法要求：核心不包含特定網站硬編碼邏輯，透過模組系統擴充

### II. 效能與可靠性（Performance & Reliability）

✅ **PASS** - 效能目標明確
- SC-003: 剪貼簿監控偵測延遲在 500 毫秒內
- SC-005: 完整新增下載流程在 5 秒內完成
- 錯誤處理優雅：邊緣案例已識別（spec.md 第 78-84 行）

### III. 使用者體驗至上（User Experience First）

✅ **PASS** - 符合 UX 原則
- SC-001: 操作步驟減少至 2 步（自動跳轉並預填）
- SC-002: 90% 使用者首次嘗試成功率目標
- FR-008 to FR-011: 自動化剪貼簿監控和預填功能
- FR-014: 提供取消/返回功能
- 延續現有的 Material UI 設計語言

### IV. 跨平台一致性（Cross-Platform Consistency）

✅ **PASS** - 跨平台考量
- 約束條件明確指出：不同作業系統的剪貼簿 API 可能有差異，需跨平台相容性考量（spec.md 第 158 行）
- 使用 Tauri 的 clipboard-manager plugin 確保跨平台一致性

### V. 安全與隱私（Security & Privacy）

✅ **PASS** - 安全考量
- FR-015: 剪貼簿偵測到的連結必須經過驗證（確認為有效 URL 格式）
- FR-008: 基於使用者設定啟用剪貼簿監控功能（使用者可控）
- 無新增敏感資料處理需求

### 整體評估

✅ **所有憲法檢查通過** - 此功能符合 Zoo Downloader 的所有核心原則，無違規需要說明。

## Project Structure

### Documentation (this feature)

```text
specs/[###-feature]/
├── plan.md              # This file (/speckit.plan command output)
├── research.md          # Phase 0 output (/speckit.plan command)
├── data-model.md        # Phase 1 output (/speckit.plan command)
├── quickstart.md        # Phase 1 output (/speckit.plan command)
├── contracts/           # Phase 1 output (/speckit.plan command)
└── tasks.md             # Phase 2 output (/speckit.tasks command - NOT created by /speckit.plan)
```

### Source Code (repository root)

```text
src/                                    # React frontend
├── components/                         # UI components
│   ├── DownloadList.tsx               # Existing: download list display
│   ├── UrlInput.tsx                   # TO MODIFY: simplify for list page
│   └── AddDownloadForm.tsx            # NEW: dedicated add download component
├── pages/
│   ├── MainPage.tsx                   # TO MODIFY: remove inline UrlInput
│   ├── AddDownloadPage.tsx            # NEW: dedicated add download page
│   └── SettingsPage.tsx               # Existing
├── hooks/
│   ├── useDownloads.ts                # Existing: download operations
│   ├── useClipboard.ts                # NEW: clipboard monitoring logic
│   └── useNavigation.ts               # NEW: page navigation helpers
├── stores/
│   ├── downloadStore.ts               # Existing: download state
│   ├── navigationStore.ts             # NEW: navigation state (current page)
│   └── settingsStore.ts               # Existing: includes default download path
├── utils/
│   ├── urlDetector.ts                 # NEW: URL detection logic
│   └── moduleLoader.ts                # NEW: website module loader interface
├── types/
│   ├── download.ts                    # Existing
│   └── urlModule.ts                   # NEW: URL detection module types
└── App.tsx                            # TO MODIFY: add routing logic

src-tauri/src/                         # Rust backend
├── commands/
│   ├── download.rs                    # Existing: download operations
│   └── clipboard.rs                   # TO MODIFY: enhance clipboard monitoring
├── modules/
│   └── url_detector/                  # NEW: URL detection module interface
│       ├── mod.rs                     # Module trait definition
│       └── builtin.rs                 # Built-in file extension detector
└── utils/
    └── clipboard.rs                   # TO MODIFY: add URL validation
```

**Structure Decision**:
此專案為 Tauri 桌面應用程式，採用前後端分離結構。前端使用 React + TypeScript，後端使用 Rust。此功能主要涉及前端 UI 重構（頁面導航）、剪貼簿監控強化（前後端協作）、以及新增 URL 檢測模組系統（主要在 Rust 後端，前端提供介面調用）。

## Complexity Tracking

> **Fill ONLY if Constitution Check has violations that must be justified**

N/A - 所有憲法檢查通過，無違規需要說明。

---

## Post-Design Constitution Re-check

*Required after Phase 1 design completion*

### Re-evaluation Date: 2025-12-01

經過 Phase 0 (Research) 和 Phase 1 (Design) 完成後，重新評估設計是否符合憲法原則：

### I. 模組化優先（Modularity First）

✅ **PASS** - 設計完全符合模組化原則
- URL 檢測系統採用 Rust Trait-based 設計（data-model.md）
- `UrlDetector` trait 定義清晰，易於擴充
- 內建 `BuiltinFileDetector` 作為範例實作
- 第三方可實作 trait 新增網站支援，無需修改核心程式碼
- 模組註冊機制（`ModuleRegistry`）支援動態載入

**設計證據**:
- contracts/tauri-commands.md: `detect_url` command 使用 `ModuleRegistryState`
- data-model.md: `UrlDetectorModule` trait 定義完整

### II. 效能與可靠性（Performance & Reliability）

✅ **PASS** - 效能目標明確且可達成
- 剪貼簿輪詢間隔：400ms（低於 500ms 目標）
- URL 檢測效能基準：
  - 附檔名檢測：< 100ms
  - 模組客製化檢測：< 500ms
- 所有 Tauri commands 都有明確的效能要求（contracts/tauri-commands.md）
- 錯誤處理完善，包含驗證、權限檢查、優雅降級

**設計證據**:
- research.md: 輪詢間隔決策（400ms）
- contracts/tauri-commands.md: Performance Benchmarks 表格
- quickstart.md: Performance profiling 工具和方法

### III. 使用者體驗至上（User Experience First）

✅ **PASS** - UX 設計優先
- 自動化流程：剪貼簿自動偵測 → 自動導航 → 預填 URL → 2 步完成
- 錯誤訊息友善且可操作（contracts 中有完整錯誤處理定義）
- UI 簡潔：主畫面僅顯示下載列表，新增下載移至獨立頁面
- 提供取消/返回功能，導航直觀
- 路徑選擇提供預設值和瀏覽按鈕

**設計證據**:
- frontend-api.md: `AddDownloadForm` 包含完整的 UX 考量
- contracts/tauri-commands.md: 友善的錯誤訊息格式
- quickstart.md: 完整的使用者流程描述

### IV. 跨平台一致性（Cross-Platform Consistency）

✅ **PASS** - 跨平台相容性已考慮
- 使用 Tauri 的跨平台 APIs（clipboard-manager, dialog）
- 路徑驗證工具（`pathValidator`）處理不同平台的路徑分隔符
- URL 檢測邏輯與平台無關
- 所有檔案操作都透過 Tauri 的跨平台抽象

**設計證據**:
- research.md: 明確選擇 Tauri plugins 確保跨平台相容
- frontend-api.md: `pathValidator.getPathSeparator()` 處理平台差異
- contracts/tauri-commands.md: 使用標準化的錯誤處理

### V. 安全與隱私（Security & Privacy）

✅ **PASS** - 安全性充分考慮
- URL 驗證防止注入攻擊（FR-015）
- 路徑驗證防止目錄遍歷（contracts 中有明確要求）
- 檔案名稱清理防止特殊字元
- 剪貼簿內容大小限制（< 10KB）
- 權限檢查（寫入權限驗證）
- 無憑證處理需求，無新增安全風險

**設計證據**:
- contracts/tauri-commands.md: Security Checklist 完整
- frontend-api.md: `urlDetector` 包含驗證工具
- data-model.md: 所有實體都有驗證規則

### 整體評估（Post-Design）

✅ **所有憲法檢查再次通過**

**設計品質總結**:
1. **模組化程度**: 優秀 - Trait-based 模組系統可擴充性極佳
2. **效能設計**: 優秀 - 所有操作都有明確的效能基準和測試計畫
3. **UX 設計**: 優秀 - 自動化流程大幅減少使用者操作步驟
4. **跨平台設計**: 良好 - 使用 Tauri 標準 APIs，考慮平台差異
5. **安全設計**: 優秀 - 完整的安全檢查清單和驗證機制

**無新增技術債務**，設計簡潔且符合所有專案原則。

---

## Implementation Readiness

### Phase 2 Prerequisites ✅

以下所有項目已完成，可以進入 Phase 2（實作階段）：

- [x] **Research Complete** (research.md)
  - 所有 NEEDS CLARIFICATION 已解決
  - 技術決策已記錄
  - 替代方案已評估

- [x] **Data Model Defined** (data-model.md)
  - 所有實體已定義
  - 驗證規則已建立
  - 狀態轉換已記錄
  - 關係圖已繪製

- [x] **API Contracts Defined** (contracts/)
  - Tauri commands 完整定義（tauri-commands.md）
  - Frontend APIs 完整定義（frontend-api.md）
  - 錯誤處理規範
  - 效能要求明確

- [x] **Quick Start Guide Created** (quickstart.md)
  - 開發流程已規劃
  - 檔案結構已定義
  - 常見任務已記錄
  - 除錯技巧已提供

- [x] **Agent Context Updated** (CLAUDE.md)
  - 新增技術已記錄
  - 資料庫資訊已更新

- [x] **Constitution Check Passed** (再次驗證)
  - 所有原則通過
  - 無違規需要說明
  - 設計品質優秀

### Ready for Phase 2 🚀

此功能已完成所有規劃階段工作，可以進入實作階段。建議使用 `/speckit.tasks` 指令生成詳細的實作任務清單（tasks.md）。

---

## Summary

### Planning Outputs

| 文件 | 狀態 | 位置 |
|------|------|------|
| Implementation Plan | ✅ Complete | plan.md |
| Research Document | ✅ Complete | research.md |
| Data Model | ✅ Complete | data-model.md |
| API Contracts (Backend) | ✅ Complete | contracts/tauri-commands.md |
| API Contracts (Frontend) | ✅ Complete | contracts/frontend-api.md |
| Quick Start Guide | ✅ Complete | quickstart.md |

### Key Technical Decisions

1. **路由方案**: 狀態管理導航（無需 React Router）
2. **測試框架**: Vitest + React Testing Library
3. **剪貼簿監控**: 400ms 輪詢間隔
4. **URL 檢測**: Trait-based 模組系統

### Next Command

```bash
/speckit.tasks
```

此命令將生成 `tasks.md`，將設計轉換為可執行的開發任務清單。

---

**Planning Phase Complete** ✅
**Branch**: `002-download-ui-improvements`
**Ready for Implementation**: Yes
**Estimated Implementation Time**: 14-19 hours
