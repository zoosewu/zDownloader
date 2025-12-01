# Frontend API Contract: Download UI Improvements

**Feature**: Download UI Improvements
**Date**: 2025-12-01
**Version**: 1.0.0

## Overview

此文件定義 Download UI Improvements 功能的前端 API 合約，包含新增的 React hooks、components 和 utility functions。

---

## React Hooks

### 1. `useClipboard`

管理剪貼簿監控功能。

**功能需求映射**:
- FR-008: 系統持續監控剪貼簿內容變化
- FR-010: 偵測到有效下載連結時自動切換頁面
- FR-011: 將偵測到的連結預填入網址欄位

**Location**: `src/hooks/useClipboard.ts`

**Interface:**
```typescript
interface UseClipboardReturn {
  // State
  isMonitoring: boolean;
  detectedUrl: string | null;
  lastCheckTime: number | null;

  // Actions
  startMonitoring: () => void;
  stopMonitoring: () => void;
  clearDetectedUrl: () => void;
}

function useClipboard(): UseClipboardReturn;
```

**Behavior:**
- Polls clipboard every 400ms when monitoring is enabled
- Automatically calls `detect_url` when clipboard content changes
- Emits navigation signal when valid URL is detected
- Implements debounce to avoid processing same content multiple times

**Dependencies:**
- Tauri commands: `get_clipboard_content`, `detect_url`
- Settings: `clipboardMonitoringEnabled` from settings store

**Example Usage:**
```typescript
function AddDownloadPage() {
  const { detectedUrl, clearDetectedUrl } = useClipboard();

  useEffect(() => {
    if (detectedUrl) {
      // Pre-fill URL field
      setFormUrl(detectedUrl);
      clearDetectedUrl();
    }
  }, [detectedUrl]);

  return <AddDownloadForm initialUrl={detectedUrl || ''} />;
}
```

**Performance Considerations:**
- Must use `useRef` to avoid re-creating interval on every render
- Must cleanup interval on unmount
- Polling interval: 400ms (configurable via constant)

---

### 2. `useNavigation`

管理頁面導航。

**功能需求映射**:
- FR-002: 主畫面右上角提供新增下載按鈕
- FR-003: 點擊新增下載按鈕時切換到新增下載頁面
- FR-007: 成功提交後自動切換回下載列表頁面
- FR-014: 提供取消/返回功能

**Location**: `src/hooks/useNavigation.ts`

**Interface:**
```typescript
interface UseNavigationReturn {
  // State
  currentPage: PageType;
  previousPage: PageType | null;
  navigationParams: Record<string, any>;

  // Actions
  navigateTo: (page: PageType, params?: Record<string, any>) => void;
  goBack: () => void;
  canGoBack: boolean;
}

type PageType = 'main' | 'settings' | 'add-download';

function useNavigation(): UseNavigationReturn;
```

**Behavior:**
- Updates `currentPageAtom` when navigating
- Stores previous page for back navigation
- Supports passing parameters between pages
- Integrates with App.tsx for page rendering

**Example Usage:**
```typescript
function MainPage() {
  const { navigateTo } = useNavigation();

  const handleAddDownload = () => {
    navigateTo('add-download', { source: 'manual' });
  };

  return (
    <Button onClick={handleAddDownload}>
      新增下載
    </Button>
  );
}

function AddDownloadPage() {
  const { goBack, canGoBack, navigationParams } = useNavigation();

  const handleCancel = () => {
    if (canGoBack) {
      goBack();
    }
  };

  return (
    <Button onClick={handleCancel}>取消</Button>
  );
}
```

---

### 3. `useDownloadPath`

管理下載路徑選擇。

**功能需求映射**:
- FR-005: 下載路徑欄位預設顯示預設下載資料夾
- FR-006: 下載路徑欄位提供瀏覽按鈕開啟資料夾選擇器

**Location**: `src/hooks/useDownloadPath.ts`

**Interface:**
```typescript
interface UseDownloadPathReturn {
  // State
  downloadPath: string;
  isValid: boolean;
  errorMessage: string | null;

  // Actions
  setDownloadPath: (path: string) => void;
  openDirectoryPicker: () => Promise<void>;
  resetToDefault: () => void;
  validatePath: () => Promise<boolean>;
}

function useDownloadPath(initialPath?: string): UseDownloadPathReturn;
```

