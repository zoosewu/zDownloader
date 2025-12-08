# K2S 實作分析與改進建議

> 基於對 lu8787ouo/k2s-Downloaderm 和官方 Keep2Share API 的研究
> 日期：2025-12-07
> Feature: 004 - Site Module Integration

## 研究來源

1. **lu8787ouo/k2s-Downloaderm** - Python 實作，具備平行下載
2. **Keep2Share 官方 API v2 文件** - 官方 REST API 規格
3. **keep2share-js** - Node.js 簡化包裝函式庫

---

## 1. CAPTCHA 處理流程差異

### 當前實作（modules/k2s/index.js:268-302）

```javascript
// 我們的實作
if (data.code === 'captcha_required' && data.captcha_url) {
  const captchaSolution = await this.handleCaptcha(data.captcha_url, context);

  const retryPayload = JSON.stringify({
    file_id: fileId,
    free_download: this.accountType === 'free' ? 1 : 0,
    captcha_response: captchaSolution  // ← 直接使用解答
  });
}
```

**問題**：
- 使用非標準的 `captcha_required` 錯誤碼
- 假設 CAPTCHA URL 直接在錯誤回應中提供
- 參數命名可能不符合官方 API

### 官方 API 流程（基於文件）

```javascript
// 官方流程
if (response.code === 406 && response.errorCode === 33) {
  // Step 1: 請求 ReCAPTCHA challenge
  const challengeResponse = await context.http.post(
    `${this.apiBaseUrl}/requestReCaptcha`,
    {
      'Authorization': `Bearer ${this.session}`,
      'Content-Type': 'application/json'
    },
    JSON.stringify({ file_id: fileId })
  );

  const challenge = JSON.parse(challengeResponse);

  // Step 2: 顯示 ReCAPTCHA 給用戶並獲取解答
  const solution = await context.captcha.request(
    null,  // 使用 challenge.siteKey 而非圖片 URL
    challenge.siteKey
  );

  // Step 3: 提交解答
  const verifyResponse = await context.http.post(
    `${this.apiBaseUrl}/reCaptcha`,
    {
      'Authorization': `Bearer ${this.session}`,
      'Content-Type': 'application/json'
    },
    JSON.stringify({
      re_captcha_challenge: challenge.challenge,
      re_captcha_response: solution
    })
  );

  // Step 4: 重試原始請求（CAPTCHA 已在會話中驗證）
  const retryResponse = await context.http.post(downloadUrl, ...);
}
```

**優勢**：
- 符合官方 ReCAPTCHA v2 流程
- 正確處理 challenge/response 機制
- 使用標準化的錯誤碼（HTTP 406 + errorCode 33）

### 建議改進

**優先級：中**

**理由**：當前實作可能可以工作（如果 K2S 同時支援兩種流程），但使用官方流程更可靠且未來兼容。

**實作建議**：
1. 保留當前流程作為 fallback
2. 優先嘗試官方 ReCAPTCHA v2 流程
3. 更新 `context.captcha.request()` 簽名以支持 siteKey

```javascript
// 建議的雙重流程
async handleCaptchaForDownload(fileId, response, context) {
  // 檢查官方 ReCAPTCHA v2 錯誤碼
  if (response.code === 406 && response.errorCode === 33) {
    return await this.handleReCaptchaV2(fileId, context);
  }

  // Fallback：檢查舊式 CAPTCHA
  if (response.code === 'captcha_required' && response.captcha_url) {
    return await this.handleLegacyCaptcha(response.captcha_url, context);
  }

  throw new Error(`Unexpected CAPTCHA response: ${JSON.stringify(response)}`);
}
```

---

## 2. API 端點驗證

### 當前實作的端點

```javascript
// modules/k2s/index.js
this.apiBaseUrl = 'https://api.keep2share.cc/v2';

// 使用的端點：
// POST /v2/login
// POST /v2/file/info
// POST /v2/file/download
// GET  /v2/account/info
```

### 官方 API v2 端點（基於文件）

```
POST /v2/login                    ✅ 相同
POST /v2/getFilesInfo             ❓ 我們使用 /file/info
POST /v2/getUrl                   ❓ 我們使用 /file/download
GET  /v2/accountInfo              ❓ 我們使用 /account/info
POST /v2/requestReCaptcha         ❌ 我們未實作
POST /v2/reCaptcha                ❌ 我們未實作
```

**潛在問題**：
- `/file/info` vs `/getFilesInfo` - 可能是不同的 API 版本
- `/file/download` vs `/getUrl` - 端點名稱差異
- `/account/info` vs `/accountInfo` - 可能兩者都有效

### 建議驗證

**優先級：高**

**行動項目**：
1. 測試當前端點是否仍然有效
2. 如果失敗，切換到官方端點名稱：
   ```javascript
   // 更新端點映射
   const ENDPOINTS = {
     LOGIN: '/v2/login',
     FILE_INFO: '/v2/getFilesInfo',      // 改為官方名稱
     DOWNLOAD: '/v2/getUrl',              // 改為官方名稱
     ACCOUNT: '/v2/accountInfo',          // 改為官方名稱
     REQUEST_CAPTCHA: '/v2/requestReCaptcha',
     VERIFY_CAPTCHA: '/v2/reCaptcha'
   };
   ```

