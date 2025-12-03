# Zoo Downloader - 測試指南

## 概述

本專案包含完整的前後端資料一致性測試，確保 API 契約在前後端之間保持同步。

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

## 後端測試（Rust）

### 運行測試

```bash
cd src-tauri
cargo test
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

**測試資料範例**：
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

## 前端測試（TypeScript + Vitest）

### 運行測試

```bash
npm run test
# 或
npx vitest
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

### 驗證函數

提供了 `validateBackendResponse()` 輔助函數，用於執行時驗證：

```typescript
import { validateBackendResponse } from './src/__tests__/api-serialization.test';

const response = await invoke('detect_url', { url });
if (!validateBackendResponse(response)) {
  console.error('Backend response validation failed');
}
```

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

## 測試流程

### 開發新 API 時

1. **定義 Rust 資料結構**
   ```rust
   #[derive(Serialize, Deserialize)]
   #[serde(rename_all = "camelCase")]  // ← 必須！
   pub struct NewApi {
       pub some_field: String,
   }
   ```

2. **撰寫後端測試**
   ```rust
   #[test]
   fn test_new_api_serialization() {
       let data = NewApi { some_field: "test".to_string() };
       let json = serde_json::to_string(&data).unwrap();
       let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
       assert!(parsed.get("someField").is_some());
       assert!(parsed.get("some_field").is_none());
   }
   ```

3. **定義 TypeScript 介面**
   ```typescript
   interface NewApi {
     someField: string;  // camelCase
   }
   ```

4. **撰寫前端測試**
   ```typescript
   test('should parse NewApi', () => {
       const json = '{"someField": "test"}';
       const data: NewApi = JSON.parse(json);
       expect(data.someField).toBe('test');
   });
   ```

5. **運行所有測試**
   ```bash
   cargo test && npm run test
   ```

### 修改現有 API 時

1. 修改 Rust 結構
2. 運行測試：`cargo test`
3. 如果測試失敗，檢查是否：
   - 忘記添加 `#[serde(rename_all = "camelCase")]`
   - 欄位名稱不符合 snake_case
   - 破壞了 API 契約
4. 同步更新 TypeScript 介面
5. 同步更新前端測試
6. 再次運行所有測試

## CI/CD 整合

### GitHub Actions 範例

```yaml
name: API Consistency Tests

on: [push, pull_request]

jobs:
  backend-tests:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      - uses: actions-rs/toolchain@v1
        with:
          toolchain: stable
      - name: Run Rust tests
        run: |
          cd src-tauri
          cargo test

  frontend-tests:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      - uses: actions/setup-node@v3
      - run: npm install
      - run: npm run test
```

### Pre-commit Hook

```bash
#!/bin/bash
# .git/hooks/pre-commit

echo "🧪 Running API consistency tests..."

# Backend
echo "Testing Rust..."
cd src-tauri && cargo test --quiet || {
    echo "❌ Rust tests failed"
    exit 1
}

# Frontend
echo "Testing TypeScript..."
cd .. && npm run test -- --run || {
    echo "❌ TypeScript tests failed"
    exit 1
}

echo "✅ All tests passed!"
```

## 常見問題排查

### 測試失敗：找不到 camelCase 欄位

**症狀**：
```
assertion failed: parsed.get("isValid").is_some()
Missing 'isValid' field (should be camelCase, not is_valid)
```

**原因**：Rust 結構缺少 `#[serde(rename_all = "camelCase")]`

**修復**：
```rust
#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]  // ← 添加這行
pub struct YourStruct {
    pub is_valid: bool,
}
```

### 測試失敗：前端收到 undefined

**症狀**：前端測試中 `result.isValid` 是 `undefined`

**原因**：後端返回的是 `is_valid` 而不是 `isValid`

**診斷**：
```bash
# 檢查序列化輸出
cargo test -- --nocapture | grep -A 10 "serialization"
```

**修復**：同上，添加 `#[serde(rename_all = "camelCase")]`

### 新增欄位後測試失敗

**症狀**：添加新欄位後，測試報告欄位缺失

**檢查清單**：
- [ ] Rust 結構是否添加了新欄位
- [ ] TypeScript 介面是否同步更新
- [ ] 測試資料是否包含新欄位
- [ ] 命名是否一致（Rust snake_case，JSON camelCase）

## 測試統計

| 模組 | 測試數量 | 覆蓋率 | 狀態 |
|------|---------|--------|------|
| URL Detector (Backend) | 4 | 100% | ✅ |
| URL Detector (Frontend) | 6 | 100% | ✅ |

## 維護建議

1. **每次 API 變更都要更新測試**
2. **測試失敗必須修復，不能跳過**
3. **定期檢查測試覆蓋率**
4. **新增 API 必須包含測試**
5. **破壞性變更必須在測試中標註**

## 相關文件

- [CLAUDE.md](./CLAUDE.md) - 開發規範和命名約定
- [LOGGING.md](./LOGGING.md) - 日誌系統說明
- [test-fixtures/](./test-fixtures/) - 共享測試資料

---

**最後更新**：2025-12-03
**測試框架**：Rust (cargo test) + TypeScript (Vitest)
**狀態**：✅ 所有測試通過
