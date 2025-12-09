# 模組開發實戰教學

**學習目標**: 從零開始建立一個完整的 Zoo Downloader 網站模組

**所需時間**: 約 30 分鐘

**先決條件**:
- 基本的 JavaScript 知識
- 了解 async/await 語法
- 安裝了 VS Code（推薦）

---

## 教學概覽

在本教學中，我們將建立一個簡單但完整的模組，用於處理虛構的檔案分享網站 "ExampleShare"。

### 我們將學習：

1. ✅ 建立模組基本結構
2. ✅ 配置 manifest.json
3. ✅ 實作 URL 偵測
4. ✅ 提取下載資訊
5. ✅ 處理錯誤和日誌
6. ✅ 測試模組
7. ✅ 使用 TypeScript 型別提示

---

## 第一部分：建立專案結構

### 步驟 1：建立模組目錄

```bash
cd modules
mkdir example-share
cd example-share
```

### 步驟 2：使用 VS Code 開啟

```bash
code .
```

**提示**: 開啟後，VS Code 會自動偵測到 `modules/@types/zoo-downloader` 的型別定義。

---

## 第二部分：建立 Manifest

### 步驟 3：建立 `manifest.json`

在 VS Code 中，輸入 `zoo-manifest` 然後按 Tab，會自動生成模板。

手動建立也很簡單：

```json
{
  "id": "example-share",
  "name": "ExampleShare Downloader",
  "version": "1.0.0",
  "description": "Download files from ExampleShare.com",
  "author": "Your Name",
  "urlPatterns": [
    "https://exampleshare.com/file/*",
    "https://exampleshare.com/d/*"
  ],
  "permissions": {
    "network": {
      "allowDomains": [
        "exampleshare.com",
        "cdn.exampleshare.com"
      ]
    }
  }
}
```

### 理解 Manifest 配置

- **id**: 模組的唯一識別符（必須是小寫、連字號）
- **urlPatterns**: 定義模組可以處理的 URL 格式
  - `*` 匹配任意字元
  - 範例中的模式會匹配：
    - `https://exampleshare.com/file/abc123`
    - `https://exampleshare.com/d/xyz789`
- **permissions.network.allowDomains**: 白名單制，只能訪問這些域名

---

## 第三部分：實作模組邏輯

### 步驟 4：建立 `index.js`

在 VS Code 中，輸入 `zoo-module` 然後按 Tab。

**完整程式碼**：

```javascript
/// <reference types="../@types/zoo-downloader" />

/**
 * ExampleShare 下載模組
 * @implements {import('zoo-downloader').SiteModule}
 */
export class ExampleShareModule {
  /**
   * 檢查 URL 是否由此模組處理
   * @param {string} url - 要檢查的 URL
   * @returns {boolean}
   */
  canHandle(url) {
    // 檢查是否匹配我們的 URL 模式
    return url.includes('exampleshare.com/file/') ||
           url.includes('exampleshare.com/d/');
  }

  /**
   * 提取下載資訊
   * @param {string} url - 檔案頁面 URL
   * @param {import('zoo-downloader').ModuleContext} context - 模組上下文
   * @returns {Promise<import('zoo-downloader').DownloadInfo>}
   */
  async extractDownloadInfo(url, context) {
    context.log.info(`開始處理 URL: ${url}`);

    try {
      // 步驟 1: 獲取檔案頁面
      context.log.debug('正在獲取檔案頁面...');
      const pageResponse = await context.http.get(url);

      if (!pageResponse.ok) {
        throw new Error(`無法訪問頁面: HTTP ${pageResponse.status}`);
      }

      // 步驟 2: 解析頁面內容
      const html = pageResponse.body;

      // 提取檔案名稱（使用正則表達式）
      const filenameMatch = html.match(/<h1 class="filename">(.+?)<\/h1>/);
      if (!filenameMatch) {
        throw new Error('無法找到檔案名稱');
      }
      const filename = filenameMatch[1];

      // 提取檔案大小（可選）
      const sizeMatch = html.match(/檔案大小:\s*(\d+)\s*MB/);
      const filesize = sizeMatch ? parseInt(sizeMatch[1]) * 1024 * 1024 : null;

      // 步驟 3: 提取下載連結
      const downloadLinkMatch = html.match(/data-download-url="(.+?)"/);
      if (!downloadLinkMatch) {
        throw new Error('無法找到下載連結');
      }
      const downloadUrl = downloadLinkMatch[1];

      // 步驟 4: 返回下載資訊
      context.log.info(`成功提取: ${filename}`);
      return {
        downloadUrl: downloadUrl,
        filename: filename,
        filesize: filesize,
        headers: {
          'User-Agent': 'Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36',
          'Referer': url
        }
      };

    } catch (error) {
      context.log.error(`提取失敗: ${error.message}`);
      throw error;
    }
  }
}

// 必須使用預設導出
export default ExampleShareModule;
```

