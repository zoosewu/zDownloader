# Phase 6: Proxy 支援與管理 - 進度報告

**生成時間**: 2025-11-28
**階段**: Phase 6 - User Story 4 (Proxy 支援與管理)
**狀態**: 核心模組完成，整合進行中

---

## 📋 執行摘要

Phase 6 的核心 Proxy 管理模組已完成實作並通過所有單元測試（共 23 個測試）。已建立完整的 Proxy 池管理、驗證、健康度評分和快取系統。下載引擎已加入 Proxy 支援的基礎架構。

### 完成進度

- ✅ **T063**: Proxy 池核心 (pool.rs) - 100%
- ✅ **T064**: Proxy 驗證器 (validator.rs) - 100%
- ✅ **T065**: 健康度評分算法 (health_scorer.rs) - 100%
- ✅ **T066**: Proxy 快取 (cache.rs) - 100%
- 🔄 **T067**: Proxy 自動切換（基礎架構完成，整合進行中）- 60%
- ⏳ **T068**: 優雅降級（待實作）- 0%
- ⏳ **T069-T071**: Proxy 管理命令（待實作）- 0%
- ⏳ **T072-T075**: 前端 Proxy 管理 UI（待實作）- 0%

---

## ✅ 已完成模組

### 1. Proxy Pool (src-tauri/src/proxy/pool.rs)

**功能**:
- CRUD 操作：新增、刪除、查詢 Proxy
- 選擇策略：最佳健康度、隨機選擇
- 狀態管理：成功/失敗追蹤、自動黑名單機制
- 健康度更新：指數移動平均（EMA）延遲計算
- 並發安全：使用 Arc<RwLock<Vec<ProxyInfo>>>

**關鍵方法**:
```rust
pub struct ProxyPool {
    proxies: Arc<RwLock<Vec<ProxyInfo>>>,
    max_failures_before_blacklist: u64,
}

// 選擇策略
pub async fn get_best_proxy(&self) -> Option<ProxyInfo>
pub async fn get_random_proxy(&self) -> Option<ProxyInfo>

// 狀態管理
pub async fn mark_success(&self, proxy_id: Uuid, latency_ms: f64)
pub async fn mark_failure(&self, proxy_id: Uuid)
```

**測試覆蓋**: 6 個單元測試，全部通過
- 新增 Proxy
- 重複 Proxy 拒絕
- 成功標記與延遲更新
- 失敗追蹤與自動黑名單
- 最佳 Proxy 選擇
- 活躍 Proxy 過濾

---

### 2. Proxy Validator (src-tauri/src/proxy/validator.rs)

**功能**:
- 批次驗證：每批 50 個 Proxy 並發驗證
- 超時控制：每個 Proxy 5 秒超時
- 並發處理：使用 futures::stream with buffer_unordered
- 結果追蹤：驗證狀態、延遲、錯誤訊息

**關鍵方法**:
```rust
pub async fn validate_proxies_batch(proxies: Vec<ProxyInfo>) -> Vec<ValidationResult>
pub async fn validate_and_update_proxies(proxies: Vec<ProxyInfo>) -> Vec<ProxyInfo>
```

**驗證流程**:
1. 將 Proxy 列表分成每批 50 個
2. 使用 reqwest::Proxy 配置測試客戶端
3. 向 http://www.google.com 發送測試請求
4. 記錄延遲和結果
5. 更新 Proxy 狀態（Active/Failed）

**測試覆蓋**: 4 個單元測試，全部通過
- 單一無效 Proxy 驗證
- 批次多個 Proxy 驗證
- 分批處理（60 個 Proxy 測試）
- 驗證並更新狀態

---

### 3. Health Scorer (src-tauri/src/proxy/health_scorer.rs)

**功能**:
- 健康度分類：Excellent (≥0.8), Good (≥0.6), Fair (≥0.4), Poor (<0.4)
- 過濾功能：按最小健康度或類別過濾
- 統計分析：計算各類別數量、平均分數
- 排序功能：取得前 N 個最佳 Proxy

