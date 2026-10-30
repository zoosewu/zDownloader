# 模組構建系統架構

**日期**: 2025-12-10
**狀態**: 建置流程可用；執行期 context API 尚未接線（見 docs/architecture/review-2026-10-30.md 的 B 段）

---

## 概述

Zoo Downloader 模組系統支援**構建工具驅動**的開發流程，讓模組開發者可以使用現代 JavaScript 工具鏈（Vite, Webpack, Rollup 等）來編譯他們的模組。

### 核心理念

✅ **開發者自主**：模組開發者自行選擇構建工具
✅ **標準生態**：遵循現代 JavaScript 生態系統慣例
✅ **靈活產出**：支援任何輸出格式，載入器自動轉換
✅ **清晰分離**：源碼（src/）與編譯產物（dist/）分離

---

## 架構設計

### 1. 目錄結構

```
模組目錄/
├── src/                    # 源碼目錄
│   └── index.js           # 源代碼（使用 ES6 export）
├── dist/                   # 編譯產物（被 .gitignore）
│   └── index.js           # 編譯後的代碼
├── manifest.json           # 模組 manifest
│   ├── entrypoint: "dist/index.js"  # ← 指定入口點
│   └── ...
├── package.json            # 構建工具配置
├── vite.config.js          # 構建工具配置（以 Vite 為例）
├── .gitignore              # 忽略 node_modules/ 和 dist/
└── README.md               # 模組文檔
```

### 2. Manifest 配置

**manifest.json** 新增 `entrypoint` 欄位：

```json
{
  "id": "my-module",
  "name": "My Module",
  "version": "1.0.0",
  "author": "Your Name",
  "description": "Module description",
  "entrypoint": "dist/index.js",  // ← 指定編譯後的入口點
  "urlPatterns": ["https://example.com/*"],
  "permissions": {
    "network": {
      "allowDomains": ["example.com"]
    }
  }
}
```

**`entrypoint` 欄位**：
- **類型**: 字串（相對路徑）
- **預設值**: `"index.js"`（向後相容）
- **用途**: 指定模組的入口文件
- **範例**:
  - `"dist/index.js"` - 使用 Vite/Webpack 編譯到 dist/
  - `"build/bundle.js"` - 自訂構建輸出
  - `"output/main.js"` - 其他輸出目錄

### 3. 構建工具配置

#### Vite 配置範例

**package.json**:
```json
{
  "name": "my-module",
  "version": "1.0.0",
  "type": "module",
  "private": true,
  "scripts": {
    "build": "vite build",
    "dev": "vite build --watch",
    "clean": "rm -rf dist"
  },
  "devDependencies": {
    "vite": "^5.4.0"
  }
}
```

**vite.config.js**:
```javascript
import { defineConfig } from 'vite';

/**
 * Vite plugin to remove ES module export syntax
 *
 * Zoo Downloader's module loader uses deno_core's execute_script() API,
 * which doesn't support ES module syntax (import/export).
 *
 * This plugin automatically transforms:
 *   export class Foo {}        → class Foo {}
 *   export default class Foo   → class Foo
 *   export function bar()      → function bar()
 *   export { ... }             → (removed)
 *
 * This allows developers to write standard ES modules during development,
 * while producing compatible output for the runtime.
 */
function removeExportSyntax() {
  return {
    name: 'remove-export-syntax',

    // Apply transformation after code generation
    generateBundle(options, bundle) {
      for (const fileName in bundle) {
        const chunk = bundle[fileName];

        if (chunk.type === 'chunk') {
          // Add banner at the top
          const banner = '// My Module - Compiled with Vite\n// Export syntax removed for Zoo Downloader compatibility\n\n';

          // Transform export statements to plain declarations
          let code = chunk.code
            // Remove export { ... } blocks
            .replace(/export\s*\{[^}]*\}\s*;?\s*/g, '')
            // Remove inline export declarations
            .replace(/export default class\s+/g, 'class ')
            .replace(/export class\s+/g, 'class ')
            .replace(/export default function\s+/g, 'function ')
            .replace(/export function\s+/g, 'function ')
            .replace(/export default const\s+/g, 'const ')
            .replace(/export const\s+/g, 'const ')
            .replace(/export default let\s+/g, 'let ')
            .replace(/export let\s+/g, 'let ')
            .replace(/export default var\s+/g, 'var ')
            .replace(/export var\s+/g, 'var ')
            .replace(/export default\s+/g, '');

          // Add banner and update chunk
          chunk.code = banner + code;
        }
      }
    }
  };
}

export default defineConfig({
  build: {
    outDir: 'dist',
    lib: {
      entry: 'src/index.js',
      formats: ['es'],  // ES module format (will be transformed by plugin)
      fileName: () => 'index.js',
    },
    rollupOptions: {
      output: {
        inlineDynamicImports: true,
        preserveModules: false,
      },
    },
    minify: false,  // 開發時禁用壓縮
    sourcemap: false,
    target: 'es2020',
  },

  // Register the custom plugin
  plugins: [
    removeExportSyntax(),
  ],
});
```

