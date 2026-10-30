# Simple Example Module

這是一個最小化的教學範例模組，展示了 Zoo Downloader 模組的基本結構。

## 目的

- ✅ 學習模組的基本結構
- ✅ 理解 `canHandle()` 和 `extractDownloadInfo()` 方法
- ✅ 熟悉 Context API 的使用

## 檔案結構

```
example-simple/
├── manifest.json    # 模組配置（id, 權限, URL 模式）
├── index.js         # 模組實作（canHandle, extractDownloadInfo）
└── README.md        # 本文件
```

## 使用方法

這不是一個實際可用的模組（example.com 是示範域名），僅供學習參考。

### 查看程式碼

1. 開啟 `manifest.json` 查看模組配置
2. 開啟 `index.js` 查看完整實作
3. 注意程式碼中的註解，解釋了每個步驟

### 測試語法

```bash
cd /nodejs/zoo-downloader/modules
node -c example-simple/index.js  # 檢查語法
```

## 學習重點

### manifest.json

- **id**: 模組唯一識別符（小寫 + 連字號）
- **urlPatterns**: 定義支援的 URL 格式（使用 `*` 萬用字元）
- **permissions.network.allowDomains**: 白名單域名列表

### index.js

#### canHandle(url)

```javascript
canHandle(url) {
  return url.includes('example.com/file/');
}
```

- 返回 `true` 表示此模組處理此 URL
- 返回 `false` 表示此模組不處理此 URL

#### extractDownloadInfo(url, context)

```javascript
async extractDownloadInfo(url, context) {
  // 1. 獲取頁面
  const response = await context.http.get(url);

  // 2. 解析 HTML
  const filename = html.match(/<h1>(.+?)<\/h1>/)[1];

  // 3. 返回下載資訊
  return {
    downloadUrl: '...',
    filename: filename,
    filesize: null,
    headers: {}
  };
}
```

## Context API 範例

### HTTP 請求

```javascript
// GET 請求
const response = await context.http.get(url);

// POST 請求
const response = await context.http.post(url, {
  body: 'key=value',
  headers: { 'Content-Type': 'application/x-www-form-urlencoded' }
});
```

### 日誌記錄

```javascript
context.log.debug('詳細的除錯資訊');
context.log.info('一般資訊');
context.log.warn('警告訊息');
context.log.error('錯誤訊息');
```

## 下一步

1. **閱讀完整教學**: `specs/004-site-module-integration/TUTORIAL.md`
2. **查看複雜範例**: `modules/k2s/index.js` (617 行的完整實作)
3. **使用模組模板**: `modules/_template/` (包含更多功能示範)
4. **查看 API 文件**: `specs/004-site-module-integration/quickstart.md`

## 參考資源

- **TypeScript 型別**: `modules/@types/zoo-downloader/index.d.ts`
- **VS Code 程式碼片段**: `modules/.vscode/zoo-downloader.code-snippets`
- **測試工具**: `src-tauri/tests/fixtures/site-modules/test-module.js`

---

**提示**: 這個模組故意保持簡單，實際模組需要更完善的錯誤處理、認證支援等功能。