### 程式碼詳解

#### 1. 型別提示（第 1-2 行）

```javascript
/// <reference types="../@types/zoo-downloader" />
```

這一行啟用 TypeScript 型別檢查，即使我們使用的是 JavaScript。VS Code 會顯示：
- 參數型別提示
- 自動完成建議
- 錯誤檢查

#### 2. canHandle 方法

```javascript
canHandle(url) {
  return url.includes('exampleshare.com/file/') ||
         url.includes('exampleshare.com/d/');
}
```

**作用**: 快速判斷是否處理此 URL
**何時呼叫**: 用戶添加下載時，系統會詢問每個模組
**返回值**: `true` = 我處理，`false` = 不處理

#### 3. extractDownloadInfo 方法

這是模組的核心。步驟：

1. **獲取頁面**: `context.http.get(url)`
2. **解析 HTML**: 使用正則表達式提取資訊
3. **返回物件**: 包含 `downloadUrl`, `filename` 等

**Context API 使用**:
- `context.log.info()` - 記錄資訊
- `context.log.debug()` - 除錯訊息
- `context.log.error()` - 錯誤訊息
- `context.http.get()` - 發送 HTTP GET 請求

---

## 第四部分：測試模組

### 步驟 5：使用測試工具

```bash
cd /nodejs/zoo-downloader/modules
node test-module.js example-share https://exampleshare.com/file/test123
```

**預期輸出**：

```
✓ Manifest 驗證通過
✓ canHandle() 返回 true
✓ extractDownloadInfo() 成功
{
  "downloadUrl": "https://cdn.exampleshare.com/files/...",
  "filename": "example-file.zip",
  "filesize": 10485760,
  "headers": {...}
}
```

### 步驟 6：除錯技巧

如果測試失敗，檢查：

1. **URL 模式不匹配**
   - 確認 `canHandle()` 邏輯正確
   - 檢查 `urlPatterns` in manifest.json

2. **權限錯誤**
   ```
   Error: Permission denied: domain not in whitelist
   ```
   → 將域名添加到 `permissions.network.allowDomains`

3. **解析失敗**
   - 使用 `context.log.debug(html)` 查看完整 HTML
   - 檢查正則表達式是否正確

---

## 第五部分：進階功能

### 處理需要認證的網站

如果網站需要登入：

#### 步驟 1: 在 manifest.json 中聲明

```json
{
  "permissions": {
    "network": {...},
    "credentials": ["username", "password"]
  }
}
```

#### 步驟 2: 使用認證

```javascript
async extractDownloadInfo(url, context) {
  // 檢查是否有認證資訊
  const hasAuth = await context.credentials.has('username');

  if (!hasAuth) {
    // 拋出特殊錯誤，觸發認證對話框
    throw new Error('AUTHENTICATION_REQUIRED: 需要登入');
  }

  // 獲取認證資訊（代理過的，安全）
  const username = await context.credentials.get('username');
  const password = await context.credentials.get('password');

  // 使用認證資訊登入
  const loginResponse = await context.http.post(
    'https://exampleshare.com/login',
    {
      body: JSON.stringify({ username, password }),
      headers: { 'Content-Type': 'application/json' }
    }
  );

  // 提取 session token
  const token = loginResponse.headers['set-cookie'];

  // 使用 token 訪問下載頁面
  const pageResponse = await context.http.get(url, {
    headers: { 'Cookie': token }
  });

  // ... 繼續處理
}
```

### 處理 CAPTCHA

如果網站有驗證碼：

```javascript
async extractDownloadInfo(url, context) {
  const pageResponse = await context.http.get(url);
  const html = pageResponse.body;

  // 檢查是否有 CAPTCHA
  const captchaMatch = html.match(/data-captcha-image="(.+?)"/);

  if (captchaMatch) {
    const captchaImageUrl = captchaMatch[1];

    // 請求用戶解決 CAPTCHA
    const solution = await context.captcha.solve({
      imageUrl: captchaImageUrl,
      type: 'text' // 或 'recaptcha', 'hcaptcha'
    });

    // 使用解答提交表單
    const downloadPageResponse = await context.http.post(
      'https://exampleshare.com/verify',
      {
        body: `captcha=${solution}`,
        headers: { 'Content-Type': 'application/x-www-form-urlencoded' }
      }
    );

    // ... 繼續處理
  }
}
```

---

## 第六部分：最佳實踐

### 1. 錯誤處理

```javascript
async extractDownloadInfo(url, context) {
  try {
    // 主要邏輯
  } catch (error) {
    // 記錄詳細錯誤
    context.log.error(`處理失敗: ${error.message}`);

    // 提供用戶友善的錯誤訊息
    if (error.message.includes('404')) {
      throw new Error('檔案不存在或已被刪除');
    } else if (error.message.includes('403')) {
      throw new Error('AUTHENTICATION_REQUIRED: 需要登入或權限不足');
    } else {
      throw new Error(`下載失敗: ${error.message}`);
    }
  }
}
```

