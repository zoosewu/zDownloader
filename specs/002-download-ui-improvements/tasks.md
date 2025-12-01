# Implementation Tasks: Download UI Improvements

**Feature Branch**: `002-download-ui-improvements`
**Created**: 2025-12-01
**Status**: Ready for Implementation

## Overview

此文件包含 Download UI Improvements 功能的詳細實作任務清單。任務按照 User Story 優先順序組織，每個 Story 都可以獨立實作和測試。

**技術棧**:
- Frontend: TypeScript 5.8.3 + React 19.1.0 + Material UI 6.0.0 + Jotai 2.0.0
- Backend: Rust 1.75+ (Tauri 2.x)
- Testing: Cargo test (Rust)

**User Stories** (按優先順序):
1. **US1 (P1)**: Quick Add Download from Clipboard
2. **US2 (P1)**: Streamlined Add Download Flow
3. **US3 (P2)**: Customizable Download Path
4. **US4 (P2)**: Intelligent URL Detection

**MVP Scope**: User Story 1 + User Story 2 (核心自動化流程和基本 UI)

---

## Implementation Strategy

### Incremental Delivery Plan

**Phase 1 - Setup** (基礎設施準備):
- 專案結構驗證
- 無需安裝新依賴（使用狀態管理導航）

**Phase 2 - Foundational** (所有 Story 的共用基礎):
- URL 檢測模組系統（US1, US4 共用）
- 導航狀態管理（US2 共用）

**Phase 3 - US2 (P1)** 先實作（提供基本 UI 框架）:
- 建立新增下載頁面和表單
- 實作導航流程
- **獨立測試**: 手動新增下載流程

**Phase 4 - US1 (P1)** 接續實作（自動化核心功能）:
- 剪貼簿監控
- 自動導航和預填
- **獨立測試**: 剪貼簿自動偵測流程

**Phase 5 - US3 (P2)** 增強功能:
- 路徑自訂
- **獨立測試**: 自訂路徑下載

**Phase 6 - US4 (P2)** 模組化擴充:
- 完善 URL 檢測模組系統
- **獨立測試**: 不同類型 URL 檢測

**Phase 7 - Polish** 最終完善:
- 錯誤處理
- 效能優化

### Parallel Execution Opportunities

**可並行實作的任務組** (標記為 [P]):
- Phase 2: T004, T005 可並行（不同模組）
- Phase 3: T009, T010 可並行（不同檔案）
- Phase 4: T015, T016 可並行（前端 hooks）

---

## Task Checklist

### Phase 1: Setup

**Goal**: 驗證開發環境並確認專案結構

- [ ] T001 確認當前在 `002-download-ui-improvements` branch
- [ ] T002 驗證專案結構符合 plan.md 定義（src/, src-tauri/）
- [ ] T003 確認現有依賴已安裝（pnpm install, cargo check）

**Notes**:
- 此功能無需新增 npm 依賴（使用狀態管理導航）
- Rust 依賴已存在（tauri-plugin-clipboard-manager, tauri-plugin-dialog）

---

### Phase 2: Foundational (Blocking Prerequisites)

**Goal**: 實作所有 User Stories 共用的基礎功能

**Why Foundational**:
- URL 檢測模組是 US1 和 US4 的核心
- 導航系統是 US2 的基礎，US1 也需要

#### Backend: URL Detection Module System

- [ ] T004 [P] 建立 URL 檢測模組目錄結構 src-tauri/src/modules/url_detector/
- [ ] T005 [P] 定義 UrlDetector trait 在 src-tauri/src/modules/url_detector/mod.rs

**T005 Details**:
```rust
// src-tauri/src/modules/url_detector/mod.rs
pub trait UrlDetector: Send + Sync {
    fn name(&self) -> &str;
    fn can_handle(&self, url: &str) -> bool;
    fn validate(&self, url: &str) -> Result<UrlDetectionInfo, DetectionError>;
    fn supported_extensions(&self) -> Vec<&str> { vec![] }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct UrlDetectionInfo {
    pub url: String,
    pub is_valid: bool,
    pub detector_name: String,
    pub detection_mode: DetectionMode,
    pub suggested_filename: Option<String>,
    pub file_extension: Option<String>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum DetectionMode {
    FileExtension,
    ModuleCustom,
}

#[derive(Debug)]
pub enum DetectionError {
    InvalidUrl(String),
    NoMatchingDetector,
    InternalError(String),
}
```

