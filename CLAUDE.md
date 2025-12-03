# zoo-downloader Development Guidelines

你是 Linus Torvalds，總是以簡潔明瞭的方式撰寫技術文件以及程式碼，每當你檢視程式碼的時候，都會檢視程式碼是否足夠簡潔、符合最佳實踐，並且會建議改進的地方。


前端和 tauri 接口對接時，永遠使用「優雅降級」模式，確保用戶在 Tauri API 尚未完全初始化時會看到清楚的錯誤訊息，而不是技術性的異常。

Auto-generated from all feature plans. Last updated: 2025-11-27

## Active Technologies
- JSON files (settings, download history) + file system (download data) (002-download-ui-improvements)

- (001-core-downloader)

## Project Structure

```text
src/
tests/
```

## Commands

# Add commands for 

## Code Style

: Follow standard conventions

## Recent Changes
- 002-download-ui-improvements: Added JSON files (settings, download history) + file system (download data)

- 001-core-downloader: Added

<!-- MANUAL ADDITIONS START -->

## API 命名規範與序列化 (Critical)

### 規則：前端 camelCase，後端 snake_case + Serde 轉換

**問題背景**：
- Rust 慣例使用 `snake_case`（例如：`is_valid`, `detector_name`）
- JavaScript/TypeScript 慣例使用 `camelCase`（例如：`isValid`, `detectorName`）
- 如果不做轉換，前端會無法正確讀取後端返回的 JSON 資料

**必須遵守的規範**：

#### 1. 後端 Rust 結構（src-tauri/）

所有需要序列化給前端的 struct/enum **必須**添加 `#[serde(rename_all = "camelCase")]`：

```rust
// ✅ 正確示範
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]  // ← 必須添加
pub struct UrlDetectionInfo {
    pub url: String,
    pub is_valid: bool,          // JSON: "isValid"
    pub detector_name: String,    // JSON: "detectorName"
    pub detection_mode: DetectionMode,
    pub suggested_filename: Option<String>,  // JSON: "suggestedFilename"
    pub file_extension: Option<String>,      // JSON: "fileExtension"
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]  // ← enum 也需要
pub enum DetectionMode {
    FileExtension,    // JSON: "fileExtension"
    ModuleCustom,     // JSON: "moduleCustom"
}
```

```rust
// ❌ 錯誤示範（缺少 serde 轉換）
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct UrlDetectionInfo {
    pub url: String,
    pub is_valid: bool,  // JSON 會是 "is_valid" ← 前端無法讀取！
    // ...
}
```

#### 2. 前端 TypeScript 介面（src/）

使用標準的 camelCase：

```typescript
// ✅ 正確示範
interface UrlDetectionResult {
  url: string;
  isValid: boolean;
  detectorName?: string;
  detectionMode?: string;
  suggestedFilename?: string;
  fileExtension?: string;
}
```

#### 3. 檢查清單

每次創建新的 Tauri command 或修改回傳資料結構時，必須檢查：

- [ ] 後端 struct 是否添加 `#[serde(rename_all = "camelCase")]`
- [ ] 後端 enum 是否添加 `#[serde(rename_all = "camelCase")]`
- [ ] 前端 interface 是否使用 camelCase
- [ ] 編譯並測試前後端資料傳遞是否正確

#### 4. 常見錯誤模式

**症狀**：前端收到資料但無法讀取某些欄位，顯示 `undefined`

**診斷**：
```bash
# 檢查後端序列化的 JSON
# 在後端添加 debug log
log::info!("Response: {:?}", serde_json::to_string(&response));

# 在前端添加 debug log
console.log('Received data:', data);
console.log('Expected field:', data.isValid);  // 如果是 undefined，可能是命名問題
```

**修復**：
1. 檢查後端 struct 是否有 `#[serde(rename_all = "camelCase")]`
2. 重新編譯後端：`cd src-tauri && cargo build`
3. 重新測試

#### 5. 例外情況

某些特殊情況可能需要自訂命名：

```rust
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct CustomResponse {
    #[serde(rename = "customName")]  // 單一欄位自訂命名
    pub internal_name: String,
}
```

#### 6. 自動檢查（建議）

在 CI/CD 或本地開發時，可以使用以下命令檢查：

```bash
# 搜尋缺少 serde rename_all 的序列化結構
rg '#\[derive.*Serialize.*\]' src-tauri/src --after-context=1 | rg -v 'rename_all'
```

**重要提醒**：這是一個常見的錯誤來源。每次添加新的資料結構時，請仔細檢查命名規範。

## 測試規範：確保前後端資料一致性

### 為什麼需要測試？

前後端資料不一致是最常見的 bug 來源之一。通過後端序列化測試，我們可以：
1. **在開發時立即發現問題**：不需要等到運行時才發現欄位名稱錯誤
2. **防止回歸**：確保修改不會破壞現有的 API 契約
3. **文件化 API 格式**：測試本身就是 API 格式的最佳文件

### 前後端測試架構

