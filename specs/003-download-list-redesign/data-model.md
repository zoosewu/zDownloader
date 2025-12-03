# Data Model: 下載列表 UI 重新設計

**Feature**: 003-download-list-redesign
**Date**: 2025-12-03
**Phase**: Phase 1 - Design & Contracts

## Overview

此功能主要為 UI 層面的重新設計，不涉及新的資料結構。使用現有的 `DownloadTask` 和 `DownloadSegment` 模型，並添加新的前端狀態來管理選中項目。

## Existing Data Models

### DownloadTask

**Purpose**: 代表一個完整的下載任務

**Source**: `src/types/download.ts` (frontend) + `src-tauri/src/models/download_task.rs` (backend)

**Key Fields**:
- `id: string (UUID)` - 唯一識別碼
- `url: string` - 下載 URL
- `filename: string` - 檔案名稱
- `filepath: string` - 儲存路徑
- `filesize: number (u64)` - 檔案大小（位元組）
- `status: DownloadStatus` - 下載狀態（Pending, Downloading, Paused, Completed, Failed）
- `progress: number (f64)` - 下載進度（0-100）
- `currentSpeed: number (u64)` - 當前速度（位元組/秒）
- `segments: DownloadSegment[]` - 分段資料
- `createdAt: string (DateTime)` - 建立時間（ISO 8601 格式）
- `startedAt: string | null (Option<DateTime>)` - 開始時間
- `completedAt: string | null (Option<DateTime>)` - 完成時間
- `errorMessage: string | null (Option<String>)` - 錯誤訊息
- `scheduledStart: string | null (Option<DateTime>)` - 排程開始時間
- `retryCount: number (u32)` - 重試次數
- `maxRetries: number | null (Option<u32>)` - 最大重試次數
- `lastRetryAt: string | null (Option<DateTime>)` - 上次重試時間

**Relationships**:
- `1:N` with `DownloadSegment` - 一個下載任務包含多個分段

**Validation Rules**:
- `filesize` 必須 >= 0
- `progress` 必須在 0-100 之間
- `currentSpeed` 必須 >= 0
- `retryCount` 必須 <= `maxRetries` (如果 maxRetries 不為 null)

**State Transitions**:
```
Pending → Downloading → Completed
   ↓          ↓
   ↓       Paused → Downloading
   ↓          ↓
   ↓       Failed → (via retry) → Pending/Downloading
   ↓
Scheduled (有 scheduledStart) → Pending (時間到達)
```

### DownloadSegment

**Purpose**: 代表下載任務的一個分段

**Source**: `src/types/download.ts` (frontend) + `src-tauri/src/models/download_segment.rs` (backend)

**Key Fields**:
- `id: string (UUID)` - 唯一識別碼
- `segmentId: number (usize)` - 分段編號（從 0 開始）
- `startByte: number (u64)` - 起始位元組
- `endByte: number (u64)` - 結束位元組
- `downloadedBytes: number (u64)` - 已下載位元組數
- `status: SegmentStatus` - 分段狀態（Pending, Downloading, Completed, Failed）
- `retryCount: number (u32)` - 重試次數
- `proxy: string | null (Option<String>)` - 使用的代理（如果有）

**Relationships**:
- `N:1` with `DownloadTask` - 多個分段屬於一個下載任務

**Validation Rules**:
- `startByte` 必須 <= `endByte`
- `downloadedBytes` 必須 <= (`endByte` - `startByte` + 1)
- `segmentId` 在同一個任務中必須唯一

**Calculated Fields**:
- `segmentSize = endByte - startByte + 1` - 分段大小
- `segmentProgress = (downloadedBytes / segmentSize) * 100` - 分段進度百分比

## New Frontend State Models

### SelectionState

**Purpose**: 管理用戶選中的下載項目

**Implementation**: Jotai atom in `src/stores/downloadStore.ts`

```typescript
// 選中的任務 ID（單選）
export const selectedTaskIdAtom = atom<string | null>(null);

// 衍生 atom：選中的任務完整資料
export const selectedTaskAtom = atom((get) => {
  const selectedId = get(selectedTaskIdAtom);
  const tasks = get(downloadTasksAtom);
  return tasks.find((t) => t.id === selectedId) || null;
});
```

**State Management Rules**:
- 同一時間只能選中一個任務（單選模式）
- 當選中的任務被刪除時，自動清除選中狀態
- 初始狀態為 `null`（無選中項目）

### DetailPanelState

**Purpose**: 管理詳細面板的 Tab 切換狀態

**Implementation**: Local component state (useState)

```typescript
type DetailTab = 'basic' | 'segments' | 'errors' | 'advanced';

// 在 DownloadDetailPanel 組件中
const [activeTab, setActiveTab] = useState<DetailTab>('basic');
```

**State Management Rules**:
- 預設顯示 'basic' Tab
- 當沒有錯誤訊息時，'errors' Tab 可能被禁用或隱藏
- Tab 狀態不需要持久化（關閉面板後重置）

## Derived/Calculated Data

### 表格顯示資料

基於 `DownloadTask`，計算用於表格顯示的格式化資料：