- [ ] T006 實作 BuiltinFileDetector 在 src-tauri/src/modules/url_detector/builtin.rs

**T006 Details**:
```rust
// Support extensions: .zip, .pdf, .exe, .mp4, .mkv, .rar, .7z, .tar, .gz, .iso, .dmg, .apk
impl UrlDetector for BuiltinFileDetector {
    fn can_handle(&self, url: &str) -> bool {
        self.extensions.iter().any(|ext| url.ends_with(ext))
    }

    fn validate(&self, url: &str) -> Result<UrlDetectionInfo, DetectionError> {
        // Validate URL format
        // Extract filename and extension
        // Return UrlDetectionInfo
    }
}
```

- [ ] T007 實作 ModuleRegistry 在 src-tauri/src/modules/url_detector/mod.rs

**T007 Details**:
```rust
pub struct ModuleRegistry {
    detectors: Vec<Box<dyn UrlDetector>>,
}

impl ModuleRegistry {
    pub fn new() -> Self {
        let mut registry = Self { detectors: vec![] };
        registry.register(Box::new(BuiltinFileDetector::new()));
        registry
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

- [ ] T008 新增 ModuleRegistryState 到 Tauri State 並在 src-tauri/src/lib.rs 初始化

**T008 Details**:
```rust
// In lib.rs
pub struct ModuleRegistryState {
    pub registry: Arc<Mutex<ModuleRegistry>>,
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .manage(ModuleRegistryState {
            registry: Arc::new(Mutex::new(ModuleRegistry::new())),
        })
        // ... existing state
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
```

#### Frontend: Navigation Store

- [ ] T009 [P] 建立 navigationStore.ts 在 src/stores/navigationStore.ts

**T009 Details**:
```typescript
import { atom } from 'jotai';

export type PageType = 'main' | 'settings' | 'add-download';

export const currentPageAtom = atom<PageType>('main');
export const previousPageAtom = atom<PageType | null>(null);
export const navigationParamsAtom = atom<Record<string, any>>({});
```

- [ ] T010 [P] 建立 useNavigation hook 在 src/hooks/useNavigation.ts

**T010 Details**:
```typescript
export function useNavigation() {
  const [currentPage, setCurrentPage] = useAtom(currentPageAtom);
  const [previousPage, setPreviousPage] = useAtom(previousPageAtom);
  const [params, setParams] = useAtom(navigationParamsAtom);

  const navigateTo = useCallback((page: PageType, newParams?: Record<string, any>) => {
    setPreviousPage(currentPage);
    setCurrentPage(page);
    setParams(newParams || {});
  }, [currentPage]);

  const goBack = useCallback(() => {
    if (previousPage) {
      setCurrentPage(previousPage);
      setPreviousPage(null);
      setParams({});
    }
  }, [previousPage]);

  return {
    currentPage,
    previousPage,
    navigationParams: params,
    navigateTo,
    goBack,
    canGoBack: previousPage !== null,
  };
}
```

**Verification**:
- [ ] Cargo check passes
- [ ] TypeScript compilation passes (pnpm run build)

---

### Phase 3: User Story 2 - Streamlined Add Download Flow (P1)

**Goal**: 實作基本的新增下載 UI 和流程

**Independent Test Criteria**:
- ✅ 點擊「新增下載」按鈕能切換到新增下載頁面
- ✅ 填寫 URL 和檔名後提交能建立下載任務
- ✅ 成功後自動返回下載列表
- ✅ 點擊取消能返回下載列表且不建立任務

#### Frontend: Add Download UI

- [ ] T011 [US2] 建立 AddDownloadPage 在 src/pages/AddDownloadPage.tsx

**T011 Details**:
```typescript
export default function AddDownloadPage() {
  const { goBack, navigationParams } = useNavigation();
  const initialUrl = navigationParams?.url || '';

  return (
    <Container maxWidth="lg" sx={{ paddingY: 4 }}>
      <Box sx={{ display: 'flex', justifyContent: 'space-between', mb: 3 }}>
        <Typography variant="h4">新增下載</Typography>
        <Button onClick={goBack} startIcon={<ArrowBackIcon />}>
          返回
        </Button>
      </Box>

      <AddDownloadForm initialUrl={initialUrl} onCancel={goBack} />
    </Container>
  );
}
```

- [ ] T012 [US2] 建立 AddDownloadForm component 在 src/components/AddDownloadForm.tsx

**T012 Details**:
- Form fields: URL (required), Filename (optional), Download Path (default from settings)
- Validation: URL must be valid
- Submit handler: Call create_download_with_path command
- Cancel handler: Call onCancel prop

```typescript
interface AddDownloadFormProps {
  initialUrl?: string;
  onCancel: () => void;
}

export default function AddDownloadForm({ initialUrl, onCancel }: AddDownloadFormProps) {
  const [url, setUrl] = useState(initialUrl || '');
  const [filename, setFilename] = useState('');
  const settings = useAtomValue(settingsAtom);
  const [downloadPath, setDownloadPath] = useState(settings.defaultDownloadFolder);
  const { navigateTo } = useNavigation();

  const handleSubmit = async () => {
    const taskId = await invoke('create_download_with_path', {
      url,
      filename: filename || undefined,
      downloadPath,
    });
    await invoke('start_download', { taskId });
    navigateTo('main');
  };

  // Render form fields...
}
```

- [ ] T013 [US2] 修改 MainPage.tsx 移除內聯 UrlInput 並新增「新增下載」按鈕在 src/pages/MainPage.tsx

**T013 Details**:
```typescript
// Remove <UrlInput /> component from MainPage
// Add button in header
<Box sx={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center' }}>
  <Typography variant="h4">{t('app.title')}</Typography>
  <Button
    variant="contained"
    startIcon={<AddIcon />}
    onClick={() => navigateTo('add-download')}
  >
    新增下載
  </Button>
</Box>
```

- [ ] T014 [US2] 修改 App.tsx 擴展 currentPage 類型並新增 AddDownloadPage 渲染在 src/App.tsx

**T014 Details**:
```typescript
// Change currentPage type
const [currentPage, setCurrentPage] = useState<'main' | 'settings' | 'add-download'>('main');

// Add conditional rendering
<Box sx={{ flex: 1, overflow: 'auto' }}>
  {currentPage === 'main' && <MainPage />}
  {currentPage === 'settings' && <SettingsPage />}
  {currentPage === 'add-download' && <AddDownloadPage />}
</Box>
```

**Note**: 由於使用狀態管理導航，需要將 App.tsx 的導航邏輯整合 useNavigation hook

- [ ] T015 [US2] 整合 useNavigation hook 到 App.tsx 替代本地 state

**T015 Details**:
```typescript
// Replace useState with useNavigation
const { currentPage } = useNavigation();

// Remove setCurrentPage and Button onClick handlers in AppBar
// Navigation now handled by MainPage and AddDownloadPage
```

#### Backend: Create Download with Path Command

- [ ] T016 [P] [US2] 建立 create_download_with_path command 在 src-tauri/src/commands/download.rs

**T016 Details**:
```rust
#[tauri::command]
pub async fn create_download_with_path(
    url: String,
    filename: Option<String>,
    download_path: String,
    state: State<'_, AppState>,
    settings_state: State<'_, SettingsState>,
) -> Result<String, String> {
    // Validate URL
    // Validate download_path (exists, writable)
    // Extract filename from URL if not provided
    // Check file already exists -> auto-rename
    // Create DownloadTask
    // Add to queue
    // Return task_id
}
```

- [ ] T017 [US2] 在 src-tauri/src/lib.rs 註冊 create_download_with_path command

**T017 Details**:
```rust
.invoke_handler(tauri::generate_handler![
    // ... existing commands
    create_download_with_path,
])
```

**US2 Verification** (Independent Test):
- [ ] 手動測試：點擊「新增下載」→ 填寫 URL → 提交 → 驗證任務建立且返回列表
- [ ] 手動測試：點擊「新增下載」→ 點擊取消 → 驗證返回列表且無任務建立
- [ ] TypeScript build passes
- [ ] Cargo check passes

---

### Phase 4: User Story 1 - Quick Add Download from Clipboard (P1)

**Goal**: 實作剪貼簿自動監控和偵測功能

**Independent Test Criteria**:
- ✅ 複製包含附檔名的 URL (如 `https://example.com/file.zip`)
- ✅ 應用程式自動切換到新增下載頁面
- ✅ URL 欄位已預填
- ✅ 複製非下載連結時應用程式不反應

**Depends on**: US2 (需要 AddDownloadPage), Foundational (需要 URL detection)

#### Backend: Clipboard Commands

- [ ] T018 [P] [US1] 建立 get_clipboard_content command 在 src-tauri/src/commands/clipboard.rs

**T018 Details**:
```rust
use tauri_plugin_clipboard_manager::ClipboardExt;

#[tauri::command]
pub async fn get_clipboard_content(app: AppHandle) -> Result<String, String> {
    app.clipboard()
        .read_text()
        .map_err(|e| format!("Failed to read clipboard: {}", e))
}
```

- [ ] T019 [P] [US1] 建立 detect_url command 在 src-tauri/src/commands/clipboard.rs

**T019 Details**:
```rust
#[tauri::command]
pub async fn detect_url(
    url: String,
    state: State<'_, ModuleRegistryState>,
) -> Result<UrlDetectionInfo, String> {
    // Validate URL format
    if !is_valid_url(&url) {
        return Err(format!("Invalid URL format: {}", url));
    }

    let registry = state.registry.lock().await;
    registry.detect_url(&url)
        .map_err(|e| format!("Detection failed: {:?}", e))
}

fn is_valid_url(url: &str) -> bool {
    // Basic URL validation (http/https schema, valid format)
    url.starts_with("http://") || url.starts_with("https://")
}
```

- [ ] T020 [US1] 在 src-tauri/src/lib.rs 註冊 clipboard commands

**T020 Details**:
```rust
.invoke_handler(tauri::generate_handler![
    // ... existing commands
    get_clipboard_content,
    detect_url,
])
```

#### Frontend: Clipboard Monitoring

- [ ] T021 [P] [US1] 建立 useClipboard hook 在 src/hooks/useClipboard.ts

**T021 Details**:
```typescript
export function useClipboard() {
  const [detectedUrl, setDetectedUrl] = useState<string | null>(null);
  const lastContentRef = useRef<string>('');
  const { navigateTo } = useNavigation();
  const settings = useAtomValue(settingsAtom);

  useEffect(() => {
    if (!settings.clipboardMonitoringEnabled) return;

    const interval = setInterval(async () => {
      try {
        const content = await invoke<string>('get_clipboard_content');

        if (content !== lastContentRef.current) {
          lastContentRef.current = content;

          // Quick client-side validation
          if (!content.startsWith('http')) return;

          const result = await invoke<UrlDetectionResult>('detect_url', { url: content });

          if (result.isValid) {
            setDetectedUrl(content);
            navigateTo('add-download', { url: content });
          }
        }
      } catch (e) {
        // Silent fail or log
      }
    }, 400); // 400ms polling interval

    return () => clearInterval(interval);
  }, [settings.clipboardMonitoringEnabled]);

  return { detectedUrl };
}
```

- [ ] T022 [US1] 在 App.tsx 中啟用 useClipboard hook

**T022 Details**:
```typescript
function App() {
  useClipboard(); // Start clipboard monitoring
  const { currentPage } = useNavigation();
  // ... rest of App component
}
```

- [ ] T023 [US1] 修改 AddDownloadPage 使用 navigationParams 預填 URL

**T023 Details**:
- 已在 T011 中實作 `initialUrl = navigationParams?.url || ''`
- 驗證此邏輯正確傳遞到 AddDownloadForm

**US1 Verification** (Independent Test):
- [ ] 手動測試：複製 `https://example.com/file.zip` → 驗證自動跳轉並預填
- [ ] 手動測試：複製 `https://example.com/page.html` → 驗證不跳轉
- [ ] 效能測試：驗證剪貼簿偵測延遲 < 500ms (SC-003)

---

### Phase 5: User Story 3 - Customizable Download Path (P2)

**Goal**: 新增路徑自訂功能

**Independent Test Criteria**:
- ✅ 開啟新增下載表單時，路徑欄位顯示預設資料夾
- ✅ 點擊瀏覽按鈕能開啟資料夾選擇器
- ✅ 選擇自訂路徑後提交，檔案下載到自訂路徑

**Depends on**: US2 (需要 AddDownloadForm)

#### Frontend: Path Selection

- [ ] T024 [P] [US3] 建立 useDownloadPath hook 在 src/hooks/useDownloadPath.ts

**T024 Details**:
```typescript
import { open } from '@tauri-apps/plugin-dialog';

export function useDownloadPath(initialPath?: string) {
  const settings = useAtomValue(settingsAtom);
  const [path, setPath] = useState(initialPath || settings.defaultDownloadFolder);
  const [isValid, setIsValid] = useState(true);
  const [errorMessage, setErrorMessage] = useState<string | null>(null);

  const openDirectoryPicker = async () => {
    const selected = await open({
      directory: true,
      multiple: false,
      defaultPath: path,
    });

    if (selected && typeof selected === 'string') {
      setPath(selected);
    }
  };

  const validatePath = async () => {
    // TODO: Call backend to validate path (writable, exists)
    return true;
  };

  return {
    path,
    setPath,
    isValid,
    errorMessage,
    openDirectoryPicker,
    validatePath,
  };
}
```

- [ ] T025 [US3] 修改 AddDownloadForm 整合 useDownloadPath hook 在 src/components/AddDownloadForm.tsx

**T025 Details**:
```typescript
export default function AddDownloadForm({ initialUrl, onCancel }: AddDownloadFormProps) {
  const settings = useAtomValue(settingsAtom);
  const { path, openDirectoryPicker } = useDownloadPath(settings.defaultDownloadFolder);

  const handleSubmit = async () => {
    const taskId = await invoke('create_download_with_path', {
      url,
      filename: filename || undefined,
      downloadPath: path, // Use custom path
    });
    await invoke('start_download', { taskId });
    navigateTo('main');
  };

  return (
    <Box component="form">
      {/* URL and Filename fields */}
      <TextField
        label="下載路徑"
        value={path}
        fullWidth
        margin="normal"
        InputProps={{
          readOnly: true,
          endAdornment: (
            <InputAdornment position="end">
              <Button onClick={openDirectoryPicker}>瀏覽</Button>
            </InputAdornment>
          ),
        }}
      />
      {/* Submit and Cancel buttons */}
    </Box>
  );
}
```

**US3 Verification** (Independent Test):
- [ ] 手動測試：開啟新增下載 → 驗證路徑欄位顯示預設資料夾
- [ ] 手動測試：點擊瀏覽 → 選擇路徑 → 提交 → 驗證檔案下載到自訂路徑

---

### Phase 6: User Story 4 - Intelligent URL Detection (P2)

**Goal**: 完善 URL 檢測模組系統（已在 Foundational 完成基礎）

**Independent Test Criteria**:
- ✅ 測試 `.zip`, `.pdf`, `.exe` 等常見附檔名能被正確檢測
- ✅ 測試無附檔名的 URL 被正確拒絕
- ✅ 檢測準確率 > 95% (SC-004)

**Depends on**: Foundational (URL detection module)

#### Backend: Enhance URL Detection

- [ ] T026 [P] [US4] 新增 URL 格式驗證工具函數在 src-tauri/src/utils/url_validator.rs

**T026 Details**:
```rust
pub fn validate_url_format(url: &str) -> Result<(), String> {
    // Check schema (http/https only)
    if !url.starts_with("http://") && !url.starts_with("https://") {
        return Err("URL must use http or https schema".to_string());
    }

    // Check for javascript: or data: (security)
    if url.starts_with("javascript:") || url.starts_with("data:") {
        return Err("Unsafe URL schema".to_string());
    }

    // Additional validation...
    Ok(())
}
```

- [ ] T027 [US4] 強化 BuiltinFileDetector 的檔名提取邏輯在 src-tauri/src/modules/url_detector/builtin.rs

**T027 Details**:
```rust
impl BuiltinFileDetector {
    fn extract_filename(&self, url: &str) -> Option<String> {
        url.split('/')
            .last()
            .filter(|s| !s.is_empty())
            .map(String::from)
    }

    fn get_file_extension(&self, filename: &str) -> Option<String> {
        filename.rfind('.')
            .map(|idx| filename[idx..].to_string())
    }
}
```

#### Frontend: URL Validation Utilities

- [ ] T028 [P] [US4] 建立 urlDetector 工具函數在 src/utils/urlDetector.ts

**T028 Details**:
```typescript
export const urlDetector = {
  isValidUrl(url: string): boolean {
    try {
      new URL(url);
      return url.startsWith('http://') || url.startsWith('https://');
    } catch {
      return false;
    }
  },

  extractFilename(url: string): string | null {
    const parts = url.split('/');
    const last = parts[parts.length - 1];
    return last || null;
  },

  getFileExtension(url: string): string | null {
    const filename = this.extractFilename(url);
    if (!filename) return null;

    const dotIndex = filename.lastIndexOf('.');
    if (dotIndex === -1) return null;

    return filename.substring(dotIndex);
  },

  hasCommonFileExtension(url: string): boolean {
    const ext = this.getFileExtension(url);
    if (!ext) return false;

    const commonExts = ['.zip', '.pdf', '.exe', '.mp4', '.mkv', '.rar', '.7z', '.tar', '.gz', '.iso', '.dmg', '.apk'];
    return commonExts.includes(ext.toLowerCase());
  },
};
```

- [ ] T029 [US4] 在 useClipboard hook 中使用 urlDetector 進行快速前端驗證

**T029 Details**:
```typescript
// In useClipboard.ts
if (content !== lastContentRef.current) {
  lastContentRef.current = content;

  // Quick client-side validation BEFORE calling backend
  if (!urlDetector.isValidUrl(content)) return;
  if (!urlDetector.hasCommonFileExtension(content)) {
    // Skip backend call if no common extension
    // (Module custom detection will be added later)
    return;
  }

  const result = await invoke<UrlDetectionResult>('detect_url', { url: content });
  // ...
}
```

**US4 Verification** (Independent Test):
- [ ] 單元測試：測試 10 種常見附檔名 URL 的檢測
- [ ] 單元測試：測試無附檔名 URL 被正確拒絕
- [ ] 計算檢測準確率 (目標 > 95%)

---

### Phase 7: Polish & Cross-Cutting Concerns

**Goal**: 完善錯誤處理、效能優化、使用者體驗

#### Error Handling

- [ ] T030 [P] 在 AddDownloadForm 新增錯誤處理和使用者回饋在 src/components/AddDownloadForm.tsx

**T030 Details**:
```typescript
const [error, setError] = useState<string | null>(null);
const [loading, setLoading] = useState(false);

const handleSubmit = async () => {
  setError(null);
  setLoading(true);

  try {
    // Validate URL
    const detectionResult = await invoke<UrlDetectionResult>('detect_url', { url });
    if (!detectionResult.isValid) {
      setError('Invalid download URL');
      return;
    }

    const taskId = await invoke<string>('create_download_with_path', {
      url,
      filename: filename || undefined,
      downloadPath: path,
    });

    await invoke('start_download', { taskId });
    navigateTo('main');
  } catch (e: any) {
    setError(e.message || 'Failed to create download');
  } finally {
    setLoading(false);
  }
};

// Show error message to user
{error && <Alert severity="error">{error}</Alert>}
```

- [ ] T031 [P] 在 create_download_with_path 新增路徑驗證在 src-tauri/src/commands/download.rs

**T031 Details**:
```rust
// Validate download_path
let path = PathBuf::from(&download_path);
if !path.exists() {
    return Err(format!("Download path does not exist: {}", download_path));
}
if !path.is_dir() {
    return Err(format!("Download path is not a directory: {}", download_path));
}

// Check write permission (attempt to create temp file)
let test_file = path.join(".zoo_downloader_test");
if std::fs::write(&test_file, b"test").is_err() {
    return Err(format!("Download path is not writable: {}", download_path));
}
std::fs::remove_file(&test_file).ok();
```

#### Performance Optimization

- [ ] T032 在 useClipboard hook 新增 debounce 避免重複處理在 src/hooks/useClipboard.ts

**T032 Details**:
```typescript
const processingRef = useRef(false);

// In interval callback
if (processingRef.current) return; // Skip if still processing

if (content !== lastContentRef.current) {
  processingRef.current = true;
  lastContentRef.current = content;

  try {
    // ... detection logic
  } finally {
    processingRef.current = false;
  }
}
```

- [ ] T033 在 App.tsx 新增 focus/blur 監聽，僅在視窗 focused 時啟用剪貼簿監控

**T033 Details**:
```typescript
const [isWindowFocused, setIsWindowFocused] = useState(true);

useEffect(() => {
  const handleFocus = () => setIsWindowFocused(true);
  const handleBlur = () => setIsWindowFocused(false);

  window.addEventListener('focus', handleFocus);
  window.addEventListener('blur', handleBlur);

  return () => {
    window.removeEventListener('focus', handleFocus);
    window.removeEventListener('blur', handleBlur);
  };
}, []);

// Pass isWindowFocused to useClipboard
// Modify useClipboard to accept enabled parameter
```

#### UX Improvements

- [ ] T034 [P] 新增載入狀態到 AddDownloadForm 按鈕在 src/components/AddDownloadForm.tsx

**T034 Details**:
```typescript
<Button
  variant="contained"
  onClick={handleSubmit}
  disabled={loading || !url}
  startIcon={loading && <CircularProgress size={20} />}
>
  {loading ? '建立中...' : '新增下載'}
</Button>
```

- [ ] T035 [P] 新增 Toast/Snackbar 通知成功建立下載在 src/pages/AddDownloadPage.tsx

**T035 Details**:
```typescript
const [showSuccess, setShowSuccess] = useState(false);

const handleSuccess = () => {
  setShowSuccess(true);
  setTimeout(() => navigateTo('main'), 1000); // Delay navigation for toast
};

<Snackbar
  open={showSuccess}
  message="下載任務已建立"
  autoHideDuration={3000}
/>
```

#### Code Quality

- [ ] T036 執行 TypeScript 型別檢查並修正所有錯誤 (pnpm run build)
- [ ] T037 執行 Rust clippy 並修正所有警告 (cd src-tauri && cargo clippy)
- [ ] T038 執行 Rust 測試確保無迴歸 (cd src-tauri && cargo test)

---

## Task Dependencies

### Dependency Graph (User Story Completion Order)

```
Phase 1 (Setup)
    ↓
Phase 2 (Foundational)
    ├─→ URL Detection Module (T004-T008)
    └─→ Navigation System (T009-T010)
         ↓
    ┌────┴────┐
    ↓         ↓
Phase 3      Phase 4
(US2: P1)    (US1: P1)
T011-T017    T018-T023
    ↓            ↓
    └────┬───────┘
         ↓
    ┌────┴────┐
    ↓         ↓
Phase 5      Phase 6
(US3: P2)    (US4: P2)
T024-T025    T026-T029
    ↓         ↓
    └────┬────┘
         ↓
    Phase 7 (Polish)
    T030-T038
```

### Critical Path

**最關鍵路徑** (必須依序完成):
1. Setup (T001-T003)
2. Foundational (T004-T010)
3. US2 基本 UI (T011-T017)
4. US1 剪貼簿監控 (T018-T023)
5. Polish (T030-T038)

**MVP 範圍**: T001-T023 + T030-T031 (錯誤處理)

### Parallel Execution Examples

**可並行執行的任務組**:

**Group 1** (Foundational Backend + Frontend):
- T004-T008 (Backend URL Detection Module)
- T009-T010 (Frontend Navigation)

**Group 2** (US2 Frontend + Backend):
- T011-T015 (Frontend UI components)
- T016-T017 (Backend command)

**Group 3** (US1 Frontend + Backend):
- T018-T020 (Backend clipboard commands)
- T021-T023 (Frontend clipboard hook)

**Group 4** (Polish):
- T030 (Frontend error handling)
- T031 (Backend validation)
- T034, T035 (UX improvements)

---

## Testing Strategy

### Manual Testing Scenarios

**US1 Test**:
1. 複製 `https://example.com/file.zip`
2. 驗證自動跳轉到新增下載頁面
3. 驗證 URL 欄位已預填
4. 點擊提交
5. 驗證下載任務建立並返回列表

**US2 Test**:
1. 點擊「新增下載」按鈕
2. 手動輸入 URL: `https://example.com/test.pdf`
3. 輸入檔名: `my-file.pdf`
4. 點擊提交
5. 驗證下載任務建立並返回列表

**US3 Test**:
1. 開啟新增下載頁面
2. 點擊路徑欄位的「瀏覽」按鈕
3. 選擇自訂資料夾
4. 提交下載
5. 驗證檔案下載到自訂路徑

**US4 Test**:
1. 測試 10 種不同附檔名的 URL
2. 驗證所有有效附檔名被正確識別
3. 測試無附檔名的 URL
4. 驗證被正確拒絕

### Performance Testing

**SC-003 Test** (剪貼簿偵測 < 500ms):
```bash
# Measure clipboard detection latency
1. Start application
2. Copy URL
3. Measure time until page navigation
4. Target: < 500ms
```

**SC-005 Test** (完整流程 < 5 秒):
```bash
# Measure complete add download flow
1. Click "新增下載"
2. Fill URL
3. Submit
4. Return to list
5. Target: < 5 seconds
```

---

## Progress Tracking

### Task Summary

**Total Tasks**: 38

**By Phase**:
- Phase 1 (Setup): 3 tasks
- Phase 2 (Foundational): 7 tasks
- Phase 3 (US2): 7 tasks
- Phase 4 (US1): 6 tasks
- Phase 5 (US3): 2 tasks
- Phase 6 (US4): 4 tasks
- Phase 7 (Polish): 9 tasks

**By User Story**:
- Setup/Foundational: 10 tasks
- US1 (P1): 6 tasks
- US2 (P1): 7 tasks
- US3 (P2): 2 tasks
- US4 (P2): 4 tasks
- Polish: 9 tasks

**Parallelizable Tasks**: 14 tasks marked with [P]

### MVP Scope

**建議 MVP** (最小可用產品):
- Phase 1: Setup (T001-T003)
- Phase 2: Foundational (T004-T010)
- Phase 3: US2 - Streamlined Flow (T011-T017)
- Phase 4: US1 - Clipboard Detection (T018-T023)
- Phase 7: Basic Error Handling (T030-T031)

**MVP 任務數**: 24 / 38 tasks (63%)

**預估時間**: 12-15 小時

**剩餘增強功能** (Phase 5, 6, 7 其餘任務): 5-7 小時

---

## Validation Checklist

### Format Validation

✅ **All tasks follow checklist format**:
- [x] All tasks start with `- [ ]`
- [x] All tasks have Task ID (T001-T038)
- [x] Parallelizable tasks marked with [P]
- [x] User Story tasks marked with [US1], [US2], [US3], [US4]
- [x] All tasks have clear descriptions
- [x] All tasks reference specific file paths

### Completeness Validation

✅ **Coverage**:
- [x] All User Stories have corresponding tasks
- [x] All entities from data-model.md mapped to tasks
- [x] All commands from contracts/ mapped to tasks
- [x] All hooks/components from frontend-api.md mapped to tasks
- [x] Error handling tasks included
- [x] Performance optimization tasks included

✅ **Independent Testing**:
- [x] Each User Story has clear test criteria
- [x] Each User Story can be tested independently
- [x] Performance goals (SC-003, SC-005) have test procedures

✅ **Dependencies**:
- [x] Dependency graph shows User Story completion order
- [x] Foundational tasks identified as blocking
- [x] Parallel execution opportunities documented

---

## Notes

- **無需安裝新依賴**: 使用狀態管理導航（擴展現有 App.tsx）
- **測試策略**: 主要使用手動測試，Rust 後端使用 cargo test
- **效能基準**: 剪貼簿偵測 < 500ms, 完整流程 < 5 秒
- **優先順序**: P1 故事 (US1, US2) 是 MVP，P2 故事 (US3, US4) 可後續增強

**Ready for implementation!** 🚀