3. 更新請求 payload 格式（如果端點變更）：
   ```javascript
   // getFilesInfo 可能使用不同的參數格式
   {
     "ids": ["file_id_here"]  // 注意：複數形式，支持批量查詢
   }
   ```

---

## 3. 平行下載優化（來自 lu8787ouo/k2s-Downloaderm）

### Python 實作的關鍵特性

```python
# 來自 k2s-Downloaderm
CONCURRENT_LIMIT = 20      # 平行線程數
CHUNK_SIZE = 20 * 1024 * 1024  # 20MB 區塊

# 使用 HTTP Range requests
headers = {
    'Range': f'bytes={start}-{end}'
}
```

**優勢**：
- 對於大檔案（> 100MB）速度提升顯著
- 繞過單連接速度限制（特別是免費帳號）
- 支持斷點續傳

### 當前實作狀態

我們的下載引擎已經支持基本的 Range requests（`src-tauri/src/download/resume.rs`），但：
- 未實作平行區塊下載
- 未針對 K2S 優化區塊大小
- 未利用 K2S 的多連接容忍度

### 建議改進

**優先級：低**（但對用戶體驗影響大）

**實作位置**：`src-tauri/src/download/engine.rs`（非模組層級）

**建議**：
1. 在 DownloadEngine 中實作平行 range request 支持
2. K2S 模組只需在 metadata 中提示支持平行下載：
   ```javascript
   // modules/k2s/index.js
   return {
     downloadUrl: downloadLink.url,
     filename: fileInfo.name,
     filesize: fileInfo.size,
     // 新增：提示下載引擎使用平行下載
     downloadHints: {
       supportsRangeRequests: true,
       optimalChunkSize: 20 * 1024 * 1024,  // 20MB
       maxConcurrentConnections: this.accountType === 'premium' ? 20 : 5
     }
   };
   ```

3. 下載引擎根據 hints 調整策略

**注意**：這是下載引擎層級的改進，不應該在 JavaScript 模組中實作（安全性和性能考量）。

---

## 4. 帳號類型偵測優化

### 當前實作（modules/k2s/index.js:108-139）

```javascript
async detectAccountType(context) {
  const accountUrl = `${this.apiBaseUrl}/account/info`;
  const response = await context.http.get(accountUrl, {
    'Authorization': `Bearer ${this.session}`
  });

  const data = JSON.parse(response);
  this.accountType = data.account.is_premium ? 'premium' : 'free';
}
```

### lu8787ouo/k2s-Downloaderm 的處理

```python
# Python 版本更詳細地檢查帳號狀態
account_info = response['account']
is_premium = account_info.get('account_type') == 'premium'
premium_expired = account_info.get('premium_expired', False)

# 處理過期的 premium 帳號
if is_premium and premium_expired:
    account_type = 'free'  # 降級為免費帳號
else:
    account_type = 'premium' if is_premium else 'free'
```

### 建議改進

**優先級：低**

**改進點**：
```javascript
async detectAccountType(context) {
  // ...
  const data = JSON.parse(response);

  // 更精確的帳號類型檢測
  const account = data.account;
  const isPremium = account.account_type === 'premium' || account.is_premium;
  const premiumExpired = account.premium_expired || false;

  // 處理過期的 premium 帳號
  if (isPremium && premiumExpired) {
    context.log.warn('Premium account has expired, treating as free account');
    this.accountType = 'free';
  } else {
    this.accountType = isPremium ? 'premium' : 'free';
  }

  // 記錄額外資訊（用於調試）
  context.log.info(`Account type: ${this.accountType}, premium_until: ${account.premium_until || 'N/A'}`);

  return this.accountType;
}
```

---

## 5. 錯誤處理和重試策略

### lu8787ouo/k2s-Downloaderm 的錯誤處理

```python
# 智能重試策略
MAX_RETRIES = 3
RETRY_DELAYS = [2, 5, 10]  # 指數退避

for attempt in range(MAX_RETRIES):
    try:
        response = make_request()
        break
    except RateLimitError:
        if attempt < MAX_RETRIES - 1:
            time.sleep(RETRY_DELAYS[attempt])
        else:
            raise
```

### 當前實作

我們的 NetworkMonitor 已有基本的 rate limiting，但：
- 未實作自動重試
- 未實作指數退避
- 錯誤訊息可能不夠明確

### 建議改進

**優先級：中**

**實作位置**：`src-tauri/src/modules/network_monitor.rs`