```typescript
interface TableDisplayData {
  id: string;                          // 原始 ID
  filename: string;                    // 原始檔名
  filesizeFormatted: string;           // 格式化大小 (e.g., "1.5 GB")
  progress: number;                    // 原始進度 (0-100)
  progressFormatted: string;           // 格式化進度 (e.g., "45.3%")
  speed: number;                       // 原始速度 (bytes/sec)
  speedFormatted: string;              // 格式化速度 (e.g., "2.3 MB/s" or "--")
  status: DownloadStatus;              // 原始狀態
  statusLabel: string;                 // 翻譯後的狀態標籤
  statusColor: ChipColor;              // 狀態顏色
  createdAt: string;                   // 原始時間戳
  createdAtFormatted: string;          // 相對時間 (e.g., "5分鐘前")
  remainingTime: number | null;        // 剩餘秒數
  remainingTimeFormatted: string;      // 格式化剩餘時間 (e.g., "2分30秒" or "--")
}
```

**Calculation Logic**:
- `filesizeFormatted`: 使用 `formatBytes()` 工具函數
- `speedFormatted`: 只有在 `status === 'Downloading' && speed > 0` 時顯示，否則 "--"
- `remainingTimeFormatted`: 使用 `calculateRemainingTime()` + `formatRemainingTime()`
- `createdAtFormatted`: 使用 `formatRelativeTime()` 工具函數

### 分段視覺化資料

基於 `DownloadSegment[]`，計算用於視覺化的資料：

```typescript
interface SegmentVisualizationData {
  segmentId: number;                   // 分段編號
  status: SegmentStatus;               // 狀態
  color: string;                       // 顏色 (根據狀態)
  tooltipData: {                       // 懸停顯示的資料
    segmentId: number;
    sizeRange: string;                 // e.g., "0 MB - 10 MB"
    downloaded: string;                // e.g., "7.5 MB"
    progress: string;                  // e.g., "75.0%"
    status: string;                    // 翻譯後的狀態
    retryCount: number;
    proxy: string | null;
  };
}
```

**Color Mapping**:
- `Pending`: `#9e9e9e` (灰色)
- `Downloading`: `#2196f3` (藍色)
- `Completed`: `#4caf50` (綠色)
- `Failed`: `#f44336` (紅色)

## Data Flow

### 初始載入

```
1. App 啟動
   ↓
2. Tauri backend 載入下載歷史 (JSON 檔案)
   ↓
3. 發送 get_all_downloads command
   ↓
4. Frontend 收到 DownloadTask[]
   ↓
5. 更新 downloadTasksAtom
   ↓
6. DownloadTable 自動重新渲染
```

### 選擇下載項目

```
1. 用戶點擊 TableRow
   ↓
2. onClick handler 觸發
   ↓
3. 更新 selectedTaskIdAtom
   ↓
4. selectedTaskAtom 自動計算
   ↓
5. DownloadDetailPanel 收到新的 selectedTask
   ↓
6. 面板顯示新項目的詳細資訊
```

### 實時進度更新

```
1. Backend 下載進行中
   ↓
2. 每 500ms 發送 download_progress event (Tauri event)
   ↓
3. Frontend event listener 收到更新
   ↓
4. 更新對應的 DownloadTask 在 downloadTasksAtom
   ↓
5. 只有該 TableRow 重新渲染 (React.memo)
   ↓
6. 如果是選中的任務，DetailPanel 也更新
```

### 任務刪除

```
1. 用戶刪除任務
   ↓
2. 調用 delete_download command
   ↓
3. Backend 刪除任務並發送 download_deleted event
   ↓
4. Frontend 從 downloadTasksAtom 移除任務
   ↓
5. 如果被刪除的任務是選中的任務
   ↓
6. useEffect 檢測到 selectedTask 為 null
   ↓
7. 清除 selectedTaskIdAtom (設為 null)
   ↓
8. DetailPanel 顯示「請選擇一個下載項目」
```

## No New Backend Data Models

此功能**不需要**修改後端資料模型。所有變更都在前端 UI 層，使用現有的 `DownloadTask` 和 `DownloadSegment` 資料結構。

## Data Persistence

- **DownloadTask**: 持久化在 `~/Library/Application Support/zoo-downloader/downloads.json`（macOS）或等效路徑
- **SelectionState**: 不持久化（每次啟動應用程式時重置為 null）
- **DetailPanelState**: 不持久化（每次打開面板時重置為 'basic' tab）

## Data Validation

### Frontend Validation

在顯示前驗證資料的完整性：

```typescript
// 確保必要欄位存在
function validateDownloadTask(task: any): task is DownloadTask {
  return (
    typeof task.id === 'string' &&
    typeof task.url === 'string' &&
    typeof task.filename === 'string' &&
    typeof task.filesize === 'number' && task.filesize >= 0 &&
    typeof task.progress === 'number' && task.progress >= 0 && task.progress <= 100 &&
    typeof task.currentSpeed === 'number' && task.currentSpeed >= 0 &&
    typeof task.status === 'string' &&
    typeof task.createdAt === 'string' &&
    Array.isArray(task.segments)
  );
}
```

### Edge Cases

- **filesize === 0**: 顯示為 "未知大小"
- **currentSpeed === 0**: 剩餘時間顯示為 "--"
- **segments.length === 0**: 分段視覺化 Tab 顯示 "無分段資訊"
- **errorMessage === null**: 錯誤信息 Tab 顯示 "無錯誤訊息"

## Summary

此功能重新使用現有的資料模型，不引入新的後端結構。主要變更為：

1. 新增前端狀態：`selectedTaskIdAtom`（選中項目）
2. 新增衍生資料：`TableDisplayData` 和 `SegmentVisualizationData`（格式化用於顯示）
3. 實時更新策略：使用 Jotai 的細粒度訂閱 + React.memo 優化渲染

所有資料流動保持單向：Backend → Tauri Events → Jotai Atoms → React Components。