**Behavior:**
- Initializes with default download folder from settings if no initialPath provided
- Validates path on change (checks if directory exists and is writable)
- Integrates with Tauri dialog plugin for directory selection

**Dependencies:**
- Tauri plugin: `@tauri-apps/plugin-dialog`
- Settings: `defaultDownloadFolder` from settings store

**Example Usage:**
```typescript
function AddDownloadForm() {
  const settings = useAtomValue(settingsAtom);
  const {
    downloadPath,
    isValid,
    errorMessage,
    openDirectoryPicker
  } = useDownloadPath(settings.defaultDownloadFolder);

  return (
    <>
      <TextField
        label="下載路徑"
        value={downloadPath}
        error={!isValid}
        helperText={errorMessage}
      />
      <Button onClick={openDirectoryPicker}>
        瀏覽
      </Button>
    </>
  );
}
```

---

## React Components

### 1. `AddDownloadPage` (NEW)

新增下載的獨立頁面。

**功能需求映射**:
- FR-003: 點擊新增下載按鈕時切換到新增下載頁面
- FR-004: 新增下載表單包含網址輸入、檔案名稱、下載路徑欄位

**Location**: `src/pages/AddDownloadPage.tsx`

**Props:**
```typescript
interface AddDownloadPageProps {
  // No props needed (gets data from navigation params and clipboard)
}
```

**Behavior:**
- Displays `AddDownloadForm` component
- Handles form submission
- Navigates back to main page on success or cancel
- Shows loading state during submission

**Layout:**
```tsx
<Container>
  <Box>
    <Typography variant="h4">新增下載</Typography>
    <Button onClick={goBack}>返回</Button>
  </Box>

  <AddDownloadForm
    initialUrl={detectedUrl}
    onSubmit={handleSubmit}
    onCancel={goBack}
  />
</Container>
```

---

### 2. `AddDownloadForm` (NEW)

新增下載的表單組件。

**功能需求映射**:
- FR-004: 表單包含網址、檔案名稱、路徑欄位
- FR-005: 路徑欄位預設顯示預設下載資料夾
- FR-006: 路徑欄位提供瀏覽按鈕

**Location**: `src/components/AddDownloadForm.tsx`

**Props:**
```typescript
interface AddDownloadFormProps {
  initialUrl?: string;
  onSubmit: (data: AddDownloadFormData) => Promise<void>;
  onCancel: () => void;
}

interface AddDownloadFormData {
  url: string;
  filename?: string;
  downloadPath: string;
}
```

**Features:**
- URL input with validation
- Optional filename input (auto-filled from URL if empty)
- Download path input with folder picker
- Submit and cancel buttons
- Form validation before submission

**Example:**
```tsx
<AddDownloadForm
  initialUrl="https://example.com/file.zip"
  onSubmit={async (data) => {
    await invoke('create_download_with_path', data);
    navigateTo('main');
  }}
  onCancel={() => goBack()}
/>
```

**Validation Rules:**
- URL: Must be valid format (FR-015)
- Filename: Optional, auto-extracted from URL if empty
- Path: Must be valid directory with write permission

---

### 3. `MainPage` (MODIFIED)

主頁面 - 移除內聯的 UrlInput，新增「新增下載」按鈕。

**功能需求映射**:
- FR-001: 主畫面預設顯示下載列表
- FR-002: 主畫面右上角提供新增下載按鈕

**Location**: `src/pages/MainPage.tsx`

**Changes:**

**Before:**
```tsx
<Container>
  <Typography variant="h4">Zoo Downloader</Typography>
  <UrlInput onSubmit={handleSubmit} />  {/* REMOVE */}
  <DownloadList />
</Container>
```

**After:**
```tsx
<Container>
  <Box sx={{ display: 'flex', justifyContent: 'space-between' }}>
    <Typography variant="h4">Zoo Downloader</Typography>
    <Button
      variant="contained"
      onClick={() => navigateTo('add-download')}
    >
      新增下載
    </Button>
  </Box>

  <DownloadList />
</Container>
```

---

## Utility Functions

### 1. `urlDetector`

URL 檢測輔助工具（前端快速驗證）。

**Location**: `src/utils/urlDetector.ts`

**Interface:**
```typescript
interface UrlDetectorUtils {
  isValidUrl: (url: string) => boolean;
  extractFilename: (url: string) => string | null;
  getFileExtension: (url: string) => string | null;
  hasCommonFileExtension: (url: string) => boolean;
}

export const urlDetector: UrlDetectorUtils;
```

