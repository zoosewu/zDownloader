# zoo-downloader Development Guidelines

你是 Linus Torvalds，總是以簡潔明瞭的方式撰寫技術文件以及程式碼，每當你檢視程式碼的時候，都會檢視程式碼是否足夠簡潔、符合最佳實踐，並且會建議改進的地方。


前端和 tauri 接口對接時，永遠使用「優雅降級」模式，確保用戶在 Tauri API 尚未完全初始化時會看到清楚的錯誤訊息，而不是技術性的異常。

## 先讀這些（2026-10-30 架構檢視後）

- `docs/architecture/review-2026-10-30.md`：已知問題清單（SEC / A / B / C / D / SET / SPEC 編號）、候選改善項 CAND-1..8、建議階段。修改相關程式前先查，修正後在該條目標註。
- `GLOSSARY.md`：領域術語。`docs/adr/`：已定案的決策，不要重新提議被否決的方案。
- 本檔下方的 specs / tasks 完成度描述與實際狀態不符，以檢視紀錄為準。

Auto-generated from all feature plans. Last updated: 2025-11-27

## Active Technologies
- JSON files (settings, download history) + file system (download data) (002-download-ui-improvements)
- TypeScript 5.8.3 (frontend), Rust 1.75+ (backend - Tauri) (003-download-list-redesign)
- JSON files (download history, settings) + in-memory state (Jotai atoms) (003-download-list-redesign)

- (001-core-downloader)

## Project Structure

```text
src/
tests/
```

## Commands

```bash
cd src-tauri && cargo test --workspace   # zoo-core + shell（含 bindings 一致性、enum 契約）
UPDATE_BINDINGS=1 cargo test bindings     # Rust DTO 變更後重新產生 src/backend/bindings
pnpm test:run                       # 前端測試（vitest，含命令名稱一致性）
pnpm exec tsc --noEmit              # 型別檢查
pnpm build                          # 生產建置
```

Windows 注意：v8 crate 在 target 目錄與 cargo registry 位於不同磁碟時需要建立 symlink（未開啟開發人員模式會失敗，錯誤 1314），可設 `CARGO_TARGET_DIR` 到與 registry 同一磁碟。

前後端介面：前端只透過 `src/backend` 呼叫後端，型別一律來自 ts-rs 產生的 `src/backend/bindings`（不要手寫或手改）。新增 Tauri command 時同步加入 `generate_handler!` 與 `src/backend/index.ts`。下載核心在 `src-tauri/crates/zoo-core`（不依賴 Tauri），shell 只做轉接。

## Code Style

: Follow standard conventions

## Recent Changes
- 003-download-list-redesign: Added TypeScript 5.8.3 (frontend), Rust 1.75+ (backend - Tauri)
- 002-download-ui-improvements: Added JSON files (settings, download history) + file system (download data)


<!-- MANUAL ADDITIONS START -->

## API 命名規範與序列化 (Critical)

### 規則：前端 camelCase，後端 snake_case + Serde 轉換

> **enum 值也會被轉換**：`rename_all = "camelCase"` 會讓 `DownloadStatus::Downloading` 序列化成 `"downloading"`，TS enum 的值必須是 `'downloading'`，並在程式中引用 enum 成員而非寫死字串。新增或修改 enum 時同步更新 `test-fixtures/api-samples.json` 的 `enums` 段（Rust 與 TS 兩邊的契約測試都會比對它）。

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

## Download List UI Redesign (Feature 003)

### Overview
完整重新設計下載列表 UI，從卡片視圖改為表格視圖，並添加選擇功能和詳細信息面板。

### Architecture

#### Core Components

**DownloadTable** (`src/components/DownloadTable.tsx`)
- 主表格組件，使用 MUI Table 與 sticky header
- 顯示 7 個欄位：檔名、大小、進度、速度、狀態、建立時間、剩餘時間
- 自動處理空狀態
- 整合 useSelectedDownload hook 進行選擇管理

