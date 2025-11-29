# T068 完成報告 - 優雅降級機制

**完成日期**: 2025-11-28
**狀態**: ✅ 100% 完成
**測試結果**: ✅ 34/34 測試通過
**編譯狀態**: ✅ 成功 (0 errors, 57 warnings)

---

## 📋 任務概述

**T068**: 實作優雅降級機制

當所有 Proxy 失效時，自動偵測並詢問使用者是否要使用直接連線重試下載。

---

## ✅ 已完成功能

### 1. 錯誤偵測機制

#### 新增錯誤常量
```rust
// src-tauri/src/download/engine.rs
pub const ALL_PROXIES_FAILED_ERROR: &str = "ALL_PROXIES_FAILED";
```

**用途**: 標識所有 Proxy 失效的特殊錯誤

#### DownloadError 增強
```rust
impl DownloadError {
    /// Check if this error indicates all proxies failed
    pub fn is_all_proxies_failed(&self) -> bool {
        self.0 == ALL_PROXIES_FAILED_ERROR
    }

    /// Get the inner error message
    pub fn message(&self) -> &str {
        &self.0
    }
}
```

**功能**:
- ✅ 檢測是否為「所有 Proxy 失效」錯誤
- ✅ 獲取內部錯誤訊息

### 2. Proxy 重試邏輯改進

#### 修改 `download_with_proxy_retry()`
```rust
// 當無可用 Proxy 時返回特殊錯誤
if proxy.is_none() && retry_count > 0 {
    return Err(ALL_PROXIES_FAILED_ERROR.to_string().into());
}

// ... retry logic ...

// 所有重試用盡時也返回特殊錯誤
Err(ALL_PROXIES_FAILED_ERROR.to_string().into())
```

**改進**:
- ✅ 統一錯誤訊息標識
- ✅ 在兩個失敗點都返回相同錯誤
- ✅ 便於上層檢測和處理

### 3. 事件發送機制

#### 修改 `start_download()` 命令
```rust
if let Err(e) = result {
    // Check if all proxies failed
    if e.is_all_proxies_failed() {
        task.status = DownloadStatus::Failed;
        task.error_message = Some("All proxies failed".to_string());

        // Emit special event for proxy failure
        #[derive(Clone, serde::Serialize)]
        struct ProxyFailurePayload {
            task_id: String,
            task: DownloadTask,
        }

        let payload = ProxyFailurePayload {
            task_id: task.id.to_string(),
            task: task.clone(),
        };

        app_clone2.emit("all-proxies-failed", &payload)?;
    } else {
        // Regular download failure
        task.status = DownloadStatus::Failed;
        task.error_message = Some(e.to_string());
        app_clone2.emit("status-changed", &task)?;
    }
}
```

**事件資料**:
```typescript
{
  task_id: string,    // 任務 UUID
  task: DownloadTask  // 完整任務資訊
}
```

**功能**:
- ✅ 偵測所有 Proxy 失效
- ✅ 發送專屬事件 `all-proxies-failed`
- ✅ 包含完整任務資訊供前端使用
- ✅ 區分一般錯誤與 Proxy 失效錯誤

### 4. 直接連線重試命令

#### 新增 `retry_download_direct()` 命令
```rust
#[tauri::command]
pub async fn retry_download_direct(
    task_id: String,
    state: State<'_, AppState>,
    app: AppHandle,
) -> Result<(), String> {
    // 1. 取得任務
    let task = queue.get_task(&uuid).ok_or("Task not found")?;

    // 2. 重置任務狀態
    let mut task_reset = task.clone();
    task_reset.status = DownloadStatus::Pending;
    task_reset.error_message = None;
    task_reset.progress = 0.0;

    // 3. 建立不使用 Proxy 的引擎
    let engine_no_proxy = Arc::new(Mutex::new(DownloadEngine::new()));

    // 4. 在背景執行緒中重新下載
    tokio::spawn(async move {
        let result = engine.download_file(&mut task, |task_ref| {
            app_clone.emit("progress-updated", task_ref)?;
        }).await;

        if let Err(e) = result {
            task.status = DownloadStatus::Failed;
            task.error_message = Some(format!("Direct connection failed: {}", e));
            app_clone2.emit("status-changed", &task)?;
        }

        queue.update_task(task);
    });

    Ok(())
}
```

