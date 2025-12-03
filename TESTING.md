# Zoo Downloader - 測試指南

## 概述

本專案包含完整的前後端資料一致性測試，確保 API 契約在前後端之間保持同步。

**重要**：測試文件不會包含在生產構建中。
- `tsconfig.json` 的 `exclude` 配置確保構建時排除測試文件
- Vitest 使用自己的配置（`vitest.config.ts`）來運行測試
- 這樣測試文件可以用於 CI/CD，但不會被打包到生產環境

## 測試結構

```
zoo-downloader/
├── src-tauri/
│   └── src/
│       └── modules/
│           └── url_detector/
│               └── mod.rs          # 包含 #[cfg(test)] 測試
├── src/
│   └── __tests__/
│       └── api-serialization.test.ts  # 前端測試
└── test-fixtures/
    └── api-samples.json           # 共享測試資料
```

---

## 後端測試（Rust）

### 運行測試

```bash
cd src-tauri
cargo test
```

運行特定模組的測試：
```bash
cargo test url_detector::tests
```

### 測試覆蓋

#### ✅ URL Detection API (`UrlDetectionInfo`)

**位置**：`src-tauri/src/modules/url_detector/mod.rs`

**測試項目**：
1. ✓ `test_url_detection_info_serialization_uses_camel_case`
   - 確保序列化輸出使用 camelCase
   - 驗證沒有 snake_case 欄位洩漏

2. ✓ `test_url_detection_info_deserialization_from_camel_case`
   - 確保能正確解析前端的 camelCase JSON

3. ✓ `test_detection_mode_serialization_uses_camel_case`
   - 確保 enum 值使用 camelCase

4. ✓ `test_url_detection_info_roundtrip`
   - 完整的序列化/反序列化往返測試

**範例輸出**：
```json
{
  "url": "https://example.com/test-file.zip",
  "isValid": true,
  "detectorName": "BuiltinFileDetector",
  "detectionMode": "fileExtension",
  "suggestedFilename": "test-file.zip",
  "fileExtension": "zip"
}
```

### 測試結果

```
running 4 tests
test modules::url_detector::tests::test_detection_mode_serialization_uses_camel_case ... ok
test modules::url_detector::tests::test_url_detection_info_deserialization_from_camel_case ... ok
test modules::url_detector::tests::test_url_detection_info_serialization_uses_camel_case ... ok
test modules::url_detector::tests::test_url_detection_info_roundtrip ... ok

test result: ok. 4 passed; 0 failed; 0 ignored
```

---

## 前端測試（TypeScript + Vitest）

### 運行測試

```bash
# 運行所有測試
pnpm test

# 運行測試一次（CI 模式）
pnpm test:run

# 開啟測試 UI
pnpm test:ui
```

### 測試覆蓋

#### ✅ URL Detection API 解析測試

**位置**：`src/__tests__/api-serialization.test.ts`

**測試項目**：
1. `should correctly parse backend camelCase JSON`
   - 驗證能正確解析後端返回的 JSON

2. `should not use snake_case field names`
   - 展示 snake_case 會導致 undefined

3. `should maintain data integrity in roundtrip`
   - 完整的資料往返測試

4. `should handle optional fields correctly`
   - 可選欄位處理

5. `should correctly parse enum values in camelCase`
   - enum 值解析測試

6. `TypeScript should catch snake_case usage at compile time`
   - 型別安全測試

### 測試結果

```
✓ src/__tests__/api-serialization.test.ts (6 tests) 3ms

Test Files  1 passed (1)
     Tests  6 passed (6)
  Start at  10:06:55
  Duration  31.25s
```

### 驗證函數

提供了 `validateBackendResponse()` 輔助函數，用於執行時驗證：

```typescript
import { validateBackendResponse } from './src/__tests__/api-serialization.test';

const response = await invoke('detect_url', { url });
if (!validateBackendResponse(response)) {
  console.error('Backend response validation failed');
}
```

---

## 測試資料共享

### test-fixtures/api-samples.json

包含前後端共享的測試資料，確保雙方使用相同的測試案例：

```json
{
  "urlDetectionInfo": {
    "description": "URL detection result - 前後端共享的測試資料",
    "json_output": {
      "url": "https://example.com/test-file.zip",
      "isValid": true,
      "detectorName": "BuiltinFileDetector",
      "detectionMode": "fileExtension",
      "suggestedFilename": "test-file.zip",
      "fileExtension": "zip"
    }
  }
}
```

---

## 開發新 API 時的測試流程

### 1. 定義 Rust 資料結構

```rust
#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]  // ← 必須添加
pub struct MyApiResponse {
    pub user_name: String,      // → userName
    pub is_active: bool,         // → isActive
    pub created_at: DateTime,    // → createdAt
}
```

### 2. 添加後端序列化測試

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_my_api_response_serialization() {
        let response = MyApiResponse {
            user_name: "test".to_string(),
            is_active: true,
            created_at: Utc::now(),
        };

        let json = serde_json::to_string(&response).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();

        // 確保是 camelCase
        assert!(parsed.get("userName").is_some());
        assert!(parsed.get("isActive").is_some());

        // 確保沒有 snake_case
        assert!(parsed.get("user_name").is_none());
        assert!(parsed.get("is_active").is_none());
    }

    #[test]
    fn test_my_api_response_roundtrip() {
        let original = MyApiResponse { /* ... */ };
        let json = serde_json::to_string(&original).unwrap();
        let deserialized: MyApiResponse = serde_json::from_str(&json).unwrap();

        assert_eq!(original.user_name, deserialized.user_name);
    }
}
```

### 3. 定義前端 TypeScript 介面

```typescript
interface MyApiResponse {
  userName: string;    // ✅ 正確
  isActive: boolean;   // ✅ 正確
  createdAt: string;   // ✅ 正確
}
```

### 4. 添加前端測試

```typescript
// src/__tests__/my-api.test.ts
import { describe, test, expect } from 'vitest';

