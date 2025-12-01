# Data Model: Download UI Improvements

**Feature**: Download UI Improvements
**Date**: 2025-12-01
**Status**: Design Complete

## Overview

此文件定義 Download UI Improvements 功能所需的資料結構、狀態管理和實體關係。大部分實體已存在於系統中，此功能主要新增導航狀態管理和 URL 檢測相關的實體。

---

## Entities

### 1. NavigationState (NEW)

導航狀態，管理應用程式當前顯示的頁面。

**屬性：**

| 欄位 | 類型 | 必填 | 說明 |
|------|------|------|------|
| `currentPage` | `PageType` | ✅ | 當前頁面標識 |
| `previousPage` | `PageType?` | ❌ | 上一個頁面（用於返回導航） |
| `navigationParams` | `Record<string, any>?` | ❌ | 頁面參數（如預填的 URL） |

**PageType Enum：**
```typescript
type PageType = 'main' | 'settings' | 'add-download';
```

**驗證規則：**
- `currentPage` 必須是有效的 PageType 值
- `previousPage` 用於實現返回功能

**狀態轉換：**
```
main <-> settings
main -> add-download -> main (完成新增)
add-download -> main (取消)
```

**儲存：**
- 不持久化（僅運行時狀態）
- 使用 Jotai atom 管理（或 React useState）

---

### 2. UrlDetectionResult (NEW)

URL 檢測結果，表示剪貼簿中的 URL 是否為有效的下載連結。

**屬性：**

| 欄位 | 類型 | 必填 | 說明 |
|------|------|------|------|
| `url` | `string` | ✅ | 原始 URL |
| `isValid` | `boolean` | ✅ | 是否為有效下載連結 |
| `detectorName` | `string?` | ❌ | 檢測到的模組名稱（如 "BuiltinFileDetector", "YouTubeModule"） |
| `detectionMode` | `DetectionMode` | ✅ | 檢測模式 |
| `suggestedFilename` | `string?` | ❌ | 建議的檔案名稱（從 URL 提取） |
| `fileExtension` | `string?` | ❌ | 檔案附檔名（如 ".zip"） |
| `errorMessage` | `string?` | ❌ | 檢測失敗的錯誤訊息 |

**DetectionMode Enum：**
```typescript
enum DetectionMode {
  FileExtension = 'FileExtension',  // 基於附檔名檢測
  ModuleCustom = 'ModuleCustom'     // 模組客製化檢測
}
```

**驗證規則：**
- `url` 必須是有效的 URL 格式（符合 FR-015）
- 如果 `isValid` 為 true，`detectorName` 應該存在
- `fileExtension` 應該包含點號（如 ".zip" 而非 "zip"）

**範例：**
```typescript
// 成功檢測 - 附檔名模式
{
  url: "https://example.com/file.zip",
  isValid: true,
  detectorName: "BuiltinFileDetector",
  detectionMode: DetectionMode.FileExtension,
  suggestedFilename: "file.zip",
  fileExtension: ".zip"
}

// 成功檢測 - 模組客製化模式
{
  url: "https://youtube.com/watch?v=xxx",
  isValid: true,
  detectorName: "YouTubeModule",
  detectionMode: DetectionMode.ModuleCustom,
  suggestedFilename: "video.mp4",
  fileExtension: ".mp4"
}

// 檢測失敗
{
  url: "https://example.com/page",
  isValid: false,
  detectionMode: DetectionMode.FileExtension,
  errorMessage: "No file extension detected"
}
```

---

### 3. ClipboardMonitorState (NEW)

剪貼簿監控狀態。

**屬性：**

| 欄位 | 類型 | 必填 | 說明 |
|------|------|------|------|
| `isEnabled` | `boolean` | ✅ | 監控是否啟用（從設定讀取） |
| `lastCheckedContent` | `string?` | ❌ | 上次檢查的剪貼簿內容（用於 debounce） |
| `lastCheckedTimestamp` | `number?` | ❌ | 上次檢查的時間戳（毫秒） |
| `detectedUrl` | `string?` | ❌ | 偵測到的有效 URL |
| `shouldShowHint` | `boolean` | ✅ | 是否顯示剪貼簿提示 |

**驗證規則：**
- `isEnabled` 預設值應從使用者設定讀取
- `lastCheckedContent` 用於避免重複處理相同內容

**狀態轉換：**
```
isEnabled: false -> true (使用者啟用設定)
detectedUrl: null -> "url" (偵測到有效連結)
shouldShowHint: false -> true (detectedUrl 有值時)
shouldShowHint: true -> false (使用者接受或拒絕)
```

---

### 4. AddDownloadFormData (NEW)

新增下載表單的資料結構。

**屬性：**

