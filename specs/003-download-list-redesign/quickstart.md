# Quickstart: 下載列表 UI 重新設計

**Feature**: 003-download-list-redesign
**Date**: 2025-12-03
**For**: Developers implementing this feature

## What This Feature Does

將現有的卡片式下載列表重新設計為：
1. **表格視圖**：每個下載項目顯示為一行，包含關鍵資訊
2. **選擇與高亮**：點擊項目進行選擇並高亮
3. **詳細面板**：下方顯示選中項目的完整資訊，使用 Tab 切換
4. **分段視覺化**：彩色方格顯示分段狀態，支持懸停工具提示

## 5-Minute Setup

### 1. 閱讀相關文檔

```bash
# 從規格開始
cat specs/003-download-list-redesign/spec.md

# 了解技術決策
cat specs/003-download-list-redesign/research.md

# 查看資料模型
cat specs/003-download-list-redesign/data-model.md
```

### 2. 檢查現有代碼

```bash
# 現有的下載列表組件
code src/components/DownloadList.tsx
code src/components/DownloadItem.tsx

# 下載狀態管理
code src/stores/downloadStore.ts

# 下載類型定義
code src/types/download.ts
```

### 3. 安裝依賴（如果需要）

```bash
# 檢查是否需要新依賴
# 本功能不需要新的 npm packages，都是現有依賴

# 如果需要 react-window（用於虛擬化大量分段）
pnpm add react-window
pnpm add -D @types/react-window
```

### 4. 運行開發環境

```bash
# 啟動前端開發伺服器
pnpm dev

# 在另一個終端啟動 Tauri
pnpm tauri:dev
```

## Key Files to Modify/Create

### New Components (需要創建)

```
src/components/
├── DownloadTable.tsx              ⭐ 主表格組件
├── DownloadTableRow.tsx           ⭐ 表格行組件
├── DownloadDetailPanel.tsx        ⭐ 詳細面板組件
├── SegmentVisualization.tsx       ⭐ 分段視覺化組件
├── SegmentGridItem.tsx            ⭐ 單個分段方格組件
├── BasicInfoTab.tsx               ⭐ 基本信息 Tab
├── ErrorInfoTab.tsx               ⭐ 錯誤信息 Tab
└── AdvancedInfoTab.tsx            ⭐ 進階資訊 Tab
```

### Existing Files to Modify

```
src/
├── components/
│   └── DownloadList.tsx           ⚙️ 改為渲染 DownloadTable
├── stores/
│   └── downloadStore.ts           ⚙️ 添加 selectedTaskIdAtom
├── hooks/
│   ├── useSelectedDownload.ts     ⭐ NEW: 選中項目管理
│   └── useTimeEstimation.ts       ⭐ NEW: 時間估算
└── utils/
    ├── formatters.ts              ⚙️ 添加時間格式化函數
    └── downloadCalculations.ts    ⭐ NEW: 計算函數（剩餘時間等）
```

### Test Files (需要創建)

```
src/__tests__/
├── components/
│   ├── DownloadTable.test.tsx
│   ├── DownloadDetailPanel.test.tsx
│   └── SegmentVisualization.test.tsx
└── utils/
    └── downloadCalculations.test.ts
```

## Implementation Order

### Phase 1: 基礎架構 (2-3 hours)

1. **創建選中狀態管理**
   ```bash
   # 修改 src/stores/downloadStore.ts
   # 添加 selectedTaskIdAtom 和 selectedTaskAtom
   ```

2. **創建計算工具函數**
   ```bash
   # 創建 src/utils/downloadCalculations.ts
   # 實現 calculateRemainingTime 和 formatRemainingTime
   ```

3. **創建 useSelectedDownload hook**
   ```bash
   # 創建 src/hooks/useSelectedDownload.ts
   # 包裝選中狀態的操作
   ```

### Phase 2: 表格視圖 (3-4 hours)

4. **創建 DownloadTableRow 組件**
   ```bash
   # 創建 src/components/DownloadTableRow.tsx
   # 使用 React.memo 優化性能
   ```

5. **創建 DownloadTable 組件**
   ```bash
   # 創建 src/components/DownloadTable.tsx
   # 使用 MUI Table 組件
   ```

6. **修改 DownloadList 組件**
   ```bash
   # 修改 src/components/DownloadList.tsx
   # 渲染 DownloadTable 而非卡片列表
   ```

### Phase 3: 詳細面板 (4-5 hours)

7. **創建 Tab 組件**
   ```bash
   # 創建 src/components/BasicInfoTab.tsx
   # 創建 src/components/ErrorInfoTab.tsx
   # 創建 src/components/AdvancedInfoTab.tsx
   ```

8. **創建 DownloadDetailPanel 組件**
   ```bash
   # 創建 src/components/DownloadDetailPanel.tsx
   # 整合所有 Tab 組件
   ```

### Phase 4: 分段視覺化 (3-4 hours)

