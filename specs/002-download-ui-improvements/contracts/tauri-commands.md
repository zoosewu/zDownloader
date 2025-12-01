# Tauri Commands API Contract: Download UI Improvements

**Feature**: Download UI Improvements
**Date**: 2025-12-01
**Version**: 1.0.0

## Overview

此文件定義 Download UI Improvements 功能所需的 Tauri Commands API 合約。包含新增的 commands 和修改的現有 commands。

---

## New Commands

### 1. `detect_url`

檢測 URL 是否為有效的下載連結。

**功能需求映射**:
- FR-009: 剪貼簿監控能偵測包含常見附檔名的直接檔案連結
- FR-012: 系統支援模組化的 URL 檢測機制
- FR-013: URL 檢測區分兩種模式
- FR-015: 剪貼簿偵測到的連結必須經過驗證

**Signature:**
```rust
#[tauri::command]
pub async fn detect_url(
    url: String,
    state: State<'_, ModuleRegistryState>,
) -> Result<UrlDetectionResult, String>
```

**Request:**
```typescript
interface DetectUrlRequest {
  url: string;
}
```

**Response:**
```typescript
interface UrlDetectionResult {
  url: string;
  isValid: boolean;
  detectorName?: string;
  detectionMode: 'FileExtension' | 'ModuleCustom';
  suggestedFilename?: string;
  fileExtension?: string;
  errorMessage?: string;
}
```

**Error Responses:**
```typescript
// Invalid URL format
{ error: "Invalid URL format: {url}" }

// No detector matched
{ error: "No matching detector for URL: {url}" }

// Internal error
{ error: "Internal detection error: {details}" }
```

**Examples:**

**成功 - 附檔名檢測：**
```typescript
// Request
invoke('detect_url', { url: 'https://example.com/file.zip' })

// Response
{
  url: 'https://example.com/file.zip',
  isValid: true,
  detectorName: 'BuiltinFileDetector',
  detectionMode: 'FileExtension',
  suggestedFilename: 'file.zip',
  fileExtension: '.zip'
}
```

**成功 - 模組客製化：**
```typescript
// Request
invoke('detect_url', { url: 'https://youtube.com/watch?v=xxx' })

// Response
{
  url: 'https://youtube.com/watch?v=xxx',
  isValid: true,
  detectorName: 'YouTubeModule',
  detectionMode: 'ModuleCustom',
  suggestedFilename: 'video.mp4',
  fileExtension: '.mp4'
}
```

**失敗 - 無法檢測：**
```typescript
// Request
invoke('detect_url', { url: 'https://example.com/page' })

// Response
{
  url: 'https://example.com/page',
  isValid: false,
  detectionMode: 'FileExtension',
  errorMessage: 'No file extension detected and no module matched'
}
```

**Performance Requirements:**
- Response time: < 100ms for file extension detection
- Response time: < 500ms for module custom detection
- Must be non-blocking (async)

**Security Considerations:**
- URL must be validated before processing
- No execution of JavaScript from URL
- Sanitize all extracted filenames

---

### 2. `get_clipboard_content`

取得當前剪貼簿內容。

**功能需求映射**:
- FR-008: 系統持續監控剪貼簿內容變化

**Signature:**
```rust
#[tauri::command]
pub async fn get_clipboard_content() -> Result<String, String>
```

**Request:**
無參數

**Response:**
```typescript
type ClipboardContent = string;
```

**Error Responses:**
```typescript
// Clipboard access denied
{ error: "Failed to access clipboard: permission denied" }

// Clipboard empty
{ error: "Clipboard is empty" }

// Clipboard contains non-text content
{ error: "Clipboard does not contain text content" }
```

**Examples:**

**成功：**
```typescript
// Request
invoke('get_clipboard_content')

// Response
"https://example.com/file.zip"
```

**失敗 - 空剪貼簿：**
```typescript
// Request
invoke('get_clipboard_content')

// Error
{ error: "Clipboard is empty" }
```

**Performance Requirements:**
- Response time: < 50ms
- Must handle concurrent calls

**Security Considerations:**
- Only return text content (no images, files, etc.)
- Limit content length to 10KB to prevent abuse

---

### 3. `create_download_with_path`

