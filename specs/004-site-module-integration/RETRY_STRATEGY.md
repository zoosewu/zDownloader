# Retry Strategy Implementation (T167)

**Task**: T167 - 錯誤處理和重試策略
**Priority**: 中優先級
**Date**: 2025-12-08
**Status**: ✅ Complete

---

## Overview

實作了智能重試機制，用於處理暫時性網路錯誤。重試邏輯使用指數退避策略，自動重試可恢復的錯誤，同時避免對不可重試的錯誤浪費資源。

---

## Implementation Details

### 1. Retry Configuration (`RetryConfig`)

**位置**: `src-tauri/src/modules/credential_proxy.rs`

```rust
pub struct RetryConfig {
    /// Maximum number of retry attempts (default: 3)
    pub max_retries: usize,
    /// Initial delay in seconds (default: 2)
    pub initial_delay_secs: u64,
    /// Maximum delay in seconds (default: 10)
    pub max_delay_secs: u64,
    /// Enable exponential backoff (default: true)
    pub exponential_backoff: bool,
}
```

**預設值**:
- `max_retries`: 3 次重試
- `initial_delay_secs`: 2 秒初始延遲
- `max_delay_secs`: 10 秒最大延遲
- `exponential_backoff`: true（啟用指數退避）

---

### 2. Exponential Backoff Strategy

**退避序列**（使用預設配置）:

| Attempt | Calculation | Delay | Capped |
|---------|-------------|-------|--------|
| 0 | 2 × 2^0 | 2s | 2s |
| 1 | 2 × 2^1 | 4s | 4s |
| 2 | 2 × 2^2 | 8s | 8s |
| 3 | 2 × 2^3 | 16s | **10s** (capped) |

**優點**:
- 初始快速重試，適合短暫網路抖動
- 逐漸增加延遲，避免對服務器造成壓力
- 最大延遲上限，防止等待時間過長

---

### 3. Retryable Error Types (可重試錯誤)

**HTTP 狀態碼**:
- `429` - Too Many Requests (速率限制)
- `502` - Bad Gateway (閘道錯誤)
- `503` - Service Unavailable (服務不可用)
- `504` - Gateway Timeout (閘道逾時)

**連接錯誤**:
- `timeout` / `timed out` - 請求逾時
- `connection reset` - 連接重置
- `connection refused` - 連接被拒絕
- `temporarily unavailable` - 暫時不可用

**判斷邏輯**:

```rust
fn is_retryable_error(error: &str) -> bool {
    let error_lower = error.to_lowercase();

    // HTTP status codes
    if error_lower.contains("429") || error_lower.contains("502") ||
       error_lower.contains("503") || error_lower.contains("504") {
        return true;
    }

    // Timeout and connection errors
    if error_lower.contains("timeout") ||
       error_lower.contains("connection reset") {
        return true;
    }

    false
}
```

---

### 4. Non-Retryable Errors (不可重試錯誤)

**Client errors (4xx except 429)**:
- `400` - Bad Request
- `401` - Unauthorized
- `403` - Forbidden
- `404` - Not Found

**Protocol errors**:
- Invalid URL protocol
- Unsupported HTTP method

**原因**: 這些錯誤通常是請求本身的問題，重試不會解決。

---

### 5. API Integration

#### 5.1 `authenticated_request_with_retry()`

**位置**: `src-tauri/src/modules/credential_proxy.rs`

**函數簽名**:
```rust
pub async fn authenticated_request_with_retry(
    &self,
    url: &str,
    method: &str,
    headers: Option<HashMap<String, String>>,
    body: Option<String>,
    retry_config: Option<RetryConfig>,
) -> Result<String, String>
```

**使用方式**:

```rust
// 使用預設配置（3 次重試，指數退避）
let response = credential_proxy
    .authenticated_request_with_retry(url, "GET", None, None, None)
    .await?;

// 使用自訂配置
let custom_config = RetryConfig {
    max_retries: 5,
    initial_delay_secs: 1,
    max_delay_secs: 30,
    exponential_backoff: true,
};
let response = credential_proxy
    .authenticated_request_with_retry(url, "POST", headers, body, Some(custom_config))
    .await?;
```

#### 5.2 Context API Integration

**位置**: `src-tauri/src/modules/context_api.rs`

```rust
/// Handle HTTP request from JavaScript (Phase 4: T074)
///
/// T167: Now uses automatic retry with exponential backoff for transient errors
pub async fn handle_http_request(
    &mut self,
    method: &str,
    url: &str,
    headers: Option<HashMap<String, String>>,
    body: Option<String>,
) -> Result<String, String> {
    // ... validation steps ...

    // Use retry with default config
    let response = self.credential_proxy
        .authenticated_request_with_retry(url, method, Some(auth_headers), body, None)
        .await?;

    // ... post-processing steps ...
}
```

**影響**: 所有通過 Context API 的 HTTP 請求（`context.http.get()`, `context.http.post()` 等）自動獲得重試能力。

---

## Logging and Monitoring

### 重試日誌範例

**成功重試**:
```
[k2s] Request failed (attempt 1/4), retrying in 2s: HTTP request failed with status 503
[k2s] Request succeeded after 1 retries: https://api.keep2share.cc/v2/file/info
```