```rust
// 在 NetworkMonitor 中添加重試邏輯
pub struct RetryConfig {
    pub max_retries: u32,
    pub initial_delay_ms: u64,
    pub max_delay_ms: u64,
    pub backoff_multiplier: f64,
}

impl NetworkMonitor {
    pub async fn request_with_retry<F, T>(
        &mut self,
        request_fn: F,
        retry_config: &RetryConfig,
    ) -> Result<T, String>
    where
        F: Fn() -> Future<Output = Result<T, String>>,
    {
        let mut delay = retry_config.initial_delay_ms;

        for attempt in 0..retry_config.max_retries {
            match request_fn().await {
                Ok(result) => return Ok(result),
                Err(e) if Self::is_retryable(&e) => {
                    if attempt < retry_config.max_retries - 1 {
                        tokio::time::sleep(Duration::from_millis(delay)).await;
                        delay = (delay as f64 * retry_config.backoff_multiplier) as u64;
                        delay = delay.min(retry_config.max_delay_ms);
                    } else {
                        return Err(e);
                    }
                }
                Err(e) => return Err(e),
            }
        }

        Err("Max retries exceeded".to_string())
    }

    fn is_retryable(error: &str) -> bool {
        error.contains("Rate limit") ||
        error.contains("timeout") ||
        error.contains("502") ||
        error.contains("503")
    }
}
```

---

## 6. 媒體驗證（ffmpeg 整合）

### lu8787ouo/k2s-Downloaderm 的特性

```python
# 下載完成後驗證媒體檔案
import subprocess

def verify_media_file(filepath):
    """使用 ffmpeg 驗證媒體檔案完整性"""
    try:
        subprocess.run(
            ['ffmpeg', '-v', 'error', '-i', filepath, '-f', 'null', '-'],
            check=True,
            capture_output=True
        )
        return True
    except subprocess.CalledProcessError:
        return False
```

### 建議

**優先級：極低**（nice-to-have）

**理由**：
- 需要 ffmpeg 二進制文件依賴
- 增加複雜度
- 不是所有下載都是媒體檔案

**如果實作**：
- 作為可選的後處理步驟
- 在 Tauri 層級實作，不在模組中
- 提供清楚的使用者設定選項

---

## 優先級總結

### 🔴 高優先級（應立即處理）

1. **API 端點驗證** - 確保使用正確的官方端點
   - 測試 `/file/info` vs `/getFilesInfo`
   - 測試 `/file/download` vs `/getUrl`
   - 更新端點如果需要

### 🟡 中優先級（Phase 4 完成後）

2. **CAPTCHA 流程更新** - 實作官方 ReCAPTCHA v2 流程
   - 添加 `/requestReCaptcha` 和 `/reCaptcha` 支持
   - 保留當前流程作為 fallback
   - 更新 Context API 的 captcha.request() 簽名

3. **錯誤處理強化** - 添加智能重試策略
   - 實作指數退避
   - 改進錯誤訊息
   - 添加 rate limit 友好處理

### 🟢 低優先級（Phase 5 或更晚）

4. **平行下載優化** - 下載引擎層級的改進
   - 實作平行 range requests
   - 針對 K2S 優化區塊大小
   - 添加 downloadHints 機制

5. **帳號類型偵測優化** - 處理過期的 premium 帳號
   - 檢查 premium_expired 標誌
   - 記錄 premium_until 日期

6. **媒體驗證** - ffmpeg 整合（可選）
   - 僅對媒體檔案
   - 作為可選功能
   - 需要外部依賴

---

## 測試建議

### 需要驗證的場景

1. **API 端點兼容性**
   ```bash
   # 手動測試當前端點
   curl -X POST https://api.keep2share.cc/v2/file/info \
     -H "Authorization: Bearer <token>" \
     -d '{"file_id": "test123"}'

   # 測試官方端點
   curl -X POST https://api.keep2share.cc/v2/getFilesInfo \
     -H "Authorization: Bearer <token>" \
     -d '{"ids": ["test123"]}'
   ```

2. **CAPTCHA 流程**
   - 測試免費帳號下載（通常需要 CAPTCHA）
   - 驗證 ReCAPTCHA v2 流程
   - 測試 CAPTCHA 失敗後重試

3. **帳號類型偵測**
   - 測試免費帳號
   - 測試有效 premium 帳號
   - 測試過期 premium 帳號

---

## 結論

**當前實作（modules/k2s/index.js）整體品質良好**，已經涵蓋了 K2S 下載的核心流程。

**主要改進方向**：
1. 確保 API 端點符合官方規格（最重要）
2. 實作完整的 ReCAPTCHA v2 流程（提高可靠性）
3. 添加智能重試策略（改善用戶體驗）
4. 考慮平行下載優化（性能提升，但需要下載引擎支持）

**不需要改進的部分**：
- URL 解析邏輯 ✅
- 基本認證流程 ✅
- 錯誤處理框架 ✅
- 模組結構設計 ✅

**建議的實作順序**：
1. 先完成 Phase 4.6 的剩余任務（如果有）
2. 驗證並修正 API 端點（高優先級）
3. 更新 CAPTCHA 流程（中優先級）
4. 其他優化可以在後續版本中迭代

---

**文件版本**：1.0
**最後更新**：2025-12-07
**作者**：Claude Code
**相關 Tasks**：T078-T084 (Phase 4.5), 未來的 Phase 5 優化