**健康度演算法** (在 ProxyInfo::calculate_health_score 中實作):
```rust
health_score = success_rate * 0.7 + latency_score * 0.2 + recency_bonus * 0.1
```
- **成功率 (70%)**: successful_requests / total_requests
- **延遲分數 (20%)**: 1.0 - (avg_latency_ms / 5000.0).min(1.0)
- **最近使用 (10%)**: 1.0 if used within 1 hour, else 0.0

**關鍵功能**:
```rust
pub fn categorize_health(health_score: f64, thresholds: &HealthThresholds) -> HealthCategory
pub fn filter_by_min_health(proxies: Vec<ProxyInfo>, min_health: f64) -> Vec<ProxyInfo>
pub fn calculate_health_stats(proxies: &[ProxyInfo]) -> HealthStats
pub fn get_best_n_proxies(mut proxies: Vec<ProxyInfo>, n: usize) -> Vec<ProxyInfo>
```

**測試覆蓋**: 7 個單元測試，全部通過

---

### 4. Proxy Cache (src-tauri/src/proxy/cache.rs)

**功能**:
- 持久化儲存：讀寫 proxies.txt
- 檔案格式：簡單的 "host:port" 每行一個
- 註解支援：# 開頭的行會被忽略
- 匯入功能：從外部檔案匯入 Proxy 列表
- 去重功能：附加時自動跳過重複項目

**檔案格式範例**:
```text
# Zoo Downloader Proxy Cache
# Format: host:port (one per line)
# Lines starting with # are ignored

127.0.0.1:8080
192.168.1.1:3128
proxy.example.com:8888
```

**關鍵方法**:
```rust
pub fn load_proxies_from_cache() -> Result<Vec<ProxyInfo>, String>
pub fn save_proxies_to_cache(proxies: &[ProxyInfo]) -> Result<(), String>
pub fn append_proxies_to_cache(new_proxies: &[ProxyInfo]) -> Result<(), String>
pub fn import_proxies_from_file(file_path: &str) -> Result<Vec<ProxyInfo>, String>
```

**測試覆蓋**: 6 個單元測試，全部通過

---

### 5. HTTP Client Proxy 支援 (src-tauri/src/download/client.rs)

**更新內容**:
```rust
// 新增方法
pub fn with_proxy(proxy_info: Option<&ProxyInfo>) -> Self

// 使用範例
let client = HttpClient::with_proxy(Some(&proxy_info));
```

**功能**:
- 可選 Proxy 配置
- 使用 reqwest::Proxy 配置 HTTP/HTTPS Proxy
- 向後兼容：預設不使用 Proxy

---

### 6. Download Engine 基礎架構 (src-tauri/src/download/engine.rs)

**新增欄位**:
```rust
pub struct DownloadEngine {
    http_client: HttpClient,
    segment_config: SegmentConfig,
    proxy_pool: Option<Arc<ProxyPool>>,  // 新增
    use_proxy: bool,                      // 新增
}
```

**新增方法**:
```rust
// 啟用 Proxy 支援
pub fn with_proxy_pool(mut self, proxy_pool: Arc<ProxyPool>) -> Self

// 設定是否使用 Proxy
pub fn set_use_proxy(&mut self, use_proxy: bool)

// 取得最佳可用 Proxy
async fn get_proxy(&self) -> Option<ProxyInfo>
```

**狀態**: 基礎架構已就緒，等待整合到實際下載流程

---

## 🔄 進行中的工作

### T067: Proxy 自動切換

**已完成**:
- ✅ ProxyPool 整合到 DownloadEngine
- ✅ HttpClient 支援 Proxy 配置
- ✅ get_proxy() 方法取得最佳 Proxy

**待完成** (估計 2-3 小時):
1. **修改 download_file 方法**
   - 取得 Proxy 並建立相應的 HttpClient
   - 包裝下載邏輯在重試迴圈中