建立下載任務（支援自訂路徑）。

**功能需求映射**:
- FR-004: 新增下載表單包含網址、檔名、路徑欄位
- FR-005: 路徑欄位預設顯示預設下載資料夾
- FR-006: 路徑欄位提供瀏覽功能

**Signature:**
```rust
#[tauri::command]
pub async fn create_download_with_path(
    url: String,
    filename: Option<String>,
    download_path: String,
    state: State<'_, AppState>,
    settings_state: State<'_, SettingsState>,
) -> Result<String, String>
```

**Request:**
```typescript
interface CreateDownloadWithPathRequest {
  url: string;
  filename?: string;
  downloadPath: string;
}
```

**Response:**
```typescript
interface CreateDownloadResponse {
  taskId: string; // UUID
}
```

**Error Responses:**
```typescript
// Invalid URL
{ error: "Invalid URL: {url}" }

// Invalid path
{ error: "Invalid download path: {path}" }

// Path not writable
{ error: "Download path is not writable: {path}" }

// File already exists
{ error: "File already exists: {filepath}" }

// Filename contains invalid characters
{ error: "Invalid filename: {filename}" }
```

**Examples:**

**成功：**
```typescript
// Request
invoke('create_download_with_path', {
  url: 'https://example.com/file.zip',
  filename: 'my-file.zip',
  downloadPath: '/home/user/Downloads'
})

// Response
{
  taskId: '550e8400-e29b-41d4-a716-446655440000'
}
```

**成功 - 自動生成檔名：**
```typescript
// Request
invoke('create_download_with_path', {
  url: 'https://example.com/file.zip',
  downloadPath: '/home/user/Downloads'
})

// Response
{
  taskId: '550e8400-e29b-41d4-a716-446655440001'
}
// filename will be extracted from URL: 'file.zip'
```

**失敗 - 路徑無寫入權限：**
```typescript
// Request
invoke('create_download_with_path', {
  url: 'https://example.com/file.zip',
  downloadPath: '/root/protected'
})

// Error
{ error: "Download path is not writable: /root/protected" }
```

**Performance Requirements:**
- Response time: < 200ms
- Must validate path permissions before creating task

**Security Considerations:**
- Validate download_path to prevent directory traversal
- Check write permissions before task creation
- Sanitize filename to prevent special characters

**Business Logic:**
- If filename is not provided, extract from URL
- If file exists, auto-rename with suffix (e.g., "file (1).zip")
- Create parent directories if they don't exist (with user permission)
- Validate URL format before creating task

---

## Modified Commands

### `create_download` (EXISTING - Enhanced)

現有的 `create_download` command 保持向後相容，但內部邏輯會調用新的路徑驗證。

**功能需求映射**:
- 保持向後相容性

**Current Signature:**
```rust
#[tauri::command]
pub async fn create_download(
    url: String,
    filename: Option<String>,
    state: State<'_, AppState>,
    settings_state: State<'_, SettingsState>,
) -> Result<String, String>
```

**Changes:**
- 內部使用設定中的 `default_download_dir` 作為 `download_path`
- 調用相同的路徑驗證邏輯
- 無 API 變更，完全向後相容

**Note:**
新程式碼應使用 `create_download_with_path` 以支援自訂路徑。`create_download` 保留用於現有程式碼的相容性。

---

## Events (Tauri Emit)

### 1. `clipboard-url-detected` (NEW)

當偵測到剪貼簿中有有效的下載 URL 時觸發。

**功能需求映射**:
- FR-010: 偵測到有效下載連結時自動切換到新增下載頁面
- FR-011: 自動切換時將連結預填入網址欄位

**Payload:**
```typescript
interface ClipboardUrlDetectedEvent {
  url: string;
  detectionResult: UrlDetectionResult;
  timestamp: number; // Unix timestamp (ms)
}
```

**Example:**
```typescript
listen('clipboard-url-detected', (event) => {
  console.log('Detected URL:', event.payload.url);
  console.log('Detection result:', event.payload.detectionResult);
  // Navigate to add-download page and pre-fill URL
});
```

**When Emitted:**
- 當剪貼簿內容變化且 `detect_url` 返回 `isValid: true` 時
- 僅當剪貼簿監控啟用時（設定中）