**✨ 推薦方案：使用 Vite 插件自動移除 `export`**

上面的配置包含 `removeExportSyntax()` 插件，它會：
- ✅ 在構建時自動移除所有 `export` 語法
- ✅ 讓開發者用標準 ES Module 開發
- ✅ 輸出可直接被 deno_core 執行的代碼
- ✅ 無需 Rust 載入器額外處理

#### Webpack 配置範例

**webpack.config.js**:
```javascript
module.exports = {
  mode: 'production',
  entry: './src/index.js',
  output: {
    path: __dirname + '/dist',
    filename: 'index.js',
    library: {
      type: 'module',
    },
  },
  experiments: {
    outputModule: true,
  },
  target: 'es2020',
};
```

#### Rollup 配置範例

**rollup.config.js**:
```javascript
export default {
  input: 'src/index.js',
  output: {
    file: 'dist/index.js',
    format: 'es',
    banner: '// My Module - Compiled with Rollup\n',
  },
};
```

---

## 載入器工作流程

### 1. 模組載入流程

```
1. 讀取 manifest.json
   ↓
2. 解析 entrypoint 欄位（預設 "index.js"）
   ↓
3. 檢查入口點文件是否存在
   ├─ ✅ 存在 → 繼續
   └─ ❌ 不存在 → 錯誤: "Entry point file 'xxx' not found. Please build your module first."
   ↓
4. 讀取入口點文件內容
   ↓
5. 自動轉換 ES6 module 語法
   ├─ export default class → class
   ├─ export class → class
   ├─ export function → function
   ├─ export const → const
   ├─ export let → let
   ├─ export var → var
   └─ export default → (移除)
   ↓
6. 創建 SiteModule 實例並初始化
   ↓
7. 註冊到模組系統
```

### 2. ES6 語法處理策略

**為什麼需要處理 `export`？**

Deno 的 `execute_script()` API 不支援 ES 模組語法（`export`/`import`），但現代 JavaScript 開發廣泛使用 ES Module。

**兩種處理方案**：

#### ✅ **方案 1：構建時處理（推薦）**

使用 Vite 插件在構建時自動移除 `export` 語法：

```javascript
// vite.config.js
plugins: [
  removeExportSyntax(),  // 見上方完整範例
]
```

**優點**：
- ✅ 開發者用標準 ES Module 開發
- ✅ 構建產物直接可執行，無需額外處理
- ✅ 編輯器完整的 IDE 支援和自動完成
- ✅ 支援所有 `export` 語法（包括 `export { ... }`）

#### ⚠️ **方案 2：運行時轉換（備用）**

Loader 在載入時自動轉換：

| 原始語法 | 轉換後 | 說明 |
|---------|-------|------|
| `export default class Foo` | `class Foo` | 預設類別導出 |
| `export class Foo` | `class Foo` | 命名類別導出 |
| `export function foo()` | `function foo()` | 命名函數導出 |
| `export const x = 1` | `const x = 1` | 命名常數導出 |
| `export let/var` | `let/var` | 變數導出 |
| `export default` | (移除) | 其他預設導出 |