2. **實作重試邏輯**
   ```rust
   async fn download_with_proxy_retry(&self, task: &mut DownloadTask, ...) -> Result<(), DownloadError> {
       let mut retry_count = 0;
       const MAX_RETRIES: usize = 3;

       loop {
           // 取得 Proxy
           let proxy = self.get_proxy().await;

           // 建立 HTTP 客戶端
           let client = HttpClient::with_proxy(proxy.as_ref());

           // 嘗試下載
           match self.download_with_client(&client, task, ...).await {
               Ok(_) => {
                   // 成功：標記 Proxy 為成功
                   if let Some(p) = proxy {
                       if let Some(pool) = &self.proxy_pool {
                           pool.mark_success(p.id, latency).await;
                       }
                   }
                   return Ok(());
               }
               Err(e) => {
                   // 失敗：標記 Proxy 為失敗
                   if let Some(p) = proxy {
                       if let Some(pool) = &self.proxy_pool {
                           pool.mark_failure(p.id).await;
                       }
                   }

                   retry_count += 1;
                   if retry_count >= MAX_RETRIES {
                       return Err(e);
                   }

                   // 繼續下一次重試（會取得新的 Proxy）
               }
           }
       }
   }
   ```

3. **更新 download_single_thread 和 download_segmented**
   - 接受 HttpClient 參數而非使用 self.http_client
   - 或：在內部使用動態建立的客戶端

---

## ⏳ 待實作項目

### T068: 優雅降級（估計 1-2 小時）

**需求**:
- 當所有 Proxy 都失效時，詢問使用者是否使用直接連線
- 需要前後端通訊

**實作建議**:
1. 在 download_with_proxy_retry 中檢測所有 Proxy 失效
2. 發送 Tauri 事件到前端：`all-proxies-failed`
3. 前端顯示對話框詢問使用者
4. 使用者選擇後，透過命令回應：`fallback-to-direct` 或 `cancel-download`
5. 後端根據回應繼續或取消下載

---

### T069-T071: Proxy 管理命令（估計 2-3 小時）

**待建立**: `src-tauri/src/commands/proxy.rs`

```rust
use crate::models::ProxyInfo;
use crate::proxy::pool::ProxyPool;
use std::sync::Arc;
use tauri::State;

// 應用程式狀態需要添加 ProxyPool
pub struct ProxyState {
    pub pool: Arc<ProxyPool>,
}

#[tauri::command]
pub async fn import_proxy_list(
    file_path: String,
    state: State<'_, ProxyState>,
) -> Result<usize, String> {
    // 1. 從檔案匯入 Proxy 列表
    let proxies = crate::proxy::cache::import_proxies_from_file(&file_path)?;

    // 2. 批次驗證
    let validated = crate::proxy::validator::validate_and_update_proxies(proxies).await;

    // 3. 添加到 Pool
    for proxy in &validated {
        state.pool.add_proxy(proxy.clone()).await;
    }

    // 4. 儲存到快取
    let all_proxies = state.pool.get_all_proxies().await;
    crate::proxy::cache::save_proxies_to_cache(&all_proxies)?;

    Ok(validated.len())
}

#[tauri::command]
pub async fn get_proxy_list(
    state: State<'_, ProxyState>,
) -> Result<Vec<ProxyInfo>, String> {
    Ok(state.pool.get_all_proxies().await)
}

#[tauri::command]
pub async fn remove_proxy(
    proxy_id: String,
    state: State<'_, ProxyState>,
) -> Result<(), String> {
    let uuid = uuid::Uuid::parse_str(&proxy_id)
        .map_err(|e| format!("Invalid UUID: {}", e))?;

    state.pool.remove_proxy(uuid).await?;

    // 更新快取
    let all_proxies = state.pool.get_all_proxies().await;
    crate::proxy::cache::save_proxies_to_cache(&all_proxies)?;

    Ok(())
}
```

**整合到 lib.rs**:
```rust
use proxy::pool::ProxyPool;

pub struct ProxyState {
    pub pool: Arc<ProxyPool>,
}

pub fn run() {
    let proxy_pool = Arc::new(ProxyPool::new());
    let proxy_state = ProxyState {
        pool: proxy_pool.clone(),
    };

    tauri::Builder::default()
        // ...
        .manage(proxy_state)
        .invoke_handler(tauri::generate_handler![
            // ... existing commands
            import_proxy_list,
            get_proxy_list,
            remove_proxy,
        ])
        // ...
}
```