**Performance:**
- Must emit within 500ms of clipboard change (SC-003)

---

## Data Types

### Common Types

```typescript
// Frontend types (TypeScript)
type PageType = 'main' | 'settings' | 'add-download';

enum DetectionMode {
  FileExtension = 'FileExtension',
  ModuleCustom = 'ModuleCustom'
}

interface UrlDetectionResult {
  url: string;
  isValid: boolean;
  detectorName?: string;
  detectionMode: DetectionMode;
  suggestedFilename?: string;
  fileExtension?: string;
  errorMessage?: string;
}
```

```rust
// Backend types (Rust)
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct UrlDetectionResult {
    pub url: String,
    pub is_valid: bool,
    pub detector_name: Option<String>,
    pub detection_mode: DetectionMode,
    pub suggested_filename: Option<String>,
    pub file_extension: Option<String>,
    pub error_message: Option<String>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum DetectionMode {
    FileExtension,
    ModuleCustom,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ClipboardUrlDetectedEvent {
    pub url: String,
    pub detection_result: UrlDetectionResult,
    pub timestamp: i64,
}
```

---

## Error Handling

### Error Categories

**1. Validation Errors (400-level)**
- Invalid URL format
- Invalid path
- Invalid filename

**2. Permission Errors (403-level)**
- Clipboard access denied
- Path not writable

**3. Not Found Errors (404-level)**
- Task not found
- Path does not exist

**4. Internal Errors (500-level)**
- Detection engine failure
- File system error

### Error Response Format

All errors should return:
```typescript
interface CommandError {
  error: string; // Human-readable error message
  code?: string; // Optional error code for programmatic handling
}
```

---

## API Versioning

**Current Version**: 1.0.0

**Backward Compatibility:**
- All new commands are additions, no breaking changes
- `create_download` maintains full backward compatibility
- Events are new, no existing events modified

**Future Changes:**
- If breaking changes needed, will introduce versioned commands (e.g., `detect_url_v2`)

---

## Testing Requirements

### Unit Tests (Rust)

**Required tests for each command:**
1. Success cases with valid inputs
2. Error cases with invalid inputs
3. Edge cases (empty strings, very long URLs, special characters)
4. Concurrent call handling

**Example test structure:**
```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_detect_url_valid_file_extension() {
        // Test implementation
    }

    #[tokio::test]
    async fn test_detect_url_invalid_format() {
        // Test implementation
    }

    #[tokio::test]
    async fn test_create_download_with_path_success() {
        // Test implementation
    }

    #[tokio::test]
    async fn test_create_download_with_path_invalid_path() {
        // Test implementation
    }
}
```

### Integration Tests (Frontend)

**Required tests:**
1. Full flow: clipboard detection → navigation → create download
2. Error handling for invalid URLs
3. Path validation and user feedback

---

## Performance Benchmarks

| Command | Target | Maximum |
|---------|--------|---------|
| `detect_url` (file ext) | < 50ms | 100ms |
| `detect_url` (module) | < 300ms | 500ms |
| `get_clipboard_content` | < 30ms | 50ms |
| `create_download_with_path` | < 100ms | 200ms |

**Measurement:**
- Benchmarks should be run with `criterion` crate
- Measure p50, p95, and p99 latencies
- Test under concurrent load (10+ simultaneous calls)

---

## Security Checklist

- [x] URL validation to prevent injection
- [x] Path validation to prevent directory traversal
- [x] Filename sanitization to prevent special characters
- [x] Clipboard content size限制 (< 10KB)
- [x] Permission checks before file operations
- [x] No execution of code from URLs
- [x] Proper error messages (no sensitive info leak)

---

## Summary

### New Commands
1. ✅ `detect_url` - URL 檢測
2. ✅ `get_clipboard_content` - 剪貼簿內容
3. ✅ `create_download_with_path` - 建立下載（含路徑）

### Modified Commands
1. ✅ `create_download` - 增強驗證（向後相容）

### New Events
1. ✅ `clipboard-url-detected` - 剪貼簿 URL 偵測事件

### Coverage
所有 15 個功能需求（FR-001 to FR-015）均有對應的 API 支援。
