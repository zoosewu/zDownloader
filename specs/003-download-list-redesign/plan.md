# Implementation Plan: 下載列表 UI 重新設計

**Branch**: `003-download-list-redesign` | **Date**: 2025-12-03 | **Spec**: [spec.md](./spec.md)
**Input**: Feature specification from `/specs/003-download-list-redesign/spec.md`

**Note**: This template is filled in by the `/speckit.plan` command. See `.specify/templates/commands/plan.md` for the execution workflow.

## Summary

將現有的卡片式下載列表重新設計為表格視圖，並在下方添加詳細信息面板。主要變更包括：

1. **表格視圖**：將每個下載項目顯示為表格的一行，包含檔名、大小、進度、速度、狀態、建立時間和剩餘時間
2. **選擇與高亮**：允許用戶點擊任何下載項目進行選擇，選中的項目會被高亮顯示
3. **詳細信息面板**：在表格下方顯示選中項目的完整資訊，使用 Tab 切換不同視圖（基本信息、分段視覺化、錯誤信息、進階資訊）
4. **分段視覺化重新設計**：將現有的可折疊列表改為固定大小的彩色方格，支持懸停顯示詳細信息

技術方法：使用 Material-UI 的 Table 組件替換現有的 Card 列表，使用 Jotai 管理選中狀態，Tab 組件用於詳細面板，Grid/Box 組件渲染分段方格。

## Technical Context

**Language/Version**: TypeScript 5.8.3 (frontend), Rust 1.75+ (backend - Tauri)
**Primary Dependencies**:
- Frontend: React 19.1.0, Material-UI 6.0.0, Jotai 2.0.0, react-i18next 14.0.0
- Backend: Tauri 2.x, tokio, reqwest, serde
**Storage**: JSON files (download history, settings) + in-memory state (Jotai atoms)
**Testing**: Vitest 4.0.15 (frontend unit tests), cargo test (backend unit tests)
**Target Platform**: Desktop (Windows, Linux, macOS) via Tauri
**Project Type**: Desktop application (Tauri frontend + Rust backend)
**Performance Goals**:
- Table rendering < 1 second for all downloads
- Item selection and highlight < 200ms
- Segment visualization render < 500ms for 100 segments
- Tooltip display < 100ms on hover
- UI update latency < 1 second for active downloads
**Constraints**:
- Must support up to 1000 download items smoothly
- Segment visualization must handle up to 1000 segments without lag
- Must maintain responsive UI during frequent updates (download progress)
- Time calculations must be accurate within 10% for stable speeds
**Scale/Scope**: Single desktop application with ~20 React components, focus on 2 main components (DownloadTable, DownloadDetailPanel)

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

### Principle I: 模組化優先（Modularity First）
✅ **PASS** - 此功能為 UI 層面的重新設計，不涉及核心下載器邏輯，不違反模組化原則。新組件（DownloadTable, DownloadDetailPanel）將作為獨立可測試的 React 組件開發。

### Principle II: 效能與可靠性（Performance & Reliability）
✅ **PASS** - UI 重新設計不會影響核心下載邏輯的執行緒安全性和斷點續傳機制。分段視覺化的性能目標明確（100個分段 < 500ms），符合效能要求。

### Principle III: 使用者體驗至上（User Experience First）
✅ **PASS** - 此功能的核心目標就是改善使用者體驗。表格視圖提供更清晰的資訊呈現，詳細面板讓用戶可以深入了解下載狀態，分段視覺化提供直觀的進度反饋。所有文字將通過 react-i18next 支援多語言。

### Principle IV: 跨平台一致性（Cross-Platform Consistency）
✅ **PASS** - 使用 Material-UI 確保跨平台 UI 一致性。Material-UI 的 Table 組件在所有桌面平台上行為一致。

### Principle V: 安全與隱私（Security & Privacy）
✅ **PASS** - UI 重新設計不涉及安全或隱私相關的變更，不處理用戶憑證或網路連線。

### Gate Evaluation
✅ **所有憲法原則檢查通過** - 無違規項目，可以進入 Phase 0 研究階段。

## Project Structure

### Documentation (this feature)