| 欄位 | 類型 | 必填 | 說明 |
|------|------|------|------|
| `url` | `string` | ✅ | 下載 URL |
| `filename` | `string?` | ❌ | 自訂檔案名稱（可選） |
| `downloadPath` | `string` | ✅ | 下載儲存路徑 |
| `scheduledStart` | `string?` | ❌ | 排程開始時間（ISO 8601） |

**驗證規則：**
- `url` 必須是有效的 URL 格式（FR-015）
- `url` 必須通過 URL 檢測（FR-009, FR-012）
- `downloadPath` 必須是有效的目錄路徑
- `downloadPath` 預設值從設定中的 `defaultDownloadFolder` 讀取（FR-005）
- `filename` 如果為空，從 `url` 自動提取
- `scheduledStart` 如果有值，必須是未來時間

**與 DownloadTask 的關係：**
- `AddDownloadFormData` 是用於建立 `DownloadTask` 的輸入資料
- 提交後會呼叫後端 command 建立完整的 `DownloadTask` 實體

---

### 5. UrlDetectorModule (NEW - Rust Backend)

URL 檢測模組的介面定義（Rust Trait）。

**Trait 定義：**
```rust
pub trait UrlDetector: Send + Sync {
    /// Module name identifier
    fn name(&self) -> &str;

    /// Check if this module can handle the given URL
    fn can_handle(&self, url: &str) -> bool;

    /// Validate and extract information from URL
    fn validate(&self, url: &str) -> Result<UrlDetectionInfo, DetectionError>;

    /// Get supported file extensions (for file-based detectors)
    fn supported_extensions(&self) -> Vec<&str> {
        vec![]
    }
}
```

**UrlDetectionInfo Struct：**
```rust
pub struct UrlDetectionInfo {
    pub url: String,
    pub is_valid: bool,
    pub detector_name: String,
    pub detection_mode: DetectionMode,
    pub suggested_filename: Option<String>,
    pub file_extension: Option<String>,
}
```

**DetectionMode Enum：**
```rust
pub enum DetectionMode {
    FileExtension,
    ModuleCustom,
}
```

**內建實作：**
- `BuiltinFileDetector`：檢測常見附檔名
  - 支援的附檔名：`.zip`, `.pdf`, `.exe`, `.mp4`, `.mkv`, `.rar`, `.7z`, `.tar`, `.gz`, `.iso`, `.dmg`, `.apk` （spec.md 第 146 行）

---

## Existing Entities (Modified)

### DownloadTask (EXISTING - No Changes)

此功能不需要修改現有的 `DownloadTask` 結構（src/types/download.ts 第 23-43 行）。

**相關欄位：**
- `url`: 由 `AddDownloadFormData.url` 提供
- `filename`: 由 `AddDownloadFormData.filename` 提供（或自動生成）
- `filepath`: 由 `AddDownloadFormData.downloadPath` + `filename` 組成

---

## State Management

### Frontend State (Jotai Atoms)

**新增 Atoms：**

1. **navigationStore.ts**
   ```typescript
   export const currentPageAtom = atom<PageType>('main');
   export const previousPageAtom = atom<PageType | null>(null);
   export const navigationParamsAtom = atom<Record<string, any>>({});
   ```

2. **clipboardStore.ts**（或整合到 downloadStore.ts）
   ```typescript
   export const clipboardMonitorEnabledAtom = atom<boolean>(false);
   export const lastClipboardContentAtom = atom<string | null>(null);
   export const detectedClipboardUrlAtom = atom<string | null>(null);
   export const showClipboardHintAtom = atom<boolean>(false);
   ```

3. **addDownloadStore.ts**
   ```typescript
   export const addDownloadFormAtom = atom<AddDownloadFormData>({
     url: '',
     filename: null,
     downloadPath: '', // Will be initialized from settings
     scheduledStart: null,
   });
   ```

**現有 Atoms（修改）：**
- `downloadStore.ts` 中的 atoms 保持不變
- `settingsStore.ts` 已包含 `defaultDownloadFolder`，無需修改

---

## Backend State (Rust)

**Module Registry：**
```rust
pub struct ModuleRegistry {
    detectors: Vec<Box<dyn UrlDetector>>,
}

impl ModuleRegistry {
    pub fn new() -> Self {
        let mut registry = Self { detectors: vec![] };
        // Register builtin detector
        registry.register(Box::new(BuiltinFileDetector::new()));
        registry
    }

    pub fn register(&mut self, detector: Box<dyn UrlDetector>) {
        self.detectors.push(detector);
    }

    pub fn detect_url(&self, url: &str) -> Result<UrlDetectionInfo, DetectionError> {
        for detector in &self.detectors {
            if detector.can_handle(url) {
                return detector.validate(url);
            }
        }
        Err(DetectionError::NoMatchingDetector)
    }
}
```

