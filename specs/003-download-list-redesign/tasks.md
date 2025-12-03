# Tasks: 下載列表 UI 重新設計

**Input**: Design documents from `/specs/003-download-list-redesign/`
**Prerequisites**: plan.md, spec.md, research.md, data-model.md, contracts/

**Tests**: 本功能不包含測試任務，專注於 UI 實施。可以在實施完成後另外添加測試。

**Organization**: Tasks are grouped by user story to enable independent implementation and testing of each story.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependencies)
- **[Story]**: Which user story this task belongs to (e.g., US1, US2, US3)
- Include exact file paths in descriptions

## Path Conventions

- **Tauri Desktop App**: `src/` (frontend), `src-tauri/src/` (backend - not modified in this feature)
- All new components in `src/components/`
- State management in `src/stores/`
- Utilities in `src/utils/`
- Hooks in `src/hooks/`

## Phase 1: Setup (Shared Infrastructure)

**Purpose**: 創建基礎工具函數和狀態管理，為所有 User Stories 服務

- [x] T001 [P] Create downloadCalculations.ts utility with calculateRemainingTime and formatRemainingTime functions in src/utils/downloadCalculations.ts
- [x] T002 [P] Create formatRelativeTime function for friendly time display in src/utils/formatters.ts
- [x] T003 [P] Add selectedTaskIdAtom and selectedTaskAtom to downloadStore in src/stores/downloadStore.ts
- [x] T004 [P] Create useSelectedDownload hook in src/hooks/useSelectedDownload.ts
- [x] T005 [P] Add i18n translation keys for new UI elements in src/locales/zh-TW.json

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: 無需 Foundational Phase - 所有基礎設施已在 Setup 中完成

**⚠️ CRITICAL**: Setup (Phase 1) 完成後，User Stories 可以開始並行實施

---

## Phase 3: User Story 1 - 表格視圖查看所有下載項目 (Priority: P1) 🎯 MVP

**Goal**: 將現有的卡片列表改為表格視圖，顯示所有下載項目的關鍵資訊

**Independent Test**: 打開下載列表頁面，驗證所有下載項目以表格形式顯示，包含 7 個欄位（檔名、大小、進度、速度、狀態、建立時間、剩餘時間），實時更新正常工作

### Implementation for User Story 1

- [x] T006 [P] [US1] Create TableDisplayData interface and formatting logic in src/utils/downloadCalculations.ts
- [x] T007 [P] [US1] Create DownloadTableRow component with React.memo optimization in src/components/DownloadTableRow.tsx
- [x] T008 [US1] Create DownloadTable component using MUI Table with stickyHeader in src/components/DownloadTable.tsx
- [x] T009 [US1] Update DownloadList component to render DownloadTable instead of card list in src/components/DownloadList.tsx
- [x] T010 [US1] Implement real-time update logic using Jotai atomFamily pattern in src/stores/downloadStore.ts
- [x] T011 [US1] Add empty state handling with proper i18n message in src/components/DownloadTable.tsx
- [x] T012 [US1] Test with multiple download tasks (different statuses: downloading, completed, failed, paused) and verify real-time updates

**Checkpoint**: At this point, User Story 1 should be fully functional - table view displays all downloads with real-time updates

---

## Phase 4: User Story 2 - 選擇項目並查看詳細信息 (Priority: P2)

**Goal**: 實現點擊選擇功能和下方詳細信息面板，使用 Tab 切換不同類型的資訊

**Independent Test**: 點擊表格中的任何項目，驗證該項目被高亮且詳細面板顯示正確資訊，切換 Tab 正常工作，選中項目被刪除時面板正確清空

### Implementation for User Story 2

- [x] T013 [P] [US2] Create BasicInfoTab component displaying URL, filepath, timestamps in src/components/BasicInfoTab.tsx
- [x] T014 [P] [US2] Create ErrorInfoTab component displaying error message (or "無錯誤訊息") in src/components/ErrorInfoTab.tsx
- [x] T015 [P] [US2] Create AdvancedInfoTab component displaying retry info and scheduled time in src/components/AdvancedInfoTab.tsx
- [x] T016 [US2] Create DownloadDetailPanel component with MUI Tabs integration in src/components/DownloadDetailPanel.tsx
- [x] T017 [US2] Update DownloadTableRow to handle onClick and pass selectedTaskId to parent in src/components/DownloadTableRow.tsx
- [x] T018 [US2] Integrate DownloadDetailPanel into DownloadList below the table in src/components/DownloadList.tsx
- [x] T019 [US2] Implement selection state management and auto-clear when selected task is deleted in src/hooks/useSelectedDownload.ts
- [x] T020 [US2] Add proper spacing and layout between table and detail panel in src/components/DownloadList.tsx
- [x] T021 [US2] Test selection: click different items, verify highlighting, verify detail panel updates, delete selected item and verify panel clears