**DownloadTableRow** (`src/components/DownloadTableRow.tsx`)
- 單一表格行組件，使用 React.memo 優化
- 自定義比較函數僅在關鍵屬性變更時重新渲染
- 支持點擊選擇與高亮顯示
- 包含 Tooltip 顯示完整信息

**DownloadDetailPanel** (`src/components/DownloadDetailPanel.tsx`)
- 詳細信息面板，使用 MUI Tabs 進行導航
- 4 個 Tabs：基本資訊、分段視覺化、錯誤資訊、進階資訊
- 自動切換到第一個 tab 當選擇改變時
- 空狀態處理當沒有選擇時

**BasicInfoTab** (`src/components/BasicInfoTab.tsx`)
- 顯示 URL、檔案路徑、大小、時間戳
- 使用可點擊的 Link 組件打開 URL
- Monospace 字體顯示檔案路徑

**SegmentVisualization** (`src/components/SegmentVisualization.tsx`)
- CSS Grid 布局顯示所有分段
- 固定大小 12px 方格，2px 間距
- 圖例顯示顏色含義
- 可滾動處理大量分段

**SegmentGridItem** (`src/components/SegmentGridItem.tsx`)
- 單一分段方格，使用 React.memo
- MUI Tooltip 顯示詳細信息
- Hover 效果：scale(1.3)
- 顏色映射：灰(Pending)、藍(Downloading)、綠(Completed)、紅(Failed)

**ErrorInfoTab** (`src/components/ErrorInfoTab.tsx`)
- 顯示錯誤訊息或成功狀態
- 使用 MUI Alert 組件
- 顯示重試次數（如果有）

**AdvancedInfoTab** (`src/components/AdvancedInfoTab.tsx`)
- 顯示重試信息、排程時間、分段數量
- Chip 組件顯示統計數據
- Monospace 顯示 Task ID

#### Utilities

**downloadCalculations.ts** (`src/utils/downloadCalculations.ts`)
- `calculateRemainingTime()`: 計算剩餘時間（秒）
- `formatRemainingTime()`: 格式化為人類可讀格式
- `formatBytes()`: 格式化字節大小
- `formatSpeed()`: 格式化速度
- `formatTaskForTable()`: 將 DownloadTask 格式化為表格顯示數據
- `getSegmentColor()`: 根據狀態返回顏色
- `formatSegmentForVisualization()`: 格式化分段數據供視覺化使用

**formatters.ts** (`src/utils/formatters.ts`)
- `formatRelativeTime()`: 將 ISO 8601 時間戳轉換為相對時間（如 "5分鐘前"）
- `formatAbsoluteTime()`: 格式化為絕對時間
- `formatTimeWithTooltip()`: 返回相對和絕對時間的物件

#### State Management

**downloadStore.ts** (`src/stores/downloadStore.ts`)
- `selectedTaskIdAtom`: 當前選中的任務 ID
- `selectedTaskAtom`: 當前選中的任務物件（派生 atom）
- 自動響應 downloadTasksAtom 變化

**useSelectedDownload.ts** (`src/hooks/useSelectedDownload.ts`)
- `selectTask()`: 選擇任務
- `clearSelection()`: 清除選擇
- `toggleSelection()`: 切換選擇
- `isSelected()`: 檢查任務是否被選中
- 自動清除選擇當任務被刪除時

### Performance Optimizations

1. **React.memo**: DownloadTableRow 和 SegmentGridItem 使用 React.memo 避免不必要的重新渲染
2. **Custom Comparison**: DownloadTableRow 使用自定義比較函數僅在關鍵屬性變更時更新
3. **Jotai Atoms**: 使用細粒度的原子狀態管理，僅訂閱需要的數據
4. **CSS Grid**: 使用原生 CSS Grid 而非 Flexbox 提升大量元素的性能
5. **Sticky Header**: 表格頭部固定，改善大量下載時的用戶體驗