```text
specs/003-download-list-redesign/
├── plan.md              # This file (/speckit.plan command output)
├── research.md          # Phase 0 output (/speckit.plan command)
├── data-model.md        # Phase 1 output (/speckit.plan command)
├── quickstart.md        # Phase 1 output (/speckit.plan command)
├── contracts/           # Phase 1 output (/speckit.plan command)
└── tasks.md             # Phase 2 output (/speckit.tasks command - NOT created by /speckit.plan)
```

### Source Code (repository root)

```text
src/
├── components/
│   ├── DownloadList.tsx                    # [MODIFY] 將改為渲染表格而非卡片列表
│   ├── DownloadItem.tsx                    # [REFACTOR] 可能重用部分邏輯
│   ├── DownloadTable.tsx                   # [NEW] 表格視圖組件
│   ├── DownloadTableRow.tsx                # [NEW] 表格行組件
│   ├── DownloadDetailPanel.tsx             # [NEW] 詳細信息面板組件
│   ├── SegmentVisualization.tsx            # [NEW] 分段視覺化組件（方格視圖）
│   ├── SegmentGridItem.tsx                 # [NEW] 單個分段方格組件
│   ├── BasicInfoTab.tsx                    # [NEW] 基本信息 Tab
│   ├── ErrorInfoTab.tsx                    # [NEW] 錯誤信息 Tab
│   └── AdvancedInfoTab.tsx                 # [NEW] 進階資訊 Tab
├── stores/
│   └── downloadStore.ts                    # [MODIFY] 添加選中項目的狀態管理
├── hooks/
│   ├── useDownloads.ts                     # [KEEP] 保持不變
│   ├── useSelectedDownload.ts              # [NEW] 管理選中項目的 hook
│   └── useTimeEstimation.ts                # [NEW] 剩餘時間計算 hook
├── utils/
│   ├── formatters.ts                       # [MODIFY] 添加時間格式化函數
│   └── downloadCalculations.ts            # [NEW] 下載相關計算（剩餘時間等）
└── types/
    └── download.ts                         # [KEEP] 已有 DownloadTask 和 DownloadSegment 類型

src/__tests__/
├── components/
│   ├── DownloadTable.test.tsx              # [NEW] 表格組件測試
│   ├── DownloadDetailPanel.test.tsx        # [NEW] 詳細面板測試
│   └── SegmentVisualization.test.tsx       # [NEW] 分段視覺化測試
└── utils/
    └── downloadCalculations.test.ts        # [NEW] 計算函數測試
```

**Structure Decision**: 採用現有的 Tauri 桌面應用結構。所有新組件放在 `src/components/` 下，遵循現有的組件命名和組織慣例。使用 Jotai atoms 管理新的 UI 狀態（選中項目）。測試文件放在 `src/__tests__/` 對應的目錄結構中。

## Complexity Tracking

無需填寫 - 所有憲法檢查均通過，無違規項目需要辯護。

## Post-Design Constitution Re-check

*Re-evaluated after Phase 1 design completion*

### Principle I: 模組化優先（Modularity First）
✅ **PASS** - 設計保持模組化：8 個新組件都是獨立、可測試的 React 組件。沒有硬編碼的網站特定邏輯。

### Principle II: 效能與可靠性（Performance & Reliability）
✅ **PASS** - 設計符合性能目標：
- 使用 React.memo 避免不必要的重新渲染
- 實施 EMA 算法確保時間估算準確
- 大量分段時使用虛擬化（react-window）
- 所有性能指標都有明確的實施方案

### Principle III: 使用者體驗至上（User Experience First）
✅ **PASS** - 設計優先考慮 UX：
- 表格視圖提供清晰的資訊架構
- 實時更新保持在 < 1s 延遲
- 所有文字通過 react-i18next 支援多語言
- 錯誤情況有明確的處理（空狀態、速度為 0 等）

### Principle IV: 跨平台一致性（Cross-Platform Consistency）
✅ **PASS** - 使用 Material-UI 確保跨平台一致性，所有組件在 Windows/Linux/macOS 上行為相同。

### Principle V: 安全與隱私（Security & Privacy）
✅ **PASS** - 純前端 UI 變更，不涉及安全或隱私問題。

### Final Gate Evaluation
✅ **所有憲法原則檢查通過（Phase 1 後）** - 設計符合所有憲法要求，可以進入實施階段。