describe('MyApiResponse', () => {
  test('should parse backend camelCase JSON', () => {
    const json = '{"userName":"test","isActive":true}';
    const result: MyApiResponse = JSON.parse(json);

    expect(result.userName).toBe('test');
    expect(result.isActive).toBe(true);
  });
});
```

### 5. 運行測試

```bash
# 後端測試
cd src-tauri && cargo test

# 前端測試
pnpm test:run
```

### 6. 提交前檢查

```bash
# 確保所有測試通過
cd src-tauri && cargo test && cd .. && pnpm test:run

# 確保構建成功
pnpm build
```

---

## CI/CD 整合

### GitHub Actions 範例

```yaml
name: Tests

on: [push, pull_request]

jobs:
  backend-tests:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      - name: Setup Rust
        uses: actions-rs/toolchain@v1
        with:
          toolchain: stable
      - name: Run Rust tests
        run: |
          cd src-tauri
          cargo test --all-features

  frontend-tests:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      - name: Setup Node.js
        uses: actions/setup-node@v3
        with:
          node-version: '18'
      - name: Install pnpm
        uses: pnpm/action-setup@v2
        with:
          version: 8
      - name: Install dependencies
        run: pnpm install
      - name: Run frontend tests
        run: pnpm test:run
```

### Pre-commit Hook

創建 `.git/hooks/pre-commit`：

```bash
#!/bin/bash

echo "Running tests..."

# 後端測試
echo "→ Rust tests"
cd src-tauri
cargo test --quiet
if [ $? -ne 0 ]; then
    echo "❌ Rust tests failed"
    exit 1
fi
cd ..

# 前端測試
echo "→ Frontend tests"
pnpm test:run
if [ $? -ne 0 ]; then
    echo "❌ Frontend tests failed"
    exit 1
fi

echo "✅ All tests passed!"
```

使其可執行：
```bash
chmod +x .git/hooks/pre-commit
```

---

## 構建配置說明

### tsconfig.json

測試文件會被排除在 TypeScript 編譯之外：

```json
{
  "exclude": [
    "src/**/__tests__",
    "src/**/*.test.ts",
    "src/**/*.test.tsx",
    "src/**/*.spec.ts",
    "src/**/*.spec.tsx"
  ]
}
```

這確保：
- ✅ `pnpm build` 不會嘗試編譯測試文件
- ✅ 測試文件不會被打包到生產環境
- ✅ 開發和生產構建都不會出錯

### vitest.config.ts

Vitest 使用自己的配置來運行測試：

```typescript
export default defineConfig({
  test: {
    include: ['src/**/__tests__/**/*.test.{ts,tsx}'],
    exclude: ['node_modules', 'dist', 'src-tauri'],
  },
});
```

這確保：
- ✅ `pnpm test` 能找到並運行所有測試
- ✅ 測試環境獨立於構建環境
- ✅ 可以在 CI/CD 中運行測試

---

## 常見問題

### Q: 為什麼測試文件不會包含在構建中？

A: `tsconfig.json` 的 `exclude` 配置確保 TypeScript 編譯器忽略測試文件。Vitest 使用自己的配置（`vitest.config.ts`）來運行測試，不受 tsconfig 影響。

### Q: 如何確保前端收到正確的資料？

A:
1. 後端測試確保輸出是 camelCase JSON
2. 前端測試驗證能正確解析 camelCase JSON
3. 共享測試資料（test-fixtures）確保前後端使用相同格式
4. TypeScript interface 提供編譯時型別檢查

### Q: 測試失敗時如何除錯？

A:
1. 檢查後端測試：`cd src-tauri && cargo test -- --nocapture`
2. 檢查前端測試：`pnpm test:ui`（開啟視覺化介面）
3. 檢查 test-fixtures 中的測試資料是否一致
4. 確認結構有 `#[serde(rename_all = "camelCase")]`

### Q: 為什麼 `cargo test` 會運行其他測試？

A: `cargo test` 預設運行所有測試。使用 `cargo test url_detector::tests` 只運行序列化相關測試。

### Q: 如何新增其他 API 的測試？

A: 參考 `url_detector/mod.rs` 和 `src/__tests__/api-serialization.test.ts` 中的測試範例，為新的結構添加類似的測試。

---

## 測試檢查清單

開發新 API 時，請確保：

- [ ] Rust struct/enum 已添加 `#[serde(rename_all = "camelCase")]`
- [ ] 已添加後端序列化測試（檢查 camelCase 輸出）
- [ ] 已添加後端反序列化測試（檢查能解析 camelCase 輸入）
- [ ] 已添加後端往返測試（確保資料完整性）
- [ ] `cargo test` 全部通過
- [ ] 前端 TypeScript interface 使用 camelCase 欄位名
- [ ] 已添加前端解析測試
- [ ] `pnpm test:run` 全部通過
- [ ] `pnpm build` 構建成功（測試文件被排除）
- [ ] 在瀏覽器中手動測試功能

---

**最後更新**：2025-12-03
**維護者**：開發團隊