### Internationalization (i18n)

所有新增的 UI 文本都已添加翻譯：
- **中文繁體** (`src/locales/zh-TW.json`): 完整翻譯
- **英文** (`src/locales/en.json`): 完整翻譯
- **日文** (`src/locales/ja.json`): 完整翻譯

關鍵翻譯鍵：
- `common.progress`: 進度
- `downloads.status.title`: 狀態標題
- `downloads.details.*`: 所有詳細面板文本（27 個鍵）

### Theme Support

所有組件使用 Material-UI 的 `sx` prop 和主題變量，自動支持深淺色模式：
- `backgroundColor`: 使用 `background.default` 等主題顏色
- `color`: 使用 `text.primary`, `text.secondary` 等
- `borderColor`: 使用 `divider` 主題顏色

無需額外配置即可支持主題切換。

### Testing

**TypeScript 編譯**: `pnpm exec tsc --noEmit` ✅ 無錯誤
**生產構建**: `pnpm run build` ✅ 成功

### Known Limitations

1. **虛擬化**: 目前分段視覺化使用簡單的 CSS Grid，對於極大量分段（> 1000）可能需要虛擬化
2. **排序**: 表格目前不支持排序功能
3. **過濾**: 表格不支持內建過濾功能

### Future Enhancements

1. 添加表格排序功能
2. 添加列顯示/隱藏設置
3. 添加更多過濾選項
4. 實現虛擬滾動處理大量下載
5. 添加批量操作功能

**詳細測試指南請參閱**: [TESTING.md](./TESTING.md)

---

## 模組開發者體驗 (Feature 004 - Module System)

### TypeScript 型別定義維護規則

當實作或修改模組系統相關功能時，**必須**檢查並更新以下檔案以保持同步：

#### 檢查清單

**每次修改 Context API、模組接口或資料結構時，執行以下檢查：**

1. **型別定義檔** (`modules/@types/zoo-downloader/index.d.ts`)
   - [ ] Context 接口是否反映最新的 API？
   - [ ] HttpClient, Logger, CaptchaClient 等接口是否完整？
   - [ ] DownloadInfo, HttpResponse 等資料型別是否正確？
   - [ ] 所有新增的欄位都有 JSDoc 註解？
   - [ ] 所有範例程式碼是否可執行？
   - [ ] 全域型別別名是否需要更新？

2. **VS Code 程式碼片段** (`modules/.vscode/zoo-downloader.code-snippets`)
   - [ ] 是否需要新增常用模式的 snippet？
   - [ ] 現有 snippets 是否反映最新 API？
   - [ ] Placeholder 變數是否正確？

3. **模組模板** (`modules/_template/`)
   - [ ] index.js 是否使用最新的型別定義？
   - [ ] manifest.json 是否包含所有新增的權限？
   - [ ] README.md 是否說明新功能？

4. **範例模組** (`modules/k2s/`)
   - [ ] 是否展示新 API 的最佳實踐？
   - [ ] JSDoc 註解是否完整？
   - [ ] 是否包含錯誤處理範例？

5. **開發者文件** (`specs/004-site-module-integration/quickstart.md`)
   - [ ] 是否更新 API 參考？
   - [ ] 是否提供新功能的使用範例？
   - [ ] 是否更新限制和注意事項？

#### 自動化檢查

**在提交前執行**：

```bash
# 檢查型別定義語法
cd modules/@types/zoo-downloader
tsc --noEmit index.d.ts

# 驗證 snippets JSON 格式
cat modules/.vscode/zoo-downloader.code-snippets | jq . > /dev/null

# 驗證模板檔案
cd modules/_template
node -c index.js  # 檢查 JavaScript 語法
jq . manifest.json > /dev/null  # 檢查 JSON 格式
```

#### 常見更新場景

**場景 1：新增 Context API**