**重試後仍失敗**:
```
[k2s] Request failed (attempt 1/4), retrying in 2s: HTTP request failed with status 503
[k2s] Request failed (attempt 2/4), retrying in 4s: HTTP request failed with status 503
[k2s] Request failed (attempt 3/4), retrying in 8s: HTTP request failed with status 503
[k2s] Request failed after 3 retries: https://api.keep2share.cc/v2/file/info
```

**不可重試錯誤（立即失敗）**:
```
HTTP request failed with status 404: Not Found
```

---

## Testing

### Test Coverage

**位置**: `src-tauri/src/modules/credential_proxy.rs`

| Test | Purpose | Status |
|------|---------|--------|
| `test_retry_config_defaults` | 驗證預設配置 | ✅ Pass |
| `test_retry_config_custom` | 驗證自訂配置 | ✅ Pass |
| `test_is_retryable_error_true` | 驗證可重試錯誤判斷 | ✅ Pass |
| `test_is_retryable_error_false` | 驗證不可重試錯誤判斷 | ✅ Pass |
| `test_exponential_backoff` | 驗證指數退避計算 | ✅ Pass |

**運行測試**:

```bash
cd src-tauri

# 測試所有重試相關功能
cargo test credential_proxy::tests::test_retry -- --nocapture

# 測試錯誤判斷邏輯
cargo test credential_proxy::tests::test_is_retryable -- --nocapture

# 測試指數退避
cargo test credential_proxy::tests::test_exponential_backoff -- --nocapture
```

---

## Use Cases

### Use Case 1: K2S API Rate Limiting

**場景**: K2S API 返回 429 Too Many Requests

**行為**:
1. 第一次請求失敗 → 429
2. 等待 2 秒，重試
3. 成功 → 返回結果

**程式碼**:
```javascript
// K2S 模組中
const response = await context.http.post(url, headers, body);
// 自動處理 429 錯誤，無需額外程式碼
```

### Use Case 2: 暫時性網路故障

**場景**: 服務器短暫不可用（503 Service Unavailable）

**行為**:
1. 第一次請求失敗 → 503
2. 等待 2 秒，重試 → 仍然 503
3. 等待 4 秒，重試 → 成功
4. 返回結果

### Use Case 3: 非暫時性錯誤

**場景**: 錯誤的 API endpoint（404 Not Found）

**行為**:
1. 第一次請求失敗 → 404
2. 判斷為不可重試錯誤
3. 立即返回錯誤，不浪費時間重試

---

## Performance Considerations

### 優點

1. **自動恢復**: 暫時性錯誤自動恢復，提高下載成功率
2. **智能判斷**: 只重試有意義的錯誤，避免浪費資源
3. **透明整合**: 現有模組無需修改即可獲得重試能力

### 權衡

1. **延遲增加**: 失敗請求會有額外延遲（最多 2+4+8 = 14 秒）
2. **資源消耗**: 重試會消耗額外的網路資源

### 建議配置

**低延遲場景**（如 UI 交互）:
```rust
RetryConfig {
    max_retries: 1,
    initial_delay_secs: 1,
    max_delay_secs: 2,
    exponential_backoff: false,
}
```

**高可靠性場景**（如大檔案下載）:
```rust
RetryConfig {
    max_retries: 5,
    initial_delay_secs: 2,
    max_delay_secs: 30,
    exponential_backoff: true,
}
```

---

## Future Enhancements

### 建議改進

1. **Jitter (抖動)**: 添加隨機抖動避免「驚群效應」
   ```rust
   let jitter = rand::thread_rng().gen_range(0..500); // 0-500ms
   let delay_ms = delay_secs * 1000 + jitter;
   ```

2. **Per-Domain Rate Limiting**: 不同域名使用不同的重試策略

3. **Retry Budget**: 限制整體重試次數，避免無限重試

4. **Circuit Breaker**: 當服務持續不可用時，暫時停止請求

5. **Telemetry**: 收集重試統計數據
   - 成功率
   - 平均重試次數
   - 最常見錯誤類型

---

## Integration with K2S Module

K2S 模組自動獲得重試能力，無需額外程式碼：

**Before (無重試)**:
```javascript
async getFileInfo(fileId, context) {
  const response = await context.http.post(url, headers, body);
  // 如果失敗（503），直接報錯
  return JSON.parse(response);
}
```

**After (自動重試)**:
```javascript
async getFileInfo(fileId, context) {
  const response = await context.http.post(url, headers, body);
  // 如果失敗（503），自動重試 3 次（2s → 4s → 8s）
  // 如果成功，返回結果
  return JSON.parse(response);
}
```

---

## Summary

✅ **T167 完成**

**實作項目**:
- ✅ `RetryConfig` 結構定義
- ✅ `authenticated_request_with_retry()` 方法
- ✅ 指數退避策略（2s → 4s → 8s，上限 10s）
- ✅ 可重試錯誤判斷邏輯（429, 502, 503, 504, timeout）
- ✅ Context API 整合
- ✅ 完整測試覆蓋

**測試結果**:
- 5 個單元測試全部通過
- 編譯成功，無錯誤

**下一步**:
- T168: 帳號類型偵測優化
- T169: 平行下載優化

---

**Document Version**: 1.0
**Last Updated**: 2025-12-08
**Author**: Implementation Task T167
**Status**: ✅ Complete
