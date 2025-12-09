# 模組開發完整指南 (T157)

**Last Updated**: 2025-12-09
**Feature**: 004-site-module-integration

## 目錄

1. [開發環境設置](#開發環境設置)
2. [使用 TypeScript 型別定義](#使用-typescript-型別定義)
3. [VSCode 整合](#vscode-整合)
4. [API 參考](#api-參考)
5. [測試與驗證](#測試與驗證)
6. [最佳實踐](#最佳實踐)

---

## 開發環境設置

### 1. IDE 配置 (推薦：VSCode)

Zoo Downloader 提供完整的 TypeScript 型別定義和 VSCode 整合：

```bash
cd zoo-downloader/modules
# 所有模組自動獲得型別提示和自動完成
```

**自動提供**：
- ✅ TypeScript 型別定義 (`@types/zoo-downloader`)
- ✅ IntelliSense 自動完成
- ✅ JSDoc 文件提示
- ✅ 錯誤檢查
- ✅ Code Snippets

### 2. 建立新模組

使用 `_template/` 模板：

```bash
cp -r modules/_template modules/my-module
cd modules/my-module
```

或使用 code snippet（在 VSCode 中輸入 `zoo-module`）

---

## 使用 TypeScript 型別定義

### 型別定義文件位置

```
modules/@types/zoo-downloader/
├── index.d.ts          # 完整型別定義
└── package.json
```

### 在模組中啟用型別提示

**方法 1: JSDoc 參考型別**（推薦）

```javascript
/// <reference types="zoo-downloader" />

/**
 * @typedef {import('zoo-downloader').ModuleContext} Context
 * @typedef {import('zoo-downloader').ModuleDownloadInfo} DownloadInfo
 */

export class MyModule {
  /**
   * @param {string} url
   * @param {Context} context - Execution context with http, log, etc.
   * @returns {Promise<DownloadInfo>}
   */
  async extractDownloadInfo(url, context) {
    // context.http.  ← 自動完成提示
    // context.log.   ← 自動完成提示
  }
}
```

**方法 2: 完整 TypeScript 型別別名**

```javascript
/// <reference types="zoo-downloader" />

/** @type {import('zoo-downloader').SiteModule} */
export class MyModule {
  canHandle(url) { /* ... */ }
  async extractDownloadInfo(url, context) { /* ... */ }
}
```

### 可用型別別名

```typescript
// 主要介面
import('zoo-downloader').SiteModule          // 模組類別介面
import('zoo-downloader').ModuleContext       // 執行上下文
import('zoo-downloader').ModuleDownloadInfo  // 下載資訊返回值

// 資料結構
import('zoo-downloader').ModuleHttpResponse  // HTTP 回應
import('zoo-downloader').ModuleCredentials   // 認證資訊
import('zoo-downloader').ModuleCaptchaChallenge  // CAPTCHA 挑戰
```

---

## VSCode 整合

### Code Snippets

在 JavaScript 文件中輸入以下前綴：

| Prefix | 描述 | 展開結果 |
|--------|------|----------|
| `zoo-module` | 完整模組模板 | 含 canHandle + extractDownloadInfo |
| `zoo-http-get` | HTTP GET 請求 | `await context.http.get(url)` |
| `zoo-http-post` | HTTP POST 請求 | `await context.http.post(url, data)` |
| `zoo-login` | 登入流程範例 | 完整認證流程 |
| `zoo-captcha` | CAPTCHA 處理 | requestCaptcha + waitForSolution |

**示例**：輸入 `zoo-http-get` 然後按 Tab：

```javascript
const response = await context.http.get(url, {
  headers: {
    'User-Agent': 'Mozilla/5.0...'
  }
});
const html = response.body;
```

### 自動完成演示

當你輸入 `context.` 時，VSCode 會顯示：

```
context.
  ├─ http          (HttpClient)
  │  ├─ get()      - Send HTTP GET request
  │  ├─ post()     - Send HTTP POST request
  │  ├─ put()      - Send HTTP PUT request
  │  └─ delete()   - Send HTTP DELETE request
  ├─ log           (Logger)
  │  ├─ info()     - Log info message
  │  ├─ warn()     - Log warning
  │  └─ error()    - Log error
  ├─ captcha       (CaptchaClient)
  │  ├─ requestCaptcha() - Request CAPTCHA challenge
  │  └─ waitForSolution() - Wait for user solution
  └─ credentials   (CredentialManager)
     ├─ get()      - Get stored credentials
     └─ has()      - Check if credentials exist
```

---

## API 參考

### Context API

#### `context.http` - HTTP 客戶端

```javascript
/**
 * 發送 HTTP GET 請求
 * @param {string} url - 請求 URL
 * @param {object} [options] - 可選配置
 * @param {object} [options.headers] - HTTP headers
 * @param {number} [options.timeout] - 超時時間 (ms)
 * @returns {Promise<HttpResponse>}
 */
await context.http.get(url, { headers: {...} });

/**
 * 發送 HTTP POST 請求
 * @param {string} url
 * @param {object} data - POST body (自動 JSON.stringify)
 * @param {object} [options]
 * @returns {Promise<HttpResponse>}
 */
await context.http.post(url, { key: 'value' }, { headers: {...} });
```

**HttpResponse 結構**：
```javascript
{
  status: 200,           // HTTP 狀態碼
  statusText: 'OK',      // 狀態文本
  headers: {},           // 回應 headers (object)
  body: '...',           // 回應 body (string)
  url: 'https://...'     // 最終 URL (處理重定向後)
}
```

#### `context.log` - 日誌記錄

```javascript
context.log.info('Starting download extraction');
context.log.warn('Premium account expired, using free tier');
context.log.error('Failed to parse download URL', { url });
```

**日誌級別**：`debug` < `info` < `warn` < `error`

#### `context.captcha` - CAPTCHA 處理

```javascript
// 請求 CAPTCHA
const challenge = await context.captcha.requestCaptcha('recaptcha', {
  siteKey: 'your-site-key',
  pageUrl: url
});

// 等待使用者解決
const solution = await context.captcha.waitForSolution(challenge.id);
console.log('User solved:', solution.token);
```

#### `context.credentials` - 認證管理

```javascript
// 檢查是否有認證
if (await context.credentials.has(['username', 'password'])) {
  const creds = await context.credentials.get(['username', 'password']);
  console.log('Username:', creds.username);
  console.log('Password:', creds.password); // 從 OS keychain 安全讀取
}
```

**注意**: 認證資訊由主程式管理，模組不應自行儲存密碼。

---

## 測試與驗證 (T158)

### 模組驗證腳本

建立 `test-module.js` 來測試你的模組：

```javascript
/// <reference types="zoo-downloader" />

import { MySiteModule } from './index.js';

// 測試 URL 檢測
console.log('Testing canHandle...');
const module = new MySiteModule();
console.assert(module.canHandle('https://my-site.com/file/123'), 'Should handle my-site URLs');

// 模擬 context（用於本地測試）
const mockContext = {
  http: {
    get: async (url) => ({ status: 200, body: '<html>test</html>', headers: {} }),
    post: async (url, data) => ({ status: 200, body: '{}', headers: {} }),
  },
  log: {
    info: (...args) => console.log('[INFO]', ...args),
    warn: (...args) => console.warn('[WARN]', ...args),
    error: (...args) => console.error('[ERROR]', ...args),
  },
  captcha: {
    requestCaptcha: async () => ({ id: 'test', type: 'recaptcha' }),
    waitForSolution: async () => ({ token: 'test-token' }),
  },
  credentials: {
    has: async () => true,
    get: async () => ({ username: 'test', password: 'test' }),
  },
};

// 測試下載資訊提取
try {
  const info = await module.extractDownloadInfo('https://my-site.com/file/123', mockContext);
  console.log('✅ Download info:', info);
  console.assert(info.downloadUrl, 'Should have downloadUrl');
  console.assert(info.filename, 'Should have filename');
} catch (error) {
  console.error('❌ Test failed:', error);
}
```

**執行測試**：
```bash
node test-module.js
```

### Manifest 驗證

檢查 manifest.json 格式：

```bash
# 驗證 JSON 格式
cat manifest.json | node -e "console.log(JSON.parse(require('fs').readFileSync(0, 'utf-8')))"

# 驗證必要欄位
node -e "
const m = JSON.parse(require('fs').readFileSync('manifest.json'));
console.assert(m.id, 'Missing id');
console.assert(m.name, 'Missing name');
console.assert(m.version, 'Missing version');
console.assert(m.urlPatterns, 'Missing urlPatterns');
console.log('✅ Manifest valid');
"
```

---

## 最佳實踐

### 1. 錯誤處理

```javascript
async extractDownloadInfo(url, context) {
  try {
    const response = await context.http.get(url);

    if (response.status !== 200) {
      throw new Error(`HTTP ${response.status}: ${response.statusText}`);
    }

    const downloadUrl = this.parseDownloadUrl(response.body);
    if (!downloadUrl) {
      throw new Error('Failed to extract download URL from page');
    }

    return {
      downloadUrl,
      filename: this.parseFilename(response.body) || 'download',
      filesize: null,
      headers: {}
    };

  } catch (error) {
    context.log.error('Extract failed:', error.message);
    throw error; // 重新拋出，讓主程式處理
  }
}
```

### 2. 認證流程

```javascript
async login(context) {
  // 1. 檢查是否有認證
  if (!await context.credentials.has(['username', 'password'])) {
    throw new Error('AUTHENTICATION_REQUIRED'); // 特殊錯誤碼
  }

  // 2. 讀取認證
  const creds = await context.credentials.get(['username', 'password']);

  // 3. 登入
  const response = await context.http.post('https://api.site.com/login', {
    username: creds.username,
    password: creds.password
  });

  // 4. 儲存 session（在實例變數中，不要用全域變數）
  this.sessionToken = response.body.token;

  context.log.info('Login successful');
}
```

### 3. CAPTCHA 處理

```javascript
async handleCaptcha(pageUrl, siteKey, context) {
  context.log.info('CAPTCHA required');

  // 1. 請求 CAPTCHA
  const challenge = await context.captcha.requestCaptcha('recaptcha', {
    siteKey,
    pageUrl
  });

  // 2. 等待使用者解決（UI 會自動顯示）
  const solution = await context.captcha.waitForSolution(challenge.id);

  // 3. 使用解決方案
  return solution.token;
}
```

### 4. 效能優化

```javascript
// ✅ 好：快取重複請求
async getAccountInfo(context) {
  if (this.cachedAccountInfo) {
    return this.cachedAccountInfo;
  }

  const info = await context.http.get('/account');
  this.cachedAccountInfo = info;
  return info;
}

// ❌ 壞：每次都請求
async getAccountInfo(context) {
  return await context.http.get('/account');
}
```

### 5. 安全性

```javascript
// ✅ 好：URL 在 manifest 白名單中
{
  "permissions": {
    "network": {
      "allowDomains": ["my-site.com", "*.my-site.com"]
    }
  }
}

// ❌ 壞：請求任意 URL（會被拒絕）
await context.http.get('https://evil-site.com'); // PermissionDenied
```

---

## 常見問題

### Q: 如何調試模組？

A: 使用 `context.log` 並查看審計日誌：
```javascript
context.log.info('Current state:', { url, sessionToken: this.sessionToken });
```

### Q: 模組可以使用 npm 套件嗎？

A: 不行，模組必須是純 JavaScript（無外部依賴）。使用內建 API 或純 JS 實作。

### Q: 如何處理動態 JavaScript 網站？

A: 目前不支援 JavaScript 渲染。使用網站的 API 端點或 mobile API。

### Q: 模組會被快取嗎？

A: 是的，模組載入後會快取。重新載入模組需要重啟應用程式或使用「重新載入模組」功能。

---

## 參考資源

- **型別定義**: `modules/@types/zoo-downloader/index.d.ts`
- **範例模組**: `modules/k2s/` (Keep2Share 完整實作)
- **模板**: `modules/_template/`
- **快速開始**: `specs/004-site-module-integration/quickstart.md`

---

**Happy Module Development!** 🚀

如有問題，請在 GitHub Issues 中提出。