```rust
// 後端新增 API
#[op2]
async fn op_http_delete(url: String) -> Result<HttpResponse, Error> { ... }
```

**必須更新**：
1. ✅ `modules/@types/zoo-downloader/index.d.ts` - 添加 `delete()` 方法
2. ✅ `modules/.vscode/zoo-downloader.code-snippets` - 添加 `zoo-http-delete` snippet
3. ✅ `modules/_template/index.js` - 在註解中提示可用
4. ✅ `quickstart.md` - 在 API 參考中說明

**場景 2：修改資料結構**

```rust
// 後端修改
pub struct DownloadInfo {
    pub download_url: String,
    pub filename: String,
    pub filesize: Option<u64>,
    pub headers: HashMap<String, String>,
    pub expiry_time: Option<DateTime<Utc>>,  // ← 新增欄位
}
```

**必須更新**：
1. ✅ `modules/@types/zoo-downloader/index.d.ts` - 添加 `expiryTime?: string`
2. ✅ `modules/_template/index.js` - 在返回範例中添加
3. ✅ `modules/k2s/index.js` - 展示如何使用（如果適用）

**場景 3：新增權限類型**

```rust
// 後端新增權限
pub struct ModulePermissions {
    pub network: NetworkPermission,
    pub credentials: Option<CredentialPermission>,
    pub storage: Option<StoragePermission>,  // ← 新增
}
```

**必須更新**：
1. ✅ `modules/@types/zoo-downloader/index.d.ts` - 添加 `storage?` 到 manifest 型別
2. ✅ `modules/_template/manifest.json` - 在註解中說明
3. ✅ `quickstart.md` - 在權限章節說明

#### 驗證流程

**完成模組系統功能後，執行以下驗證**：

1. **型別定義測試**
   ```bash
   # 建立測試檔案
   cat > /tmp/test-module.js << 'EOF'
   /// <reference types="./modules/@types/zoo-downloader" />

   /** @type {import('zoo-downloader').SiteModule} */
   export class TestModule {
     canHandle(url) { return true; }
     async extractDownloadInfo(url, context) {
       // 測試所有新 API
       await context.http.get(url);
       context.log.info('test');
       return { downloadUrl: '', filename: '', filesize: null, headers: {} };
     }
   }
   EOF

   # 用 VS Code 打開，檢查是否有型別錯誤
   code /tmp/test-module.js
   ```

2. **Snippet 測試**
   - 在 VS Code 中創建新 `.js` 檔案
   - 輸入每個 snippet prefix（如 `zoo-module`）
   - 確認生成的程式碼正確且可執行

3. **文件測試**
   - 讓不熟悉專案的人閱讀 quickstart.md
   - 確認他們能成功建立一個基本模組

#### 維護週期

- **即時更新**：每次修改模組 API 時立即更新
- **Phase 完成時檢查**：每個 Phase 完成後完整檢查一次
- **Release 前審查**：正式發布前完整審查所有開發者文件

### Code Snippets 設計原則

1. **簡潔優先**：snippets 應該生成最少但可用的程式碼
2. **提供佔位符**：使用 `${1:description}` 提供有意義的預設值
3. **包含註解**：關鍵位置加入註解提示
4. **遵循最佳實踐**：生成的程式碼應展示正確的錯誤處理和型別使用
5. **保持更新**：API 變更時立即更新對應的 snippets

### 型別定義撰寫原則

1. **完整的 JSDoc**：每個接口、方法、屬性都要有說明
2. **實用的範例**：在 `@example` 中提供真實可用的程式碼
3. **清晰的型別**：避免過度使用 `any`，提供具體型別
4. **一致的命名**：遵循 camelCase 慣例
5. **版本註記**：重大變更時在註解中標記版本

**重要**：這些檔案是模組開發者的主要參考，必須保持高品質和最新狀態。

<!-- MANUAL ADDITIONS END -->