### 2. 日誌記錄層級

```javascript
context.log.debug('僅在開發時顯示的詳細資訊');
context.log.info('重要的流程資訊');
context.log.warn('可能的問題，但不阻礙執行');
context.log.error('錯誤，導致失敗');
```

### 3. HTTP 請求最佳化

```javascript
// ✅ 好的做法：設置合理的 User-Agent
const response = await context.http.get(url, {
  headers: {
    'User-Agent': 'Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36'
  }
});

// ✅ 好的做法：設置 Referer（某些網站需要）
const downloadResponse = await context.http.get(downloadUrl, {
  headers: { 'Referer': pageUrl }
});

// ❌ 壞的做法：不處理錯誤
const response = await context.http.get(url); // 可能拋出異常
```

### 4. 資源限制意識

```javascript
// ✅ 好的做法：避免下載整個頁面（如果只需要部分資訊）
const response = await context.http.head(url); // 只獲取 headers

// ❌ 壞的做法：無限重試
let retries = 0;
while (retries < 100) { // 太多次！
  // ...
}

// ✅ 好的做法：合理的重試次數
const MAX_RETRIES = 3;
for (let i = 0; i < MAX_RETRIES; i++) {
  // ...
}
```

---

## 第七部分：發布模組

### 步驟 1：最終檢查

```bash
# 驗證 manifest
node test-module.js example-share validate

# 測試所有支援的 URL 格式
node test-module.js example-share https://exampleshare.com/file/test1
node test-module.js example-share https://exampleshare.com/d/test2
```

### 步驟 2：文件化

建立 `README.md`：

```markdown
# ExampleShare 模組

支援從 ExampleShare.com 下載檔案。

## 支援的 URL 格式

- `https://exampleshare.com/file/*`
- `https://exampleshare.com/d/*`

## 功能

- ✅ 自動提取檔案名稱和大小
- ✅ 支援免費和高級帳戶
- ❌ 不支援資料夾下載

## 使用方法

直接複製檔案連結到 Zoo Downloader 即可。

## 已知限制

- 免費帳戶有每日下載限制
- 大文件需要高級帳戶
```

### 步驟 3：部署

將整個 `example-share/` 資料夾複製到 Zoo Downloader 的 `modules/` 目錄：

```bash
cp -r example-share /path/to/zoo-downloader/modules/
```

重啟 Zoo Downloader，模組會自動載入。

---

## 故障排除

### 問題 1: 模組沒有被載入

**檢查**:
- [ ] `manifest.json` 是否有語法錯誤？（使用 JSON 驗證器）
- [ ] `id` 是否唯一？
- [ ] `index.js` 是否有語法錯誤？（執行 `node -c index.js`）

### 問題 2: canHandle 返回 true，但 extractDownloadInfo 失敗

**檢查**:
- [ ] 網站結構是否改變？（使用瀏覽器開發者工具檢查）
- [ ] 是否需要認證？（檢查是否收到 401/403 錯誤）
- [ ] 正則表達式是否正確？（使用 regex101.com 測試）

### 問題 3: Permission denied 錯誤

**解決方法**:
```json
// 在 manifest.json 中添加域名
"permissions": {
  "network": {
    "allowDomains": [
      "exampleshare.com",
      "cdn.exampleshare.com",  // ← 添加這個
      "api.exampleshare.com"   // ← 和這個
    ]
  }
}
```

---

## 進一步學習

### 推薦閱讀

1. **完整 API 參考**: `specs/004-site-module-integration/quickstart.md`
2. **進階指南**: `specs/004-site-module-integration/MODULE_DEVELOPMENT_GUIDE.md`
3. **K2S 範例**: `modules/k2s/index.js` (617 行的完整實作)

### 實用資源

- **TypeScript 型別定義**: `modules/@types/zoo-downloader/index.d.ts`
- **VS Code 程式碼片段**: `modules/.vscode/zoo-downloader.code-snippets`
- **模組模板**: `modules/_template/`

### 社群

- **問題回報**: GitHub Issues
- **功能建議**: GitHub Discussions
- **範例分享**: 歡迎貢獻您的模組到社群！

---

## 總結

恭喜！您已經學會：

✅ 建立模組結構（manifest + index.js）
✅ 使用 Context API（http, log, credentials, captcha）
✅ 解析網頁並提取下載資訊
✅ 處理錯誤和認證
✅ 測試和除錯模組
✅ 使用 TypeScript 型別提示

### 下一步

- 嘗試為您常用的檔案分享網站建立模組
- 研究 `modules/k2s/` 的進階技巧
- 加入錯誤重試機制
- 實作更複雜的認證流程

**Happy Coding!** 🚀