**Functions:**

**`isValidUrl(url: string): boolean`**
- Quick client-side URL format validation
- Uses regex or URL constructor
- Returns true if URL has valid format

**`extractFilename(url: string): string | null`**
- Extracts filename from URL path
- Example: `https://example.com/file.zip` → `file.zip`
- Returns null if no filename detected

**`getFileExtension(url: string): string | null`**
- Extracts file extension from URL
- Example: `https://example.com/file.zip` → `.zip`
- Returns null if no extension

**`hasCommonFileExtension(url: string): boolean`**
- Checks if URL has common download file extension
- Extensions: `.zip`, `.pdf`, `.exe`, `.mp4`, etc. (from spec assumptions)

**Example Usage:**
```typescript
import { urlDetector } from '@/utils/urlDetector';

if (urlDetector.isValidUrl(clipboardContent)) {
  if (urlDetector.hasCommonFileExtension(clipboardContent)) {
    // Likely a download link, proceed with detection
    const result = await invoke('detect_url', { url: clipboardContent });
  }
}
```

---

### 2. `pathValidator`

路徑驗證輔助工具。

**Location**: `src/utils/pathValidator.ts`

**Interface:**
```typescript
interface PathValidatorUtils {
  isValidPath: (path: string) => boolean;
  normalizedPath: (path: string) => string;
  getPathSeparator: () => string;
}

export const pathValidator: PathValidatorUtils;
```

**Functions:**

**`isValidPath(path: string): boolean`**
- Basic path format validation
- Checks for invalid characters
- Platform-agnostic

**`normalizePath(path: string): string`**
- Normalizes path separators for current platform
- Removes trailing slashes
- Resolves `..` and `.`

**`getPathSeparator(): string`**
- Returns platform-specific path separator
- `/` for Linux/Mac, `\` for Windows

---

## Store (Jotai Atoms)

### 1. `navigationStore.ts` (NEW)

**Location**: `src/stores/navigationStore.ts`

**Atoms:**
```typescript
export const currentPageAtom = atom<PageType>('main');

export const previousPageAtom = atom<PageType | null>(null);

export const navigationParamsAtom = atom<Record<string, any>>({});

// Derived atom for navigation
export const navigationAtom = atom(
  (get) => ({
    currentPage: get(currentPageAtom),
    previousPage: get(previousPageAtom),
    params: get(navigationParamsAtom),
  }),
  (get, set, update: { page: PageType; params?: Record<string, any> }) => {
    const currentPage = get(currentPageAtom);
    set(previousPageAtom, currentPage);
    set(currentPageAtom, update.page);
    set(navigationParamsAtom, update.params || {});
  }
);
```

---

### 2. `clipboardStore.ts` (NEW)

**Location**: `src/stores/clipboardStore.ts`

**Atoms:**
```typescript
export const clipboardMonitorEnabledAtom = atom<boolean>((get) => {
  const settings = get(settingsAtom);
  return settings.clipboardMonitoringEnabled;
});

export const lastClipboardContentAtom = atom<string | null>(null);

export const detectedClipboardUrlAtom = atom<string | null>(null);

export const showClipboardHintAtom = atom<boolean>(false);

export const clipboardDetectionResultAtom = atom<UrlDetectionResult | null>(null);
```

---

### 3. `addDownloadStore.ts` (NEW)

**Location**: `src/stores/addDownloadStore.ts`

**Atoms:**
```typescript
export const addDownloadFormAtom = atom<AddDownloadFormData>({
  url: '',
  filename: null,
  downloadPath: '',
});