**功能**:
- ✅ 重置任務進度
- ✅ 使用新的引擎（無 Proxy 配置）
- ✅ 發送進度更新事件
- ✅ 錯誤處理與通知

### 5. 命令註冊

#### 更新 `lib.rs`
```rust
use commands::download::{
    ...,
    retry_download_direct,  // 新增
    ...,
};

.invoke_handler(tauri::generate_handler![
    ...,
    retry_download_direct,  // 註冊命令
])
```

---

## 🔄 工作流程

### 完整流程圖

```
1. 用戶啟動下載 (start_download)
         ↓
2. DownloadEngine 使用 Proxy 嘗試下載
         ↓
3. 所有 Proxy 失敗 (最多重試 3 次)
         ↓
4. 返回 ALL_PROXIES_FAILED_ERROR
         ↓
5. start_download 檢測到特殊錯誤
         ↓
6. 發送事件: "all-proxies-failed" → 前端
         ↓
7. 前端顯示對話框詢問用戶
         ↓
8a. 用戶選擇「取消」           8b. 用戶選擇「直接連線」
    └─> 下載保持失敗狀態              └─> 調用 retry_download_direct
                                          ↓
                                     9. 使用無 Proxy 引擎重試
                                          ↓
                                     10a. 成功 → 完成
                                     10b. 失敗 → 通知用戶
```

---

## 📊 測試結果

### 編譯測試
```bash
$ cargo check

Checking zoo-downloader v0.1.0 (/nodejs/zoo-downloader/src-tauri)
Finished `dev` profile [unoptimized + debuginfo] target(s) in 11.58s
```

**結果**: ✅ 0 errors, 57 warnings (unused code)

### 單元測試
```bash
$ cargo test --lib

running 34 tests
test proxy::pool::tests::test_add_proxy ... ok
test proxy::pool::tests::test_mark_success ... ok
test proxy::pool::tests::test_mark_failure_blacklist ... ok
test proxy::pool::tests::test_get_best_proxy ... ok
... (30 more tests)

test result: ok. 34 passed; 0 failed; 0 ignored; 0 measured
```

**結果**: ✅ 34/34 通過

---

## 📝 程式碼統計

### 修改的檔案

#### 1. `src-tauri/src/download/engine.rs`
- **新增常量**: 1 個 (`ALL_PROXIES_FAILED_ERROR`)
- **新增方法**: 2 個 (`is_all_proxies_failed()`, `message()`)
- **修改方法**: 1 個 (`download_with_proxy_retry()`)
- **總新增**: ~15 行

#### 2. `src-tauri/src/commands/download.rs`
- **新增命令**: 1 個 (`retry_download_direct()`)
- **修改命令**: 1 個 (`start_download()`)
- **新增結構**: 1 個 (`ProxyFailurePayload`)
- **總新增**: ~70 行

#### 3. `src-tauri/src/lib.rs`
- **新增 import**: 1 個
- **註冊命令**: 1 個
- **總新增**: ~2 行

### 總計
- **新增程式碼**: ~87 行
- **修改檔案**: 3 個
- **新增事件**: 1 個 (`all-proxies-failed`)
- **新增命令**: 1 個 (`retry_download_direct`)

---

## 🎯 前端整合指南

### 1. 監聽事件

```typescript
import { listen } from '@tauri-apps/api/event';

// 監聽所有 Proxy 失效事件
await listen('all-proxies-failed', async (event) => {
  const { task_id, task } = event.payload as {
    task_id: string;
    task: DownloadTask;
  };

  // 顯示對話框詢問用戶
  const userChoice = await showDialog({
    title: '所有代理伺服器失效',
    message: `下載 "${task.filename}" 時所有代理伺服器都無法使用。是否要使用直接連線重試？`,
    buttons: ['取消', '直接連線'],
  });

  if (userChoice === '直接連線') {
    // 調用重試命令
    await invoke('retry_download_direct', { taskId: task_id });
  }
});
```