9. **創建 SegmentGridItem 組件**
   ```bash
   # 創建 src/components/SegmentGridItem.tsx
   # 使用 MUI Tooltip 顯示詳細信息
   ```

10. **創建 SegmentVisualization 組件**
    ```bash
    # 創建 src/components/SegmentVisualization.tsx
    # 使用 CSS Grid 布局
    ```

11. **整合到 DetailPanel**
    ```bash
    # 在 DownloadDetailPanel 的分段 Tab 中使用
    ```

### Phase 5: 測試與優化 (2-3 hours)

12. **編寫單元測試**
    ```bash
    pnpm test
    ```

13. **性能測試**
    ```bash
    # 使用 React DevTools Profiler
    # 測試大量下載項目和分段的渲染性能
    ```

14. **多語言支持**
    ```bash
    # 確保所有文字都通過 react-i18next
    # 更新語言文件
    ```

## Common Gotchas

### 1. React.memo 比較函數

❌ **錯誤**：
```typescript
const DownloadTableRow = React.memo((props) => {
  // ...
});
```

✅ **正確**：
```typescript
const DownloadTableRow = React.memo((props) => {
  // ...
}, (prev, next) => {
  return prev.taskId === next.taskId && prev.isSelected === next.isSelected;
});
```

### 2. 時間計算的邊界情況

❌ **錯誤**：
```typescript
const remaining = remainingBytes / currentSpeed;  // 可能除以 0
```

✅ **正確**：
```typescript
if (currentSpeed === 0 || task.status !== 'Downloading') {
  return null;
}
const remaining = remainingBytes / currentSpeed;
```

### 3. 選中項目被刪除

❌ **錯誤**：不處理選中項目被刪除的情況

✅ **正確**：
```typescript
useEffect(() => {
  if (selectedTaskId && !tasks.find(t => t.id === selectedTaskId)) {
    setSelectedTaskId(null);  // 清除選中狀態
  }
}, [tasks, selectedTaskId]);
```

### 4. 分段方格過多導致卡頓

❌ **錯誤**：直接渲染 1000 個分段

✅ **正確**：
```typescript
if (segments.length > 500) {
  return <VirtualizedSegmentGrid segments={segments} />;
}
```

## Quick Reference

### Material-UI Components Used

- `Table`, `TableHead`, `TableBody`, `TableRow`, `TableCell` - 表格
- `TableContainer`, `Paper` - 容器
- `Tabs`, `Tab` - Tab 切換
- `Tooltip` - 懸停工具提示
- `Chip` - 狀態標籤
- `Box`, `Stack` - 布局
- `Typography` - 文字

### Jotai Patterns

```typescript
// 創建 atom
export const selectedTaskIdAtom = atom<string | null>(null);

// 使用 atom
const [selectedId, setSelectedId] = useAtom(selectedTaskIdAtom);

// 只讀 atom
const selectedTask = useAtomValue(selectedTaskAtom);
```

### Tauri Event Handling

```typescript
import { listen } from '@tauri-apps/api/event';

const unlisten = await listen('download_progress', (event) => {
  // 處理事件
});

// 清理
return () => {
  unlisten();
};
```

## Testing Strategy

### Unit Tests

```bash
# 測試計算函數
pnpm test src/__tests__/utils/downloadCalculations.test.ts

# 測試組件
pnpm test src/__tests__/components/DownloadTable.test.tsx
```

### Manual Testing

1. **表格視圖**：
   - 創建 5-10 個下載任務
   - 驗證所有欄位正確顯示
   - 驗證實時更新

2. **選擇與高亮**：
   - 點擊不同項目
   - 驗證高亮切換
   - 驗證詳細面板更新

3. **分段視覺化**：
   - 選擇有多個分段的任務
   - 驗證方格顏色正確
   - 驗證懸停工具提示

4. **邊界情況**：
   - 空列表
   - 速度為 0
   - 刪除選中項目
   - 大量分段（> 500）

## Performance Targets

| 指標 | 目標 | 測試方法 |
|------|------|---------|
| 表格初始渲染 | < 1s | React DevTools Profiler |
| 選擇響應 | < 200ms | Performance API |
| 分段視覺化 (100個) | < 500ms | React DevTools Profiler |
| Tooltip 顯示 | < 100ms | 手動測試 |
| 實時更新延遲 | < 1s | 手動測試 |

## Need Help?

- **規格問題**: 查看 `spec.md`
- **技術決策**: 查看 `research.md`
- **資料結構**: 查看 `data-model.md`
- **API 契約**: 查看 `contracts/frontend-backend-interface.md`
- **實施任務**: 等待 `/speckit.tasks` 生成 `tasks.md`

## Next Steps

1. 閱讀完整規格：`specs/003-download-list-redesign/spec.md`
2. 了解技術決策：`specs/003-download-list-redesign/research.md`
3. 開始實施 Phase 1
4. 運行 `/speckit.tasks` 生成詳細的實施任務清單
