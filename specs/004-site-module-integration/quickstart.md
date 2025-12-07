# Quick Start: 開發 Zoo Downloader 模組

**Date**: 2025-12-04
**Feature**: 004-site-module-integration
**Target Audience**: 模組開發者

## 目標讀者

本指南面向希望為 Zoo Downloader 開發網站支援模組的開發者。你需要具備：
- JavaScript/TypeScript 基礎知識
- 了解 HTTP 請求和 Web scraping
- 熟悉 JSON 格式

不需要：
- Rust 知識（模組使用 JavaScript 編寫）
- 編譯或構建工具（模組是純 JavaScript）

---

## 5 分鐘快速開始

### 步驟 1：建立模組目錄

```bash
cd zoo-downloader/modules
mkdir my-site-module
cd my-site-module
```

### 步驟 2：建立 manifest.json

```json
{
  "id": "my-site",
  "name": "My Site Downloader",
  "version": "1.0.0",
  "author": "Your Name",
  "description": "Download files from my-site.com",
  "homepage": "https://github.com/yourname/my-site-module",
  "urlPatterns": [
    "https://my-site.com/file/*",
    "https://*.my-site.com/download/*"
  ],
  "permissions": {
    "network": {
      "allowDomains": [
        "my-site.com",
        "*.my-site.com"
      ]
    }
  }
}
```

### 步驟 3：建立 index.js

```javascript
// index.js - 模組主程式
export class MySiteModule {
  /**
   * 檢查此模組是否能處理給定的 URL
   * @param {string} url - 要檢查的 URL
   * @returns {boolean} 是否支援此 URL
   */
  canHandle(url) {
    return url.includes('my-site.com/file/') ||
           url.includes('my-site.com/download/');
  }

  /**
   * 從 URL 解析真實的下載連結
   * @param {string} url - 分享連結
   * @param {object} context - 執行上下文（由主程式提供）
   * @returns {Promise<object>} 下載資訊
   */
  async extractDownloadInfo(url, context) {
    // 發送 HTTP 請求
    const response = await context.http.get(url);

    // 解析 HTML
    const html = response.body;
    const downloadUrl = this.parseDownloadUrl(html);
    const filename = this.parseFilename(html);

    return {
      downloadUrl: downloadUrl,
      filename: filename,
      filesize: null,  // 可選
      headers: {}      // 額外的 HTTP headers
    };
  }

  /**
   * 從 HTML 中解析下載連結
   */
  parseDownloadUrl(html) {
    // 使用正則表達式或 DOM 解析
    const match = html.match(/data-download="(.+?)"/);
    return match ? match[1] : null;
  }

  /**
   * 從 HTML 中解析檔案名稱
   */
  parseFilename(html) {
    const match = html.match(/<h1>(.+?)<\/h1>/);
    return match ? match[1] : 'download';
  }
}

// 匯出模組類別（必須）
export default MySiteModule;
```

### 步驟 4：測試模組

1. 重新啟動 Zoo Downloader
2. 開啟設定 → 網站模組管理
3. 確認你的模組出現在列表中且狀態為 "Active"
4. 嘗試貼上一個你網站的連結

**完成！** 🎉 你已經建立了第一個模組。

---

## 模組結構深入

### 必須的檔案

```
my-site-module/
├── manifest.json   # 模組元資訊和權限
├── index.js        # 模組主程式
└── README.md       # 文件（建議）
```

### manifest.json 完整範例

```json
{
  "id": "k2s",
  "name": "Keep2Share Downloader",
  "version": "1.0.0",
  "author": "Zoo Downloader Community",
  "description": "支援 Keep2Share (k2s.cc) 的檔案下載，包含自動認證和驗證碼處理",
  "homepage": "https://github.com/zoo-downloader/modules/k2s",
  "urlPatterns": [
    "https://k2s.cc/file/*",
    "https://*.k2s.cc/file/*",
    "https://keep2share.cc/file/*"
  ],
  "permissions": {
    "network": {
      "allowDomains": [
        "k2s.cc",
        "*.k2s.cc",
        "keep2share.cc",
        "api.keep2share.cc"
      ]
    },
    "credentials": {
      "required": true,
      "credentialKeys": ["username", "password", "apiKey"]
    }
  }
}
```

### 模組類別完整範例