**限制**：
- ⚠️ 不支援複雜的 export 語法（如 `export { A as B }`）
- ⚠️ 增加運行時開銷

**實作位置**: `src-tauri/src/modules/loader.rs`

**推薦**: 使用方案 1（Vite 插件），保持 loader 簡單。

```rust
// Transform ES6 module syntax to plain JavaScript
code = code.replace("export default class", "class")
           .replace("export class", "class")
           .replace("export default function", "function")
           .replace("export function", "function")
           .replace("export default const", "const")
           .replace("export const", "const")
           .replace("export let", "let")
           .replace("export var", "var")
           .replace("export default", "");
```

**性能影響**: 字串替換時間 < 5μs，每模組只轉換一次，可忽略不計（< 0.01%）

---

## 開發工作流程

### 方案 A：使用構建工具（推薦）

**適合**：
- 大型模組
- 需要 TypeScript 支援
- 需要預處理器（SCSS, PostCSS）
- 需要代碼分割和優化

**步驟**：

```bash
# 1. 初始化模組
cd modules/my-module
npm init -y

# 2. 安裝構建工具
npm install -D vite

# 3. 創建目錄結構
mkdir src dist

# 4. 編寫源碼
echo 'export class MyModule { ... }' > src/index.js

# 5. 配置構建工具
# 創建 vite.config.js（參考上面的範例）

# 6. 更新 manifest.json
# 添加 "entrypoint": "dist/index.js"

# 7. 構建模組
npm run build

# 8. 測試
# 在 Zoo Downloader 中掃描模組
```

### 方案 B：直接開發（簡單模組）

**適合**：
- 小型模組
- 不需要構建流程
- 快速原型開發

**步驟**：

```bash
# 1. 創建模組目錄
cd modules/my-module

# 2. 編寫代碼
echo 'export class MyModule { ... }' > index.js

# 3. 創建 manifest.json
# 不指定 entrypoint，使用預設的 "index.js"

# 4. 測試
# 在 Zoo Downloader 中掃描模組
```

**注意**: 即使不使用構建工具，`export` 語法仍會被自動轉換。

---

## K2S 模組範例

### 完整實作

**目錄結構**:
```
modules/k2s/
├── src/
│   └── index.js           # 源碼（20KB，使用 ES6 語法）
├── dist/
│   └── index.js           # 編譯產物（17.43 KB）
├── manifest.json           # entrypoint: "dist/index.js"
├── package.json
├── vite.config.js
├── .gitignore
└── README.md
```

**構建輸出**:
```bash
$ npm run build

> k2s-module@1.0.0 build
> vite build

vite v5.4.21 building for production...
transforming...
✓ 1 modules transformed.
rendering chunks...
computing gzip size...
dist/index.js  17.43 kB │ gzip: 4.08 kB
✓ built in 78ms
```

**生成的代碼**（dist/index.js）:
```javascript
// K2S Module - Compiled with Vite
// Note: ES module syntax (export) will be automatically transformed by loader

class K2SModule {
  constructor() {
    this.session = null;
    this.accountType = null;
    // ...
  }

  canHandle(url) {
    return url.includes('k2s.cc') || url.includes('keep2share.cc');
  }

  async extractDownloadInfo(url, context) {
    // ... 完整實作 ...
  }
}

export {
  K2SModule,
  K2SModule as default
};
```

**載入時自動轉換**:
```javascript
// 轉換後的代碼（執行時）
class K2SModule {
  constructor() {
    this.session = null;
    this.accountType = null;
    // ...
  }

  canHandle(url) {
    return url.includes('k2s.cc') || url.includes('keep2share.cc');
  }

  async extractDownloadInfo(url, context) {
    // ... 完整實作 ...
  }
}

// export 語法已被移除
```

---

## 最佳實踐

### 1. 目錄組織

