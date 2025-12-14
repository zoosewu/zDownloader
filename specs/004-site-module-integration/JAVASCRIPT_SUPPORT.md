# JavaScript 版本支援

**更新日期**: 2025-12-10
**Runtime**: deno_core 0.372
**JavaScript 引擎**: V8（最新穩定版）

---

## 支援的 JavaScript 版本

### ✅ 完整支援: ES2015 - ES2023+

Zoo Downloader 的模組沙箱使用 **deno_core 0.372**，基於 Google V8 JavaScript 引擎，支援最新的 ECMAScript 規範。

---

## ECMAScript 特性支援表

### ES2015 (ES6) ✅

| 特性 | 支援 | 範例 |
|-----|------|------|
| let/const | ✅ | `const x = 1; let y = 2;` |
| Arrow functions | ✅ | `const fn = () => 42;` |
| Template literals | ✅ | `const s = \`hello \${name}\`;` |
| Destructuring | ✅ | `const {a, b} = obj;` |
| Default parameters | ✅ | `function fn(x = 5) { }` |
| Rest parameters | ✅ | `function fn(...args) { }` |
| Spread operator | ✅ | `[1, ...[2, 3]]` |
| Classes | ✅ | `class Foo { }` |
| for...of | ✅ | `for (const x of arr) { }` |
| Map/Set | ✅ | `new Map(), new Set()` |
| Promise | ✅ | `new Promise(resolve => { })` |
| Modules (import/export) | ⚠️ | 自動轉換（見下方說明） |

### ES2016 ✅

| 特性 | 支援 | 範例 |
|-----|------|------|
| Exponentiation | ✅ | `2 ** 3` |
| Array.includes | ✅ | `[1,2,3].includes(2)` |

### ES2017 ✅

| 特性 | 支援 | 範例 |
|-----|------|------|
| async/await | ✅ | `async function fn() { await ... }` |
| Object.values | ✅ | `Object.values(obj)` |
| Object.entries | ✅ | `Object.entries(obj)` |
| String.padStart/padEnd | ✅ | `'5'.padStart(3, '0')` |

### ES2018 ✅

| 特性 | 支援 | 範例 |
|-----|------|------|
| Rest/Spread properties | ✅ | `const {x, ...rest} = obj;` |
| Promise.finally | ✅ | `promise.finally(() => { })` |
| Async iteration | ✅ | `async function* gen() { }` |
| RegExp named groups | ✅ | `/(?<year>\\d{4})/` |

### ES2019 ✅

| 特性 | 支援 | 範例 |
|-----|------|------|
| Array.flat | ✅ | `[1, [2, 3]].flat()` |
| Array.flatMap | ✅ | `arr.flatMap(x => [x, x*2])` |
| Object.fromEntries | ✅ | `Object.fromEntries(entries)` |
| String.trimStart/trimEnd | ✅ | `' hello '.trimStart()` |
| Optional catch | ✅ | `try { } catch { }` |

### ES2020 ✅

| 特性 | 支援 | 範例 |
|-----|------|------|
| Optional chaining | ✅ | `obj?.a?.b` |
| Nullish coalescing | ✅ | `value ?? defaultValue` |
| BigInt | ✅ | `123n` |
| Promise.allSettled | ✅ | `Promise.allSettled([...])` |
| globalThis | ✅ | `globalThis.x = 1` |
| String.matchAll | ✅ | `'abc'.matchAll(/./g)` |

### ES2021 ✅

| 特性 | 支援 | 範例 |
|-----|------|------|
| String.replaceAll | ✅ | `'aaa'.replaceAll('a', 'b')` |
| Promise.any | ✅ | `Promise.any([...])` |
| Logical assignment | ✅ | `x ||= 2; x &&= 3; x ??= 4;` |
| Numeric separators | ✅ | `1_000_000` |
| WeakRef | ✅ | `new WeakRef(obj)` |

### ES2022 ✅

| 特性 | 支援 | 範例 |
|-----|------|------|
| Class fields | ✅ | `class C { x = 1; }` |
| Private fields | ✅ | `class C { #x = 1; }` |
| Static fields | ✅ | `class C { static x = 1; }` |
| Top-level await | ❌ | 不支援（模組系統限制） |
| Array.at | ✅ | `[1,2,3].at(-1)` |
| Object.hasOwn | ✅ | `Object.hasOwn(obj, 'key')` |

### ES2023 ✅

| 特性 | 支援 | 範例 |
|-----|------|------|
| Array.findLast | ✅ | `arr.findLast(x => x > 1)` |
| Array.findLastIndex | ✅ | `arr.findLastIndex(x => x > 1)` |
| Array.toSorted | ✅ | `[3,1,2].toSorted()` |
| Array.toReversed | ✅ | `[1,2,3].toReversed()` |
| Array.toSpliced | ✅ | `arr.toSpliced(1, 1, 'x')` |
| Array.with | ✅ | `arr.with(1, 'x')` |

---

## 特殊支援與限制

### ⚠️ ES 模組語法（import/export）

**狀態**: 自動轉換支援

模組系統使用 `execute_script()` 而非完整的 ES 模組載入器，因此：

**支援的語法**（會自動轉換）:
```javascript
// ✅ 這些會自動轉換為普通 JavaScript
export class MyModule { }
export function foo() { }
export const x = 1;
export default MyModule;
```