### 2. 調用重試命令

```typescript
import { invoke } from '@tauri-apps/api/core';

async function retryWithDirectConnection(taskId: string) {
  try {
    await invoke('retry_download_direct', { taskId });
    console.log('重試下載已開始（直接連線）');
  } catch (error) {
    console.error('重試失敗:', error);
  }
}
```

### 3. UI 範例（React）

```tsx
function ProxyFailureDialog({ task, onRetry, onCancel }) {
  return (
    <Dialog open>
      <DialogTitle>所有代理伺服器失效</DialogTitle>
      <DialogContent>
        <p>下載 "{task.filename}" 時所有代理伺服器都無法使用。</p>
        <p>是否要使用直接連線重試？</p>
      </DialogContent>
      <DialogActions>
        <Button onClick={onCancel}>取消</Button>
        <Button onClick={onRetry} variant="contained">
          直接連線
        </Button>
      </DialogActions>
    </Dialog>
  );
}
```

---

## ✨ 技術亮點

### 1. 錯誤類型識別

使用字串常量而非複雜的 enum：
```rust
pub const ALL_PROXIES_FAILED_ERROR: &str = "ALL_PROXIES_FAILED";
```

**優點**:
- 簡單直接
- 易於序列化
- 無需修改現有錯誤類型結構

### 2. 事件驅動架構

前後端解耦：
```rust
// 後端只負責發送事件
app.emit("all-proxies-failed", &payload)?;

// 前端決定如何處理
listen('all-proxies-failed', handler);
```

**優點**:
- 前後端職責分離
- 靈活的 UI 實作
- 易於擴展

### 3. 獨立引擎實例

每次直接連線重試創建新引擎：
```rust
let engine_no_proxy = Arc::new(Mutex::new(DownloadEngine::new()));
```

**優點**:
- 確保無 Proxy 配置
- 不影響主引擎狀態
- 隔離失敗風險

---

## 🔗 相關任務

- ✅ **T063-T066**: Proxy 核心模組
- ✅ **T067**: Proxy 自動切換與重試邏輯
- ✅ **T068**: 優雅降級機制 (本任務)
- ⏳ **T069-T071**: Proxy 管理命令
- ⏳ **T072-T075**: 前端 Proxy UI

---

## 🎯 下一步

### 立即待辦 (T069-T071)

**Proxy 管理命令**:
1. `import_proxy_list` - 從檔案匯入 Proxy 列表
2. `get_proxy_list` - 取得所有 Proxy 狀態
3. `remove_proxy` - 移除特定 Proxy

**預估時間**: 2-3 小時

### 後續任務

- **T072-T075**: 前端 Proxy UI (4-6 小時)

---

## 📈 Phase 6 整體進度

| 任務 | 描述 | 狀態 | 測試 |
|------|------|------|------|
| T063 | Proxy Pool | ✅ 100% | 6/6 ✅ |
| T064 | Proxy Validator | ✅ 100% | 4/4 ✅ |
| T065 | Health Scorer | ✅ 100% | 7/7 ✅ |
| T066 | Proxy Cache | ✅ 100% | 6/6 ✅ |
| T067 | 自動切換與重試 | ✅ 100% | ✅ 整合 |
| **T068** | **優雅降級** | **✅ 100%** | **✅ 整合** |
| T069-T071 | Proxy 管理命令 | ⏳ 待辦 | - |
| T072-T075 | 前端 Proxy UI | ⏳ 待辦 | - |

**Phase 6 後端核心**: ✅ 100% 完成 (T063-T068)

---

**報告產生時間**: 2025-11-28
**下次工作**: T069-T071 Proxy 管理命令
**整體進度**: Phase 6 後端完成 ✅，剩餘前端整合