---

### T072-T075: 前端 Proxy 管理 UI（估計 4-6 小時）

**T072: ProxySettings.tsx**
- Proxy 列表顯示（DataGrid）
- 匯入按鈕（開啟檔案選擇器）
- 刪除按鈕
- 狀態顯示（Active/Failed/Blacklisted）
- 健康度分數顯示
- 驗證按鈕

**T073: proxyStore.ts**
```typescript
import { atom } from 'jotai';
import { invoke } from '@tauri-apps/api/core';

export interface ProxyInfo {
  id: string;
  host: string;
  port: number;
  status: 'Unknown' | 'Active' | 'Failed' | 'Blacklisted';
  health_score: number;
  avg_latency_ms: number;
  successful_requests: number;
  failed_requests: number;
}

export const proxyListAtom = atom<ProxyInfo[]>([]);
export const isLoadingProxiesAtom = atom(false);

export async function loadProxies() {
  const proxies = await invoke<ProxyInfo[]>('get_proxy_list');
  return proxies;
}

export async function importProxies(filePath: string) {
  return await invoke<number>('import_proxy_list', { filePath });
}

export async function removeProxy(proxyId: string) {
  await invoke('remove_proxy', { proxyId });
}
```

**T074: SettingsPage.tsx**
- 建立設定頁面路由
- 整合 ProxySettings 組件
- 其他設定區塊（未來擴充）

**T075: DownloadItem.tsx**
- 顯示當前使用的 Proxy（如果有）
- 顯示位置：下載項目的資訊區域
- 格式：`📡 Proxy: 127.0.0.1:8080 (85% health)`

---

## 📊 測試統計

### 單元測試結果

```
✓ proxy::pool::tests (6/6 passed)
  - test_add_proxy
  - test_add_duplicate_proxy
  - test_mark_success
  - test_mark_failure_blacklist
  - test_get_best_proxy
  - test_get_active_proxies

✓ proxy::validator::tests (4/4 passed)
  - test_validate_single_invalid_proxy
  - test_validate_batch_multiple_proxies
  - test_validate_proxies_batch_chunking
  - test_validate_and_update_proxies

✓ proxy::health_scorer::tests (7/7 passed)
  - test_categorize_health
  - test_get_proxy_health_category
  - test_filter_by_min_health
  - test_calculate_health_stats
  - test_get_best_n_proxies
  - test_should_use_proxy
  - test_health_stats_empty

✓ proxy::cache::tests (6/6 passed)
  - test_parse_proxy_line
  - test_parse_proxy_line_invalid
  - test_save_and_load_proxies
  - test_import_with_comments_and_empty_lines
  - test_import_with_invalid_lines
  - test_load_nonexistent_cache

總計: 23/23 測試通過 ✅
```

### 編譯狀態

```
✅ cargo check: 通過（55 個未使用程式碼警告，符合預期）
✅ cargo build: 成功
✅ cargo test: 所有測試通過
```

---

## 🏗️ 架構設計

### 模組依賴關係

```
models/proxy_info.rs (基礎資料結構)
        ↓
proxy/pool.rs ←─── proxy/health_scorer.rs
        ↓                   ↓
proxy/validator.rs    proxy/cache.rs
        ↓                   ↓
download/client.rs ←────────┘
        ↓
download/engine.rs (整合層)
        ↓
commands/proxy.rs (API 層)
        ↓
前端 UI (React + Jotai)
```

### Proxy 使用流程

```
1. 初始化
   └→ 從 proxies.txt 載入快取
   └→ 建立 ProxyPool
   └→ 批次驗證 Proxy

2. 下載開始
   └→ Engine.get_proxy() 取得最佳 Proxy
   └→ HttpClient::with_proxy() 建立客戶端
   └→ 開始下載

3. 下載成功
   └→ pool.mark_success(proxy_id, latency)
   └→ 更新健康度分數

4. 下載失敗
   └→ pool.mark_failure(proxy_id)
   └→ 重試：取得新的 Proxy
   └→ 重試次數用盡：執行優雅降級

5. 優雅降級
   └→ 檢查所有 Proxy 狀態
   └→ 發送事件到前端
   └→ 等待使用者決定
   └→ 使用直接連線或取消下載
```