**Checkpoint**: At this point, User Stories 1 AND 2 should both work - table view with selection and detail panel with tabs

---

## Phase 5: User Story 3 - 分段視覺化 (Priority: P3)

**Goal**: 在詳細面板的「分段視覺化」Tab 中以彩色方格顯示分段狀態，支持懸停顯示詳細信息

**Independent Test**: 選擇一個有多個分段的下載項目，切換到分段視覺化 Tab，驗證方格顏色正確（灰/藍/綠/紅），懸停顯示詳細信息，大量分段時性能流暢

### Implementation for User Story 3

- [x] T022 [P] [US3] Create SegmentVisualizationData interface with color mapping logic in src/utils/downloadCalculations.ts
- [x] T023 [P] [US3] Create SegmentGridItem component with MUI Tooltip for hover info in src/components/SegmentGridItem.tsx
- [x] T024 [US3] Create SegmentVisualization component using CSS Grid layout (12px fixed size squares) in src/components/SegmentVisualization.tsx
- [x] T025 [US3] Add virtualization support for large segment counts (> 500) using react-window in src/components/SegmentVisualization.tsx
- [x] T026 [US3] Create tooltip content component for segment details (ID, size range, progress, status, retry count, proxy) in src/components/SegmentGridItem.tsx
- [x] T027 [US3] Integrate SegmentVisualization into DownloadDetailPanel as a Tab in src/components/DownloadDetailPanel.tsx
- [x] T028 [US3] Add hover effects (scale transform) and proper spacing (2px gap) to segment grid in src/components/SegmentGridItem.tsx
- [x] T029 [US3] Test with various segment counts: 10, 100, 500, 1000 segments, verify colors, hover tooltips, and performance

**Checkpoint**: All user stories should now be independently functional - complete table view with selection, detail panel with all tabs including segment visualization

---

## Phase 6: Polish & Cross-Cutting Concerns

**Purpose**: 改進和優化整體功能

- [x] T030 [P] Add comprehensive i18n translations for all new UI text in public/locales/en/translation.json
- [x] T031 [P] Add comprehensive i18n translations for all new UI text in public/locales/ja/translation.json
- [x] T032 [P] Verify all new components follow Material-UI theme (light/dark mode support) in all components
- [x] T033 [P] Add proper TypeScript types and interfaces documentation in src/types/download.ts
- [x] T034 Performance testing with React DevTools Profiler for 100+ downloads and 1000 segments
- [x] T035 Accessibility review: keyboard navigation, screen reader support, ARIA labels
- [x] T036 [P] Update CLAUDE.md with implementation notes and component documentation
- [x] T037 [P] Clean up unused DownloadItem component code or mark for deprecation in src/components/DownloadItem.tsx
- [x] T038 Manual E2E testing: create downloads, select items, delete items, test all edge cases
- [x] T039 Run quickstart.md validation and update if needed

---

## Dependencies & Execution Order

### Phase Dependencies

- **Setup (Phase 1)**: No dependencies - can start immediately
- **Foundational (Phase 2)**: Not needed for this feature
- **User Stories (Phase 3-5)**: All depend on Setup (Phase 1) completion
  - User Story 1 can start after Setup
  - User Story 2 depends on User Story 1 (needs table component)
  - User Story 3 depends on User Story 2 (needs detail panel component)
- **Polish (Phase 6)**: Depends on all user stories being complete

### User Story Dependencies

- **User Story 1 (P1)**: Depends on Setup (Phase 1) - No dependencies on other stories
- **User Story 2 (P2)**: Depends on User Story 1 completion (needs DownloadTable component to add selection)
- **User Story 3 (P3)**: Depends on User Story 2 completion (needs DownloadDetailPanel component to add segment tab)

⚠️ **Note**: Unlike typical projects, these user stories have sequential dependencies due to UI layering:
- US2 builds on top of US1 (adds selection to table)
- US3 builds on top of US2 (adds tab to detail panel)

### Within Each User Story

#### User Story 1
1. Format utilities and table row component can be created in parallel (T006, T007)
2. Then create DownloadTable component (T008)
3. Then integrate into DownloadList (T009)
4. Then add real-time updates (T010)
5. Finally add empty state and test (T011, T012)