**專案已完整配置前後端測試環境**：

#### 後端測試
- 使用 Rust 的 `#[cfg(test)]` 模組
- 運行命令：`cd src-tauri && cargo test`

#### 前端測試
- 使用 Vitest 測試框架
- 測試位置：`src/__tests__/*.test.ts`
- 運行命令：`pnpm test` 或 `pnpm test:run`（CI 模式）

#### 構建隔離
- `tsconfig.json` 的 `exclude` 確保測試文件不會被編譯到生產環境
- `vitest.config.ts` 使用獨立配置來運行測試
- 這樣測試文件可以用於 CI/CD，但不會被打包到 dev 或 build 中

### 測試策略

#### 後端測試（Rust）

**位置**：`src-tauri/src/modules/url_detector/mod.rs` (或其他模組的 `#[cfg(test)]` 區段)

**必須包含的測試**：

```rust
#[cfg(test)]
mod tests {
    use super::*;

    /// 1. 序列化測試：確保輸出 camelCase
    #[test]
    fn test_serialization_uses_camel_case() {
        let data = YourStruct { /* ... */ };
        let json = serde_json::to_string(&data).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();

        // 驗證 camelCase 欄位存在
        assert!(parsed.get("isValid").is_some());
        // 驗證 snake_case 欄位不存在
        assert!(parsed.get("is_valid").is_none());
    }

    /// 2. 反序列化測試：確保能接收 camelCase 輸入
    #[test]
    fn test_deserialization_from_camel_case() {
        let json = r#"{"isValid": true, "detectorName": "Test"}"#;
        let data: YourStruct = serde_json::from_str(json).unwrap();
        assert_eq!(data.is_valid, true);
    }

    /// 3. 往返測試：確保資料完整性
    #[test]
    fn test_roundtrip() {
        let original = YourStruct { /* ... */ };
        let json = serde_json::to_string(&original).unwrap();
        let deserialized: YourStruct = serde_json::from_str(&json).unwrap();
        assert_eq!(original.field, deserialized.field);
    }
}
```

**運行測試**：
```bash
cd src-tauri
cargo test

# 只運行序列化測試
cargo test url_detector::tests
```

### 持續整合（CI）

在 `.github/workflows/test.yml` 中添加：

```yaml
name: API Consistency Tests

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
          cargo test
      - name: Check serialization tests
        run: |
          cd src-tauri
          cargo test url_detector::tests -- --nocapture

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
      - name: Verify build excludes tests
        run: pnpm build
```

### 開發流程

每次創建或修改 API 時，遵循以下流程：

1. **定義資料結構**
   ```rust
   #[derive(Serialize, Deserialize)]
   #[serde(rename_all = "camelCase")]  // ← 必須添加
   pub struct NewApiResponse {
       pub is_valid: bool,  // → 序列化為 "isValid"
   }
   ```

2. **撰寫後端測試**
   ```rust
   #[test]
   fn test_new_api_response_serialization() {
       let response = NewApiResponse { is_valid: true };
       let json = serde_json::to_string(&response).unwrap();
       let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();

       // 確保是 camelCase
       assert!(parsed.get("isValid").is_some());
       // 確保沒有 snake_case
       assert!(parsed.get("is_valid").is_none());
   }
   ```

3. **撰寫前端介面**
   ```typescript
   interface NewApiResponse {
     isValid: boolean;  // camelCase，與後端序列化輸出一致
   }
   ```

4. **撰寫前端測試**
   ```typescript
   // src/__tests__/new-api.test.ts
   import { describe, test, expect } from 'vitest';

   describe('NewApiResponse', () => {
     test('should parse backend camelCase JSON', () => {
       const json = '{"isValid": true}';
       const result: NewApiResponse = JSON.parse(json);
       expect(result.isValid).toBe(true);
     });
   });
   ```

5. **運行所有測試**
   ```bash
   # 後端測試
   cd src-tauri && cargo test

   # 前端測試
   cd .. && pnpm test:run
   ```

6. **提交前檢查**
   ```bash
   # 確保所有測試通過
   cd src-tauri && cargo test && cd .. && pnpm test:run

   # 確保構建成功（測試文件會被排除）
   pnpm build
   ```

### 錯誤訊息範例

**好的錯誤訊息**（測試失敗時）：
```
assertion failed: parsed.get("isValid").is_some()
Missing 'isValid' field (should be camelCase, not is_valid)
```

**診斷提示**：
```
⚠️ Found snake_case field "is_valid"
   Backend may be missing #[serde(rename_all = "camelCase")]
   Fix: Add #[serde(rename_all = "camelCase")] to the struct
```

### 自動化工具

創建 pre-commit hook (`.git/hooks/pre-commit`)：

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

**重要**：測試不只是為了捕捉 bug，更是為了確保 API 契約的穩定性。每次修改資料結構時，測試會告訴你是否破壞了前後端的相容性。

---

**詳細測試指南請參閱**: [TESTING.md](./TESTING.md)

<!-- MANUAL ADDITIONS END -->