---

## 📝 後續步驟

### 短期（本週內）

1. **完成 T067**: 實作 Proxy 自動切換重試邏輯
   - [ ] 修改 download_file 流程
   - [ ] 實作 download_with_proxy_retry
   - [ ] 更新 download_single_thread 接受 HttpClient
   - [ ] 測試 Proxy 切換功能

2. **完成 T068**: 實作優雅降級
   - [ ] 新增 Tauri 事件：all-proxies-failed
   - [ ] 前端處理事件並顯示對話框
   - [ ] 實作回應命令
   - [ ] 測試降級流程

### 中期（下週）

3. **完成 T069-T071**: Proxy 管理命令
   - [ ] 建立 commands/proxy.rs
   - [ ] 實作三個命令
   - [ ] 整合到 lib.rs
   - [ ] 測試命令功能

4. **完成 T072-T075**: 前端 UI
   - [ ] ProxySettings 組件
   - [ ] proxyStore 狀態管理
   - [ ] SettingsPage 整合
   - [ ] DownloadItem 更新

### 長期（未來迭代）

5. **效能優化**
   - [ ] Proxy 池持久化（定期儲存健康度）
   - [ ] 背景驗證（定期重新驗證 Failed Proxy）
   - [ ] 智能選擇（根據目標網域選擇最佳 Proxy）

6. **進階功能**
   - [ ] 支援 SOCKS5 Proxy
   - [ ] 支援需要認證的 Proxy
   - [ ] Proxy 輪替策略（輪流、隨機、最佳）
   - [ ] Proxy 來源訂閱（自動從 URL 更新列表）

---

## 🎯 驗收標準

### Phase 6 完整驗收條件

- [x] T063: Proxy 池核心完成並測試
- [x] T064: Proxy 驗證器完成並測試
- [x] T065: 健康度評分算法完成並測試
- [x] T066: Proxy 快取完成並測試
- [ ] T067: Proxy 自動切換運作正常
- [ ] T068: 優雅降級運作正常
- [ ] T069-T071: Proxy 管理命令可用
- [ ] T072-T075: 前端 UI 完整且可用

### 整合測試場景

1. **場景 1: 匯入並使用 Proxy**
   - [ ] 匯入 10 個 Proxy 列表
   - [ ] 系統自動驗證
   - [ ] 下載時使用可用 Proxy
   - [ ] UI 顯示當前使用的 Proxy

2. **場景 2: Proxy 失敗切換**
   - [ ] 下載時第一個 Proxy 失敗
   - [ ] 自動切換到下一個 Proxy
   - [ ] 繼續下載不中斷
   - [ ] 失敗的 Proxy 被標記

3. **場景 3: 所有 Proxy 失效**
   - [ ] 所有 Proxy 均失敗
   - [ ] 前端顯示詢問對話框
   - [ ] 使用者選擇直接連線
   - [ ] 下載繼續進行

---

## 📚 文件連結

- [tasks.md](specs/001-core-downloader/tasks.md) - 完整任務清單
- [proxy/pool.rs](src-tauri/src/proxy/pool.rs) - Proxy 池實作
- [proxy/validator.rs](src-tauri/src/proxy/validator.rs) - 驗證器實作
- [proxy/health_scorer.rs](src-tauri/src/proxy/health_scorer.rs) - 健康度評分
- [proxy/cache.rs](src-tauri/src/proxy/cache.rs) - 快取實作
- [download/client.rs](src-tauri/src/download/client.rs) - HTTP 客戶端
- [download/engine.rs](src-tauri/src/download/engine.rs) - 下載引擎

---

## 📞 聯絡資訊

如有問題或建議，請在專案 issues 中討論。

**Last Updated**: 2025-11-28
**Author**: Claude Code
**Status**: Phase 6 Core Modules Complete ✅