// Derived atom with default path from settings
export const addDownloadFormWithDefaultsAtom = atom(
  (get) => {
    const form = get(addDownloadFormAtom);
    const settings = get(settingsAtom);

    return {
      ...form,
      downloadPath: form.downloadPath || settings.defaultDownloadFolder,
    };
  },
  (_get, set, update: Partial<AddDownloadFormData>) => {
    set(addDownloadFormAtom, (prev) => ({ ...prev, ...update }));
  }
);
```

---

## API Call Patterns

### Pattern 1: Clipboard Monitoring

```typescript
// In useClipboard hook
useEffect(() => {
  if (!isMonitoring) return;

  const interval = setInterval(async () => {
    try {
      const content = await invoke<string>('get_clipboard_content');

      if (content !== lastContent) {
        setLastContent(content);

        if (urlDetector.isValidUrl(content)) {
          const result = await invoke<UrlDetectionResult>('detect_url', {
            url: content
          });

          if (result.isValid) {
            setDetectedUrl(content);
            setDetectionResult(result);

            // Auto-navigate if configured
            if (autoNavigate) {
              navigateTo('add-download', { url: content });
            }
          }
        }
      }
    } catch (error) {
      console.error('Clipboard monitoring error:', error);
    }
  }, 400);

  return () => clearInterval(interval);
}, [isMonitoring, lastContent]);
```

### Pattern 2: Form Submission

```typescript
// In AddDownloadForm component
const handleSubmit = async (formData: AddDownloadFormData) => {
  try {
    setLoading(true);

    // Validate URL
    const detectionResult = await invoke<UrlDetectionResult>('detect_url', {
      url: formData.url
    });

    if (!detectionResult.isValid) {
      setError('Invalid download URL');
      return;
    }

    // Create download task
    const taskId = await invoke<string>('create_download_with_path', {
      url: formData.url,
      filename: formData.filename,
      downloadPath: formData.downloadPath
    });

    // Start download
    await invoke('start_download', { taskId });

    // Navigate back to main
    navigateTo('main');

  } catch (error) {
    setError(error.message);
  } finally {
    setLoading(false);
  }
};
```

---

## Testing Requirements

### Unit Tests

**Hooks:**
- `useClipboard`: Test monitoring start/stop, URL detection, debouncing
- `useNavigation`: Test page transitions, back navigation, params passing
- `useDownloadPath`: Test path validation, directory picker

**Utils:**
- `urlDetector`: Test all validation functions with various inputs
- `pathValidator`: Test path normalization on different platforms

**Example Test:**
```typescript
import { renderHook, act } from '@testing-library/react';
import { useNavigation } from '@/hooks/useNavigation';

describe('useNavigation', () => {
  it('should navigate to add-download page', () => {
    const { result } = renderHook(() => useNavigation());

    act(() => {
      result.current.navigateTo('add-download', { url: 'test.zip' });
    });

    expect(result.current.currentPage).toBe('add-download');
    expect(result.current.navigationParams.url).toBe('test.zip');
  });

  it('should go back to previous page', () => {
    const { result } = renderHook(() => useNavigation());

    act(() => {
      result.current.navigateTo('add-download');
    });

    act(() => {
      result.current.goBack();
    });

    expect(result.current.currentPage).toBe('main');
  });
});
```

### Integration Tests

**Scenarios:**
1. Complete flow: Clipboard detection → Auto-navigate → Pre-fill → Submit → Back to main
2. Manual add: Click button → Fill form → Submit → Success
3. Error handling: Invalid URL → Show error → User corrects → Submit success

---

## Performance Requirements

| Operation | Target | Maximum |
|-----------|--------|---------|
| Clipboard polling | 400ms | 500ms |
| Page navigation | < 50ms | 100ms |
| Form validation | < 100ms | 200ms |
| Directory picker open | < 200ms | 500ms |

---

## Accessibility

**Requirements:**
- All buttons must have aria-labels
- Form inputs must have labels
- Error messages must be announced to screen readers
- Keyboard navigation must work for all interactions

**Example:**
```tsx
<Button
  onClick={handleAddDownload}
  aria-label="開啟新增下載頁面"
>
  新增下載
</Button>
```

---

## Summary

### New Hooks
1. ✅ `useClipboard` - 剪貼簿監控
2. ✅ `useNavigation` - 頁面導航
3. ✅ `useDownloadPath` - 路徑選擇

### New Components
1. ✅ `AddDownloadPage` - 新增下載頁面
2. ✅ `AddDownloadForm` - 新增下載表單

### Modified Components
1. ✅ `MainPage` - 移除內聯輸入，新增按鈕

### New Utilities
1. ✅ `urlDetector` - URL 驗證工具
2. ✅ `pathValidator` - 路徑驗證工具

### New Stores
1. ✅ `navigationStore` - 導航狀態
2. ✅ `clipboardStore` - 剪貼簿狀態
3. ✅ `addDownloadStore` - 表單狀態

所有 API 都與功能需求對應，並提供完整的類型定義和範例。