**不支援的語法**:
```javascript
// ❌ 不支援 import 語句
import { foo } from './utils.js';

// ❌ 不支援動態 import
const module = await import('./plugin.js');
```

**解決方案**: 使用構建工具（Vite, Webpack）將所有代碼打包成單一文件。

### ❌ Top-level await

**狀態**: 不支援

```javascript
// ❌ 不支援
const data = await fetch('...');

// ✅ 使用 async 函數包裝
async function init() {
  const data = await fetch('...');
}
```

**原因**: `execute_script()` 不支援模組的 top-level await

---

## 推薦的構建目標

### Vite 配置

```javascript
// vite.config.js
export default defineConfig({
  build: {
    target: 'es2020',  // ✅ 推薦: 穩定且廣泛支援
    // target: 'es2021',  // ✅ 也可以，支援更多現代特性
    // target: 'es2022',  // ⚠️ 可以但注意 top-level await
  },
});
```

### TypeScript 配置

```json
{
  "compilerOptions": {
    "target": "ES2020",  // ✅ 推薦
    "module": "ESNext",
    "lib": ["ES2020", "DOM"]
  }
}
```

---

## 瀏覽器 API 支援

### ✅ 支援的全域 API

| API | 支援 | 說明 |
|-----|------|------|
| `console` | ✅ | console.log, error, warn, info |
| `JSON` | ✅ | JSON.parse, stringify |
| `Math` | ✅ | 所有 Math 方法 |
| `Date` | ✅ | 日期時間操作 |
| `setTimeout/setInterval` | ✅ | 定時器（沙箱限制） |
| `Promise` | ✅ | 非同步操作 |
| `Array/Object/String/Number` | ✅ | 所有內建方法 |
| `RegExp` | ✅ | 正則表達式 |
| `Error` | ✅ | 錯誤處理 |

### ❌ 不支援的瀏覽器 API

| API | 支援 | 替代方案 |
|-----|------|---------|
| `fetch` | ❌ | 使用 `context.http` |
| `XMLHttpRequest` | ❌ | 使用 `context.http` |
| `localStorage` | ❌ | N/A（安全考量） |
| `sessionStorage` | ❌ | N/A（安全考量） |
| `document` | ❌ | N/A（不是瀏覽器環境） |
| `window` | ❌ | 使用 `globalThis` |
| `location` | ❌ | N/A |
| `navigator` | ❌ | N/A |

---

## 效能考量

### V8 最佳化

deno_core 使用 Google V8 引擎，享有完整的 JIT 編譯和優化：

- ✅ **TurboFan JIT 編譯器**: 執行時優化
- ✅ **內聯快取**: 加速屬性存取
- ✅ **隱藏類**: 優化物件結構
- ✅ **垃圾回收**: 自動記憶體管理

### 效能建議

**✅ 推薦做法**:
```javascript
// 使用現代 JavaScript 特性
const result = array
  .filter(x => x > 0)
  .map(x => x * 2);

// 使用可選鏈
const value = obj?.nested?.property;

// 使用 async/await
async function process() {
  const data = await fetch();
  return transform(data);
}
```

**❌ 避免**:
```javascript
// 過度使用 eval
eval(dynamicCode);  // 無法優化

// 動態修改物件結構
obj.newProp = 1;    // 破壞隱藏類優化
delete obj.prop;    // 破壞隱藏類優化
```

---

## 測試與驗證

### 驗證 JavaScript 特性

建立測試程序確認特定特性是否支援：

```javascript
// test.js
class TestModule {
  canHandle(url) {
    // 測試 optional chaining
    const hasProtocol = url?.startsWith?.('https://');

    // 測試 nullish coalescing
    const port = url.port ?? 443;

    // 測試 array methods
    const params = url.split('&')
      .filter(x => x.includes('='))
      .map(x => x.split('='));

    return true;
  }

  async extractDownloadInfo(url, context) {
    // 測試 async/await
    const response = await context.http.get(url);

    // 測試解構
    const { body, status } = response;

    // 測試 template literals
    const message = `Status: ${status}`;

    return {
      downloadUrl: body.url,
      filename: body.name ?? 'unknown.bin',
      filesize: null,
      headers: {}
    };
  }
}
```

---

## 總結

### JavaScript 支援

| 規範 | 支援程度 |
|-----|---------|
| **ES2015-ES2021** | ✅ 100% 支援 |
| **ES2022** | ✅ 99% 支援（除 top-level await）|
| **ES2023** | ✅ 100% 支援 |
| **ES2024+** | ✅ 大部分支援（V8 持續更新）|

### 推薦配置

**Vite/Webpack 目標**:
```
target: 'es2020' 或 'es2021'
```

**TypeScript 目標**:
```json
{
  "target": "ES2020",
  "lib": ["ES2020"]
}
```

### 關鍵限制

1. ❌ 不支援 `import` 語句（使用構建工具打包）
2. ❌ 不支援 top-level `await`（使用 async 函數包裝）
3. ❌ 不支援瀏覽器 DOM API（使用 `context` API）
4. ✅ 支援幾乎所有現代 JavaScript 特性

---

**技術細節**:
- Runtime: deno_core 0.372
- Engine: V8 (最新穩定版)
- ECMAScript: ES2015 - ES2023+

**參考資源**:
- [deno_core 文檔](https://github.com/denoland/deno_core)
- [V8 JavaScript 引擎](https://v8.dev/)
- [ECMAScript 規範](https://tc39.es/ecma262/)