```javascript
export class MySiteModule {
  constructor() {
    this.session = null;
  }

  /**
   * URL 匹配檢查
   * @param {string} url
   * @returns {boolean}
   */
  canHandle(url) {
    return url.includes('my-site.com/file/');
  }

  /**
   * 主要下載資訊解析函數
   * @param {string} url - 分享連結
   * @param {object} context - 執行上下文
   * @returns {Promise<object>} 下載資訊
   * @throws {Error} 解析失敗時拋出錯誤
   */
  async extractDownloadInfo(url, context) {
    try {
      // 1. 檢查是否需要登入
      if (context.credentials) {
        await this.login(context);
      }

      // 2. 取得頁面內容
      const response = await context.http.get(url);

      // 3. 檢查是否需要驗證碼
      if (this.needsCaptcha(response.body)) {
        const captchaSolution = await context.captcha.request({
          type: 'image',
          imageUrl: this.extractCaptchaUrl(response.body)
        });

        // 重新提交帶有驗證碼的請求
        return await this.extractWithCaptcha(url, captchaSolution, context);
      }

      // 4. 解析下載資訊
      return {
        downloadUrl: this.parseDownloadUrl(response.body),
        filename: this.parseFilename(response.body),
        filesize: this.parseFilesize(response.body),
        headers: {
          'Referer': url,
          'User-Agent': 'Zoo-Downloader/1.0'
        }
      };
    } catch (error) {
      throw new Error(`Failed to extract download info: ${error.message}`);
    }
  }

  /**
   * 登入處理
   * @param {object} context
   */
  async login(context) {
    if (this.session) {
      return; // 已登入
    }

    const loginData = {
      username: context.credentials.username,
      password: context.credentials.password
    };

    const response = await context.http.post(
      'https://my-site.com/login',
      {
        body: JSON.stringify(loginData),
        headers: {
          'Content-Type': 'application/json'
        }
      }
    );

    if (response.status !== 200) {
      throw new Error('Login failed');
    }

    this.session = response.cookies.session;
  }

  /**
   * 檢查是否需要驗證碼
   */
  needsCaptcha(html) {
    return html.includes('data-captcha');
  }

  /**
   * 解析驗證碼圖片 URL
   */
  extractCaptchaUrl(html) {
    const match = html.match(/data-captcha="(.+?)"/);
    return match ? match[1] : null;
  }

  /**
   * 帶驗證碼重新提交
   */
  async extractWithCaptcha(url, captchaSolution, context) {
    const response = await context.http.post(url, {
      body: JSON.stringify({
        captcha: captchaSolution.solution
      })
    });

    return {
      downloadUrl: this.parseDownloadUrl(response.body),
      filename: this.parseFilename(response.body),
      filesize: null,
      headers: {}
    };
  }

  // 輔助解析函數
  parseDownloadUrl(html) {
    const match = html.match(/href="(https:\/\/cdn\.my-site\.com\/[^"]+)"/);
    return match ? match[1] : null;
  }

  parseFilename(html) {
    const match = html.match(/<title>(.+?) - Download<\/title>/);
    return match ? match[1] : 'download';
  }

  parseFilesize(html) {
    const match = html.match(/File size: ([\d.]+) (MB|GB)/);
    if (!match) return null;

    const size = parseFloat(match[1]);
    const unit = match[2];
    return unit === 'GB' ? size * 1024 * 1024 * 1024 : size * 1024 * 1024;
  }
}

export default MySiteModule;
```

---

## Context API 參考

模組執行時會收到一個 `context` 物件，提供以下 API：

### 1. HTTP 請求 (context.http)

```javascript
// GET 請求
const response = await context.http.get(url, options);

// POST 請求
const response = await context.http.post(url, {
  body: JSON.stringify(data),
  headers: {
    'Content-Type': 'application/json'
  }
});

// Response 物件
{
  status: 200,
  body: "HTML content",
  headers: { ... },
  cookies: { ... }
}
```

**限制**：
- 只能存取 manifest 中聲明的網域
- 每分鐘最多 60 個請求
- 單次回應最大 10MB

### 2. 認證存取 (context.credentials)

```javascript
// 只有 manifest 中 credentials.required = true 時可用
if (context.credentials) {
  const username = context.credentials.username;
  const password = context.credentials.password;
  const apiKey = context.credentials.apiKey;  // 如果在 credentialKeys 中聲明
}
```

**重要**：模組永遠無法直接存取原始密碼，只能透過 `context.http` 發送認證請求。

### 3. 驗證碼處理 (context.captcha)

```javascript
// 請求使用者輸入驗證碼
const solution = await context.captcha.request({
  type: 'image',           // 或 'recaptcha', 'hcaptcha'
  imageUrl: captchaUrl,    // 驗證碼圖片 URL
  siteKey: null            // reCAPTCHA/hCaptcha 的 site key
});

// solution.solution 包含使用者輸入
const userInput = solution.solution;
```

