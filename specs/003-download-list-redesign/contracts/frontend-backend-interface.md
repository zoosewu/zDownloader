# Frontend-Backend Interface Contract

**Feature**: 003-download-list-redesign
**Date**: 2025-12-03
**Protocol**: Tauri Commands + Events

## Overview

此功能重新使用現有的 Tauri commands 和 events，不需要新的 API。本文檔記錄相關的介面契約以供參考。

## Tauri Commands (Frontend → Backend)

### get_all_downloads

**Purpose**: 獲取所有下載任務列表

**Request**:
```typescript
invoke<DownloadTask[]>('get_all_downloads')
```

**Response**:
```typescript
DownloadTask[] // Array of all download tasks
```

**Used By**: DownloadTable 初始化時載入所有任務

**Error Handling**:
```typescript
try {
  const tasks = await invoke<DownloadTask[]>('get_all_downloads');
} catch (error) {
  console.error('Failed to load downloads:', error);
  // 顯示錯誤提示
}
```

### start_download

**Purpose**: 開始或恢復一個下載任務

**Request**:
```typescript
invoke('start_download', { taskId: string })
```

**Parameters**:
- `taskId: string` - 下載任務的 UUID

**Response**: `void` (成功無返回值)

**Side Effects**: 觸發 `download_started` event

**Used By**: 表格中的「開始」按鈕

### pause_download

**Purpose**: 暫停一個正在進行的下載

**Request**:
```typescript
invoke('pause_download', { taskId: string })
```

**Parameters**:
- `taskId: string` - 下載任務的 UUID

**Response**: `void`

**Side Effects**: 觸發 `download_paused` event

**Used By**: 表格中的「暫停」按鈕

### cancel_download

**Purpose**: 取消一個下載任務

**Request**:
```typescript
invoke('cancel_download', { taskId: string })
```

**Parameters**:
- `taskId: string` - 下載任務的 UUID

**Response**: `void`

**Side Effects**: 觸發 `download_cancelled` event

**Used By**: 表格中的「取消」按鈕

### delete_download

**Purpose**: 刪除一個下載任務記錄

**Request**:
```typescript
invoke('delete_download', { taskId: string })
```

**Parameters**:
- `taskId: string` - 下載任務的 UUID

**Response**: `void`

**Side Effects**: 觸發 `download_deleted` event

**Used By**: 詳細面板或表格的「刪除」功能（未來可能添加）

## Tauri Events (Backend → Frontend)

### download_progress

**Purpose**: 實時更新下載進度

**Payload**:
```typescript
{
  taskId: string;
  progress: number;        // 0-100
  currentSpeed: number;    // bytes/sec
  downloadedBytes: number; // total bytes downloaded
  segments: DownloadSegment[]; // updated segment states
}
```

**Frequency**: 每 500ms 發送一次（對於活躍的下載）

**Subscription**:
```typescript
import { listen } from '@tauri-apps/api/event';

const unlisten = await listen<DownloadProgressPayload>('download_progress', (event) => {
  const { taskId, progress, currentSpeed, segments } = event.payload;
  // 更新對應任務的狀態
  updateDownloadTask(taskId, { progress, currentSpeed, segments });
});

// 清理時
unlisten();
```

**Used By**: DownloadTable 和 DownloadDetailPanel 實時更新

### download_started

**Purpose**: 通知下載已開始

**Payload**:
```typescript
{
  taskId: string;
  startedAt: string; // ISO 8601 timestamp
}
```

**Used By**: 更新任務狀態為 'Downloading'

### download_paused

**Purpose**: 通知下載已暫停

**Payload**:
```typescript
{
  taskId: string;
}
```

**Used By**: 更新任務狀態為 'Paused'

### download_completed

**Purpose**: 通知下載已完成

**Payload**:
```typescript
{
  taskId: string;
  completedAt: string; // ISO 8601 timestamp
  filepath: string;    // 完整檔案路徑
}
```

**Used By**: 更新任務狀態為 'Completed'，顯示成功通知

### download_failed

**Purpose**: 通知下載失敗

**Payload**:
```typescript
{
  taskId: string;
  errorMessage: string;
  retryCount: number;
}
```

**Used By**: 更新任務狀態為 'Failed'，顯示錯誤訊息

### download_deleted

**Purpose**: 通知任務已被刪除

**Payload**:
```typescript
{
  taskId: string;
}
```

**Used By**: 從列表中移除任務，清除選中狀態（如果被刪除的是選中項目）

## Data Types

### DownloadTask

```typescript
interface DownloadTask {
  id: string;                      // UUID
  url: string;
  filename: string;
  filepath: string;
  filesize: number;                // bytes
  status: DownloadStatus;
  progress: number;                // 0-100
  currentSpeed: number;            // bytes/sec
  segments: DownloadSegment[];
  createdAt: string;               // ISO 8601
  startedAt: string | null;        // ISO 8601
  completedAt: string | null;      // ISO 8601
  errorMessage: string | null;
  scheduledStart: string | null;   // ISO 8601
  retryCount: number;
  maxRetries: number | null;
  lastRetryAt: string | null;      // ISO 8601
}
```