**全域狀態：**
- `ModuleRegistry` 在應用程式啟動時初始化
- 儲存在 Tauri `State` 中供 commands 使用

---

## Relationships

### Entity Relationship Diagram

```
┌─────────────────────┐
│ NavigationState     │
│ - currentPage       │
│ - previousPage      │
│ - navigationParams  │
└──────────┬──────────┘
           │
           │ navigates to
           ▼
┌─────────────────────┐         ┌──────────────────────┐
│ AddDownloadPage     │◄────────│ ClipboardMonitor     │
│                     │         │ - detectedUrl        │
└──────────┬──────────┘         │ - shouldShowHint     │
           │                    └──────────────────────┘
           │ submits
           ▼
┌─────────────────────┐
│ AddDownloadFormData │
│ - url               │
│ - filename          │─────┐
│ - downloadPath      │     │
│ - scheduledStart    │     │
└─────────────────────┘     │
           │                │
           │ creates        │ validates via
           ▼                ▼
┌─────────────────────┐  ┌──────────────────────┐
│ DownloadTask        │  │ UrlDetectionResult   │
│ (existing)          │  │ - isValid            │
└─────────────────────┘  │ - detectorName       │
                         │ - suggestedFilename  │
                         └───────────┬──────────┘
                                     │
                                     │ produced by
                                     ▼
                         ┌──────────────────────┐
                         │ UrlDetectorModule    │
                         │ (Trait)              │
                         ├──────────────────────┤
                         │ BuiltinFileDetector  │
                         │ (Implementation)     │
                         └──────────────────────┘
```

---

## Data Flow

### 1. 剪貼簿監控流程

```
[Clipboard Changes]
    ↓
[React useEffect - 400ms polling]
    ↓
[Call Tauri: get_clipboard_content()]
    ↓
[Compare with lastCheckedContent]
    ↓ (if different)
[Call Tauri: detect_url(content)]
    ↓
[UrlDetectionResult returned]
    ↓ (if isValid)
[Update detectedClipboardUrlAtom]
    ↓
[Update showClipboardHintAtom = true]
    ↓
[User accepts]
    ↓
[Navigate to add-download page]
    ↓
[Pre-fill url in AddDownloadFormData]
```

### 2. 新增下載流程

```
[User clicks "Add Download" button]
    ↓
[Update currentPageAtom = 'add-download']
    ↓
[Load defaultDownloadFolder from settings]
    ↓
[Initialize AddDownloadFormData]
    ↓
[User fills form]
    ↓
[Validate inputs (FR-015)]
    ↓
[Call Tauri: create_download_task(formData)]
    ↓
[Create DownloadTask on backend]
    ↓
[Auto-start download (invoke start_download)]
    ↓
[Update currentPageAtom = 'main']
    ↓
[Refresh download list]
```

---

## Validation Rules Summary

### URL Validation（FR-015）
- 必須符合 URL 格式規範（使用 `url` crate 或 regex）
- 必須包含 schema（http/https）
- 不允許 javascript: 或 data: schema（安全考量）

### Path Validation
- 必須是有效的檔案系統路徑
- 必須是目錄（非檔案）
- 建議檢查寫入權限（graceful degradation）

### Filename Validation
- 不允許路徑分隔符號（/ 或 \）
- 不允許特殊字元（依作業系統而定）
- 長度限制：1-255 字元

---

## Storage

### Persistent Storage

無新增持久化儲存需求。所有新狀態都是運行時狀態。

**現有儲存（不變）：**
- `settings.json`：使用者設定（包含 `defaultDownloadFolder`）
- Download history：`DownloadTask` 列表

### Runtime Storage

**Frontend（Memory）：**
- Jotai atoms（navigation, clipboard, form state）

**Backend（Memory）：**
- `ModuleRegistry`：URL 檢測模組註冊表

---

## Migration

無資料遷移需求。此功能為純新增功能，不修改現有資料結構。

---

## Summary

### 新增實體
1. ✅ `NavigationState`：頁面導航狀態
2. ✅ `UrlDetectionResult`：URL 檢測結果
3. ✅ `ClipboardMonitorState`：剪貼簿監控狀態
4. ✅ `AddDownloadFormData`：新增下載表單資料
5. ✅ `UrlDetectorModule` (Rust Trait)：URL 檢測模組介面

### 修改實體
無（所有現有實體保持不變）

### 新增 State Management
- Frontend: 3 個新 Jotai store files（navigationStore, clipboardStore, addDownloadStore）
- Backend: ModuleRegistry 全域狀態

### 資料完整性
所有驗證規則已定義，確保資料完整性和安全性。