### 4. 日誌記錄 (context.log)

```javascript
context.log.info('Processing URL:', url);
context.log.warn('Approaching rate limit');
context.log.error('Failed to parse:', error);
```

---

## 測試與除錯

### 本地測試

1. **使用瀏覽器開發者工具**
   - 打開目標網站
   - 觀察 Network tab 找出 API endpoints
   - 複製 HTTP requests 到模組

2. **使用 console.log**
   ```javascript
   context.log.info('Response body:', response.body);
   ```

3. **查看審計日誌**
   - 設定 → 網站模組管理 → 選擇模組 → 審計日誌

### 常見錯誤

#### 錯誤：`"Permission denied: domain not in whitelist"`
**原因**：嘗試存取未在 manifest 中聲明的網域

**解決**：
```json
{
  "permissions": {
    "network": {
      "allowDomains": [
        "my-site.com",
        "cdn.my-site.com"  // 新增此行
      ]
    }
  }
}
```

#### 錯誤：`"Rate limit exceeded"`
**原因**：超過每分鐘 60 個請求限制

**解決**：添加延遲
```javascript
await context.sleep(1000); // 延遲 1 秒
```

#### 錯誤：`"Execution timeout"`
**原因**：模組執行時間超過 5 秒

**解決**：優化程式碼，減少不必要的請求

---

## 進階功能

### 1. 使用通配符匹配多個網域

```json
{
  "permissions": {
    "network": {
      "allowDomains": [
        "*.example.com",  // 匹配 api.example.com, cdn.example.com 等
        "example.*"       // 匹配 example.com, example.net 等
      ]
    }
  }
}
```

### 2. 處理重定向

```javascript
const response = await context.http.get(url, {
  followRedirects: true,  // 自動跟隨重定向
  maxRedirects: 5
});
```

### 3. Cookie 管理

```javascript
// Cookies 會自動儲存在 session 中
const loginResponse = await context.http.post(loginUrl, { ... });
// 後續請求會自動帶上 cookies

// 如需手動設定
await context.http.get(url, {
  headers: {
    'Cookie': 'session=abc123'
  }
});
```

### 4. 錯誤處理最佳實踐

```javascript
async extractDownloadInfo(url, context) {
  try {
    const response = await context.http.get(url);

    if (response.status === 404) {
      throw new Error('File not found');
    }

    if (response.status === 403) {
      throw new Error('Access denied. Login may be required.');
    }

    const downloadUrl = this.parseDownloadUrl(response.body);
    if (!downloadUrl) {
      throw new Error('Failed to parse download URL. Website structure may have changed.');
    }

    return { downloadUrl, filename: '...', filesize: null, headers: {} };
  } catch (error) {
    // 提供清晰的錯誤訊息
    throw new Error(`Failed to extract download info: ${error.message}`);
  }
}
```

---

## 發布模組

### 1. 準備發布

- ✅ 測試所有功能
- ✅ 撰寫完整的 README.md
- ✅ 確認 manifest.json 正確
- ✅ 使用語義化版本號 (semver)

### 2. 分享模組

建議將模組發布到 GitHub：

```bash
git init
git add .
git commit -m "Initial commit"
git remote add origin https://github.com/yourname/my-site-module.git
git push -u origin main
```

### 3. 提交到社群模組庫（未來功能）

未來將支援社群模組庫，讓使用者直接從應用內安裝模組。

---

## 安全最佳實踐

1. **最小權限原則**
   - 只聲明必要的網域
   - 避免使用過於寬鬆的通配符

2. **輸入驗證**
   - 驗證所有使用者輸入
   - 避免注入攻擊

3. **錯誤處理**
   - 不洩露敏感資訊到錯誤訊息
   - 不記錄密碼或 API keys

4. **資料隱私**
   - 不收集使用者資料
   - 不發送資料到第三方服務（除非在 manifest 中聲明）

---

## 取得幫助

- **文件**：查看 `data-model.md` 和 `tauri-commands.md`
- **範例**：參考 `modules/k2s/` 目錄
- **社群**：在 GitHub Discussions 提問
- **錯誤回報**：在 GitHub Issues 報告 bug

---

## 下一步

1. 閱讀 [data-model.md](./data-model.md) 了解完整的資料結構
2. 閱讀 [research.md](./research.md) 了解技術選擇
3. 參考 K2S 模組的完整實作範例
4. 加入社群討論模組開發

Happy coding! 🚀