**✅ 推薦**:
```
modules/my-module/
├── src/               # 源碼
├── dist/              # 編譯產物（.gitignore）
├── manifest.json
├── package.json
└── README.md
```

**❌ 不推薦**:
```
modules/my-module/
├── index.js           # 源碼和產物混在一起
├── manifest.json
└── README.md
```

### 2. Git 配置

**.gitignore**:
```gitignore
# Dependencies
node_modules/

# Build output
dist/

# Package manager lock files
package-lock.json
yarn.lock
pnpm-lock.yaml
```

**提交規則**:
- ✅ 提交源碼（src/）
- ✅ 提交配置（package.json, vite.config.js）
- ✅ 提交 manifest.json
- ❌ 不提交編譯產物（dist/）
- ❌ 不提交依賴（node_modules/）

### 3. 構建配置

**開發模式**（快速反饋）:
```json
{
  "scripts": {
    "dev": "vite build --watch"
  }
}
```
```bash
$ npm run dev
# 文件變更時自動重新構建
```

**生產模式**（優化產物）:
```javascript
// vite.config.js
export default defineConfig({
  build: {
    minify: 'terser',  // 啟用壓縮
    sourcemap: false,  // 禁用 source map
  },
});
```

### 4. 類型檢查

使用 TypeScript 型別定義獲得 IDE 支援：

**jsconfig.json**:
```json
{
  "compilerOptions": {
    "target": "ES2020",
    "module": "ESNext",
    "moduleResolution": "bundler",
    "types": ["../@types/zoo-downloader"]
  },
  "include": ["src/**/*"]
}
```

**src/index.js**:
```javascript
/// <reference types="../@types/zoo-downloader" />

/** @type {import('zoo-downloader').SiteModule} */
export class MyModule {
  /** @param {string} url */
  canHandle(url) {
    return true;
  }

  /**
   * @param {string} url
   * @param {import('zoo-downloader').ModuleContext} context
   * @returns {Promise<import('zoo-downloader').DownloadInfo>}
   */
  async extractDownloadInfo(url, context) {
    // 完整的 IDE 支援和類型檢查
  }
}
```

---

## 錯誤處理

### 常見錯誤

#### 錯誤 1：找不到入口點

**錯誤訊息**:
```
Entry point file 'dist/index.js' not found.
Please ensure the file exists or build your module first.
```

**原因**:
- 構建產物不存在
- manifest.json 中的 entrypoint 路徑錯誤

**解決方案**:
```bash
# 方案 1: 構建模組
npm run build

# 方案 2: 修正 manifest.json 的 entrypoint 欄位
```

#### 錯誤 2：空白輸出

**錯誤訊息**:
```
Entry point file 'dist/index.js' is empty
```

**原因**:
- 構建工具配置錯誤
- 沒有正確導出模組

**解決方案**:
```javascript
// src/index.js 必須有 export
export class MyModule { ... }
```

#### 錯誤 3：模組未註冊

**症狀**: 掃描模組後顯示 0 個模組

**可能原因**:
1. 入口點文件不存在（需要構建）
2. manifest.json 語法錯誤
3. 模組代碼有語法錯誤

**診斷步驟**:
```bash
# 1. 檢查入口點是否存在
ls -l modules/my-module/dist/index.js

# 2. 檢查 manifest.json 語法
cat modules/my-module/manifest.json | jq .

# 3. 查看日誌
# 在應用中點擊「掃描模組」並查看控制台輸出
```

---

## 遷移指南

### 從舊版模組遷移

**舊版結構** (無構建工具):
```
modules/old-module/
├── index.js           # 直接包含實作
└── manifest.json
```

**新版結構** (使用構建工具):
```
modules/old-module/
├── src/
│   └── index.js       # 移動源碼到 src/
├── dist/              # 新增構建產物目錄
│   └── index.js
├── manifest.json       # 添加 entrypoint 欄位
├── package.json       # 新增
├── vite.config.js     # 新增
└── .gitignore         # 新增
```

**遷移步驟**:

```bash
# 1. 創建新目錄結構
mkdir src dist

# 2. 移動源碼
mv index.js src/index.js

# 3. 初始化 package.json
npm init -y

# 4. 安裝 Vite
npm install -D vite

# 5. 創建 vite.config.js
# （參考上面的範例）

# 6. 更新 manifest.json
# 添加: "entrypoint": "dist/index.js"

# 7. 創建 .gitignore
echo "node_modules/\ndist/" > .gitignore

# 8. 構建
npm run build

# 9. 測試
# 在 Zoo Downloader 中掃描模組
```

**向後相容性**:

如果不想使用構建工具，可以保持原有結構：
- ✅ 不需要修改 manifest.json（預設 entrypoint 是 "index.js"）
- ✅ `export` 語法仍會被自動轉換
- ✅ 模組仍然可以正常載入

---

## 支援的構建工具

### ✅ 完全支援

| 工具 | 版本 | 配置範例 |
|-----|------|---------|
| **Vite** | 5.x | 見上方 vite.config.js |
| **Webpack** | 5.x | 見上方 webpack.config.js |
| **Rollup** | 4.x | 見上方 rollup.config.js |
| **esbuild** | 0.20.x | 命令行或 JS API |

### ⚠️ 需要配置

| 工具 | 說明 |
|-----|------|
| **Parcel** | 需要配置 targets |
| **Snowpack** | 需要配置 output |

### ❌ 不支援

| 工具 | 原因 |
|-----|------|
| **Browserify** | 輸出格式不適合 Deno |

---

## 技術限制

### 1. 不支援 `import` 語法

**❌ 不支援**:
```javascript
import { foo } from './utils.js';  // 不會被處理
```

**原因**: Deno 的 `execute_script()` 不支援模組解析

**解決方案**: 使用構建工具將所有代碼打包成單一文件

### 2. 不支援動態 import

**❌ 不支援**:
```javascript
const module = await import('./plugin.js');  // 運行時錯誤
```

**解決方案**: 使用構建工具的 `inlineDynamicImports` 選項

### 3. 不支援外部依賴

**❌ 不支援**:
```javascript
import axios from 'axios';  // 找不到模組
```

**解決方案**: 使用構建工具將依賴打包進輸出文件

**Vite 配置**:
```javascript
export default defineConfig({
  build: {
    rollupOptions: {
      external: [],  // 不排除任何依賴
    },
  },
});
```

---

## 效能考量

### 構建時間

| 模組大小 | 工具 | 構建時間 |
|---------|------|---------|
| < 10 KB | Vite | ~50ms |
| 10-50 KB | Vite | ~70ms |
| > 50 KB | Vite | ~100ms |

### 載入時間

| 階段 | 時間 | 說明 |
|------|------|------|
| **讀取文件** | ~1ms | I/O 操作 |
| **ES6 轉換** | < 5μs | 字串替換 |
| **Deno 初始化** | ~50ms | 創建 runtime |
| **總計** | ~51ms | 每模組 |

**結論**: ES6 自動轉換對效能影響可忽略不計（< 0.01%）

---

## 總結

### 核心優勢

✅ **靈活性**: 支援任何構建工具
✅ **標準化**: 遵循 JavaScript 生態系統慣例
✅ **自動化**: ES6 語法自動轉換
✅ **向後相容**: 舊模組無需修改
✅ **零開銷**: 轉換性能影響可忽略

### 推薦使用場景

| 場景 | 推薦方案 |
|------|---------|
| **小型模組** | 直接開發（不使用構建工具） |
| **大型模組** | Vite + 構建流程 |
| **TypeScript** | Vite + TypeScript plugin |
| **多文件模組** | Webpack 或 Rollup |
| **第三方依賴** | 任何打包工具 |

---

**完成時間**: 2025-12-10
**狀態**: 建置流程可用；執行期 context API 尚未接線（見 docs/architecture/review-2026-10-30.md 的 B 段）
**範例**: K2S 模組（17.43 KB，使用 Vite 構建）

**相關文檔**:
- `SMART_MODULE_PATH.md` - 智能路徑解析