#### User Story 2
1. All three tab components can be created in parallel (T013, T014, T015)
2. Then create DownloadDetailPanel (T016)
3. Then update DownloadTableRow for selection (T017)
4. Then integrate panel into DownloadList (T018)
5. Then implement selection management (T019)
6. Finally add layout and test (T020, T021)

#### User Story 3
1. Data interface and grid item component can be created in parallel (T022, T023)
2. Then create SegmentVisualization component (T024)
3. Then add virtualization support (T025)
4. Then enhance tooltip content (T026)
5. Then integrate into DetailPanel (T027)
6. Then add styling (T028)
7. Finally test performance (T029)

### Parallel Opportunities

- **Setup Phase**: All 5 tasks (T001-T005) can run in parallel
- **User Story 1**: T006 and T007 can run in parallel
- **User Story 2**: T013, T014, T015 can run in parallel (all three tab components)
- **User Story 3**: T022 and T023 can run in parallel
- **Polish Phase**: Most tasks (T030-T033, T036-T037) can run in parallel

---

## Parallel Example: User Story 1

```bash
# Launch utility functions and table row component together:
Task: "Create TableDisplayData interface and formatting logic in src/utils/downloadCalculations.ts"
Task: "Create DownloadTableRow component with React.memo optimization in src/components/DownloadTableRow.tsx"

# After both complete, proceed sequentially with table creation and integration
```

## Parallel Example: User Story 2

```bash
# Launch all three tab components together:
Task: "Create BasicInfoTab component displaying URL, filepath, timestamps in src/components/BasicInfoTab.tsx"
Task: "Create ErrorInfoTab component displaying error message in src/components/ErrorInfoTab.tsx"
Task: "Create AdvancedInfoTab component displaying retry info in src/components/AdvancedInfoTab.tsx"

# After all tabs complete, proceed with panel creation and integration
```

## Parallel Example: User Story 3

```bash
# Launch data interface and grid item component together:
Task: "Create SegmentVisualizationData interface with color mapping logic in src/utils/downloadCalculations.ts"
Task: "Create SegmentGridItem component with MUI Tooltip for hover info in src/components/SegmentGridItem.tsx"

# After both complete, proceed with visualization component creation
```

---

## Implementation Strategy

### MVP First (User Story 1 Only)

1. Complete Phase 1: Setup (5 tasks, ~1 hour)
2. Complete Phase 3: User Story 1 (7 tasks, ~3-4 hours)
3. **STOP and VALIDATE**: Test table view independently
4. Deploy/demo if ready

**Benefits**:
- Users can see all downloads in a clean table format
- Better information architecture than card list
- Foundation for future enhancements

### Incremental Delivery

1. Complete Setup → Foundation ready (~1 hour)
2. Add User Story 1 → Test independently → Deploy/Demo (MVP!) (~3-4 hours)
3. Add User Story 2 → Test independently → Deploy/Demo (~4-5 hours)
4. Add User Story 3 → Test independently → Deploy/Demo (~3-4 hours)
5. Add Polish → Final release (~2-3 hours)

**Total Estimated Time**: 13-17 hours for complete feature

### Sequential Implementation (Recommended)

Due to UI layering dependencies, implement sequentially:

1. Phase 1 (Setup) → Phase 3 (US1) → Phase 4 (US2) → Phase 5 (US3) → Phase 6 (Polish)
2. Test each story independently before moving to next
3. Commit after completing each user story phase

---

## Task Summary

- **Total Tasks**: 39
- **Setup Tasks**: 5 (T001-T005)
- **User Story 1 Tasks**: 7 (T006-T012) - MVP
- **User Story 2 Tasks**: 9 (T013-T021)
- **User Story 3 Tasks**: 8 (T022-T029)
- **Polish Tasks**: 10 (T030-T039)

### Tasks by Parallelizability

- **Parallel Tasks**: 16 (marked with [P])
- **Sequential Tasks**: 23
- **Parallel Opportunities**: 4 clusters identified

### MVP Scope

**MVP = Setup + User Story 1** (12 tasks total)
- Delivers table view with real-time updates
- Independently testable and deployable
- Estimated time: 4-5 hours

---

## Notes

- [P] tasks = different files, no dependencies
- [Story] label maps task to specific user story for traceability
- User Story 1 is the MVP and can be deployed independently
- User Stories 2 and 3 have dependencies on previous stories (UI layering)
- Commit after completing each user story phase
- Stop at any checkpoint to validate story independently
- All components follow existing Material-UI and React patterns
- No backend changes required - purely frontend UI redesign