### DownloadSegment

```typescript
interface DownloadSegment {
  id: string;                      // UUID
  segmentId: number;               // 0-based index
  startByte: number;
  endByte: number;
  downloadedBytes: number;
  status: SegmentStatus;
  retryCount: number;
  proxy: string | null;
}
```

### DownloadStatus

```typescript
type DownloadStatus =
  | 'Pending'
  | 'Downloading'
  | 'Paused'
  | 'Completed'
  | 'Failed';
```

### SegmentStatus

```typescript
type SegmentStatus =
  | 'Pending'
  | 'Downloading'
  | 'Completed'
  | 'Failed';
```

## Serialization Format

所有 Tauri commands 和 events 使用 JSON 序列化，欄位名稱使用 **camelCase** 格式（Rust 端使用 `#[serde(rename_all = "camelCase")]`）。

**Backend (Rust)**:
```rust
#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DownloadTask {
    pub id: Uuid,
    pub url: String,
    pub filename: String,
    pub filepath: String,
    pub filesize: u64,
    pub status: DownloadStatus,
    pub progress: f64,
    pub current_speed: u64,  // → currentSpeed in JSON
    pub segments: Vec<DownloadSegment>,
    pub created_at: DateTime<Utc>,  // → createdAt in JSON
    // ... 其他欄位
}
```

**Frontend (TypeScript)**:
```typescript
interface DownloadTask {
  id: string;
  url: string;
  filename: string;
  filepath: string;
  filesize: number;
  status: DownloadStatus;
  progress: number;
  currentSpeed: number;  // ✅ camelCase
  segments: DownloadSegment[];
  createdAt: string;     // ✅ camelCase
  // ... 其他欄位
}
```

## Error Handling

### Command Errors

所有 Tauri commands 可能拋出錯誤，前端必須處理：

```typescript
try {
  await invoke('start_download', { taskId });
} catch (error: any) {
  if (error.includes('Task not found')) {
    // 任務不存在
    showError('下載任務不存在');
  } else if (error.includes('Already downloading')) {
    // 任務已在下載中
    showWarning('任務已在下載中');
  } else {
    // 其他錯誤
    showError(`操作失敗：${error}`);
  }
}
```

### Event Reliability

- **保證順序**: 同一個任務的 events 按時間順序發送
- **不保證送達**: Frontend 應該有錯誤恢復機制（例如定期輪詢狀態）
- **重複處理**: Events 可能重複發送，frontend 應該冪等處理

## Performance Considerations

### Event Frequency

- `download_progress`: 每 500ms（對於活躍下載）
- 其他 events: 按需發送（狀態變更時）

### Batching

如果同時有多個下載進行中，`download_progress` events 會分別發送（不批次處理）。Frontend 使用 Jotai 的細粒度訂閱確保只有變更的任務重新渲染。

### Throttling

Frontend 可能需要 throttle UI 更新：

```typescript
const throttledUpdate = useCallback(
  throttle((taskId: string, data: Partial<DownloadTask>) => {
    updateDownloadTask(taskId, data);
  }, 500), // 最多每 500ms 更新一次 UI
  []
);
```

## Testing Contracts

### Unit Tests

使用 mock Tauri commands 進行單元測試：

```typescript
// __tests__/mocks/tauri.ts
export const mockInvoke = vi.fn();

vi.mock('@tauri-apps/api/core', () => ({
  invoke: mockInvoke,
}));

// 在測試中
mockInvoke.mockResolvedValueOnce([
  { id: '1', filename: 'test.zip', /* ... */ },
]);

const tasks = await invoke<DownloadTask[]>('get_all_downloads');
expect(tasks).toHaveLength(1);
```

### Integration Tests

測試完整的 command-event 循環（需要 Tauri 測試環境）：

```typescript
test('download progress updates table', async () => {
  // 1. 調用 start_download
  await invoke('start_download', { taskId: 'test-id' });

  // 2. 等待 download_started event
  await waitFor(() => {
    expect(getTaskStatus('test-id')).toBe('Downloading');
  });

  // 3. 模擬 download_progress event
  emitEvent('download_progress', {
    taskId: 'test-id',
    progress: 50,
    currentSpeed: 1024000,
  });

  // 4. 驗證 UI 更新
  await waitFor(() => {
    expect(screen.getByText('50.0%')).toBeInTheDocument();
  });
});
```

## Versioning

**Current Version**: 1.0.0 (與現有 API 相同)

**Backward Compatibility**: 此功能不改變任何 API 契約，完全向後相容。

## Summary

| 介面類型 | 數量 | 變更 |
|---------|------|------|
| Tauri Commands | 5個（重用現有） | 無變更 |
| Tauri Events | 6個（重用現有） | 無變更 |
| Data Models | 2個（重用現有） | 無變更 |
| 新增 API | 0個 | N/A |

此功能是純前端 UI 重新設計，不需要任何新的 backend API。所有現有的 Tauri commands 和 events 保持不變。
