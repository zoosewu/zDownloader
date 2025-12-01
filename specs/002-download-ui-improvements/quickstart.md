# Quick Start Guide: Download UI Improvements

**Feature**: Download UI Improvements
**Branch**: `002-download-ui-improvements`
**Date**: 2025-12-01

## Overview

本指南協助開發者快速上手 Download UI Improvements 功能的開發。涵蓋環境設定、關鍵概念、開發工作流程和測試方法。

---

## Prerequisites

確認開發環境已具備以下條件：

- [x] Node.js 18+ 和 pnpm installed
- [x] Rust 1.75+ installed
- [x] Tauri CLI installed
- [x] 已 checkout 到 `002-download-ui-improvements` branch

**驗證環境：**
```bash
# Check versions
node --version  # v18+
pnpm --version
rustc --version  # 1.75+
cargo --version

# Verify current branch
git branch --show-current  # Should be: 002-download-ui-improvements
```

---

## Quick Setup (< 5 minutes)

### 1. Install Dependencies

```bash
# Frontend dependencies (already installed, but verify)
cd /nodejs/zoo-downloader
pnpm install

# Backend dependencies
cd src-tauri
cargo check
```

### 2. Run Development Server

```bash
# From project root
pnpm run tauri:dev

# This starts:
# - Vite dev server (frontend)
# - Tauri dev window (backend)
```

### 3. Verify Setup

開啟應用程式後，應該看到：
- [x] 主畫面顯示下載列表
- [x] 右上角導航列有「下載」和「設定」按鈕

---

## Key Concepts

### 1. State Management Navigation

**不使用 React Router**，而是使用狀態管理來控制頁面切換。

**Current Implementation** (App.tsx):
```typescript
const [currentPage, setCurrentPage] = useState<'main' | 'settings'>('main');
```

**New Implementation** (需擴展):
```typescript
const [currentPage, setCurrentPage] = useState<'main' | 'settings' | 'add-download'>('main');
```

**Why?**
- 簡單直接
- 零額外依賴
- 符合桌面應用需求（不需要 URL 路由）

### 2. Clipboard Monitoring

**輪詢機制** (400ms interval):

```typescript
useEffect(() => {
  const interval = setInterval(async () => {
    const content = await invoke('get_clipboard_content');
    // Process content...
  }, 400);

  return () => clearInterval(interval);
}, []);
```

**Performance Goal**: 偵測延遲 < 500ms (SC-003)

### 3. Modular URL Detection

**Rust Trait System**:

```rust
pub trait UrlDetector {
    fn name(&self) -> &str;
    fn can_handle(&self, url: &str) -> bool;
    fn validate(&self, url: &str) -> Result<UrlDetectionInfo, Error>;
}
```

**Builtin Detector**:
- 檢測常見附檔名 (`.zip`, `.pdf`, `.exe`, etc.)
- 第一版只需實作這個

**Future Extensibility**:
- 網站模組可實作相同 trait
- 無需修改核心程式碼

---

## Development Workflow

### Phase 1: Setup Navigation (2-3 hours)

**Goal**: 實作頁面導航機制

**Steps**:

1. **Create navigation store**
   ```bash
   touch src/stores/navigationStore.ts
   ```

   ```typescript
   // navigationStore.ts
   export const currentPageAtom = atom<PageType>('main');
   export const previousPageAtom = atom<PageType | null>(null);
   ```

2. **Create useNavigation hook**
   ```bash
   touch src/hooks/useNavigation.ts
   ```

3. **Modify App.tsx**
   - 擴展 `currentPage` type: `'main' | 'settings' | 'add-download'`
   - 新增條件渲染 `AddDownloadPage`

4. **Modify MainPage.tsx**
   - 移除內聯的 `UrlInput`
   - 新增右上角「新增下載」按鈕
   - 按鈕點擊 → `setCurrentPage('add-download')`

**驗證**:
- [x] 點擊「新增下載」按鈕能切換到空白頁面
- [x] 按返回能回到主畫面

---

### Phase 2: Create Add Download UI (3-4 hours)

**Goal**: 建立新增下載的頁面和表單

**Steps**:

1. **Create AddDownloadPage**
   ```bash
   touch src/pages/AddDownloadPage.tsx
   ```

   ```typescript
   export default function AddDownloadPage() {
     const { goBack } = useNavigation();

     return (
       <Container>
         <Box>
           <Typography variant="h4">新增下載</Typography>
           <Button onClick={goBack}>返回</Button>
         </Box>

         <AddDownloadForm onCancel={goBack} />
       </Container>
     );
   }
   ```

2. **Create AddDownloadForm**
   ```bash
   touch src/components/AddDownloadForm.tsx
   ```

   **Form fields**:
   - URL input (TextField)
   - Filename input (TextField, optional)
   - Download path (TextField + Browse button)

3. **Create useDownloadPath hook**
   ```bash
   touch src/hooks/useDownloadPath.ts
   ```

   ```typescript
   export function useDownloadPath() {
     const settings = useAtomValue(settingsAtom);
     const [path, setPath] = useState(settings.defaultDownloadFolder);

     const openPicker = async () => {
       const selected = await open({
         directory: true,
         multiple: false,
       });
       if (selected) setPath(selected);
     };

     return { path, setPath, openPicker };
   }
   ```

**驗證**:
- [x] 表單顯示三個輸入欄位
- [x] 路徑欄位預設顯示設定中的預設資料夾
- [x] 瀏覽按鈕能開啟資料夾選擇器

---

### Phase 3: Implement Backend Commands (4-5 hours)

**Goal**: 實作 URL 檢測和路徑支援的後端 commands

**Steps**:

1. **Create URL detector module**
   ```bash
   mkdir -p src-tauri/src/modules/url_detector
   touch src-tauri/src/modules/url_detector/mod.rs
   touch src-tauri/src/modules/url_detector/builtin.rs
   ```

   **mod.rs** (Trait definition):
   ```rust
   pub trait UrlDetector: Send + Sync {
       fn name(&self) -> &str;
       fn can_handle(&self, url: &str) -> bool;
       fn validate(&self, url: &str) -> Result<UrlDetectionInfo, DetectionError>;
   }
   ```

   **builtin.rs** (Implementation):
   ```rust
   pub struct BuiltinFileDetector {
       extensions: Vec<String>,
   }

   impl BuiltinFileDetector {
       pub fn new() -> Self {
           Self {
               extensions: vec![
                   ".zip", ".pdf", ".exe", ".mp4", ".mkv",
                   ".rar", ".7z", ".tar", ".gz", ".iso",
                   ".dmg", ".apk"
               ].into_iter().map(String::from).collect(),
           }
       }
   }

   impl UrlDetector for BuiltinFileDetector {
       fn name(&self) -> &str { "BuiltinFileDetector" }

       fn can_handle(&self, url: &str) -> bool {
           self.extensions.iter().any(|ext| url.ends_with(ext))
       }

       fn validate(&self, url: &str) -> Result<UrlDetectionInfo, DetectionError> {
           // Implementation...
       }
   }
   ```

2. **Create detect_url command**
   ```bash
   touch src-tauri/src/commands/clipboard.rs
   ```

   ```rust
   #[tauri::command]
   pub async fn detect_url(
       url: String,
       state: State<'_, ModuleRegistryState>,
   ) -> Result<UrlDetectionResult, String> {
       let registry = state.registry.lock().await;
       registry.detect_url(&url)
           .map_err(|e| e.to_string())
   }
   ```

3. **Create get_clipboard_content command**
   ```rust
   #[tauri::command]
   pub async fn get_clipboard_content() -> Result<String, String> {
       // Use tauri-plugin-clipboard-manager
       clipboard_manager::read()
           .map_err(|e| format!("Failed to read clipboard: {}", e))
   }
   ```

4. **Create create_download_with_path command**
   - Extend existing `create_download` in `src-tauri/src/commands/download.rs`
   - Add `download_path` parameter

**驗證**:
- [x] `cargo check` passes
- [x] `cargo test` passes (unit tests)

---

### Phase 4: Clipboard Monitoring (3-4 hours)

**Goal**: 實作自動剪貼簿監控和 URL 檢測

**Steps**:

1. **Create useClipboard hook**
   ```bash
   touch src/hooks/useClipboard.ts
   ```

   ```typescript
   export function useClipboard() {
     const [detectedUrl, setDetectedUrl] = useState<string | null>(null);
     const lastContentRef = useRef<string>('');
     const { navigateTo } = useNavigation();

     useEffect(() => {
       const interval = setInterval(async () => {
         try {
           const content = await invoke<string>('get_clipboard_content');

           if (content !== lastContentRef.current) {
             lastContentRef.current = content;

             const result = await invoke<UrlDetectionResult>('detect_url', { url: content });

             if (result.isValid) {
               setDetectedUrl(content);
               navigateTo('add-download', { url: content });
             }
           }
         } catch (e) {
           // Silent fail or log
         }
       }, 400);

       return () => clearInterval(interval);
     }, []);

     return { detectedUrl };
   }
   ```

2. **Integrate with AddDownloadPage**
   - Get `detectedUrl` from navigation params
   - Pre-fill URL field

3. **Add clipboard monitoring toggle in settings**
   - Read from `settings.clipboardMonitoringEnabled`
   - Only start monitoring if enabled

**驗證**:
- [x] 複製下載連結（如 `https://example.com/file.zip`）
- [x] 應用程式自動切換到新增下載頁面
- [x] URL 欄位已預填連結

---

### Phase 5: Integration & Testing (2-3 hours)

**Goal**: 整合所有功能並測試完整流程

**Steps**:

1. **End-to-end manual testing**
   - Test scenario 1: 手動新增下載
   - Test scenario 2: 剪貼簿自動偵測
   - Test scenario 3: 自訂路徑
   - Test scenario 4: 錯誤處理

2. **Unit tests**
   ```bash
   # Frontend (Vitest)
   pnpm run test

   # Backend (Cargo)
   cd src-tauri
   cargo test
   ```

3. **Performance verification**
   - 測量剪貼簿偵測延遲（應 < 500ms）
   - 測量完整新增流程時間（應 < 5 秒）

**驗證 Checklist**:
- [x] All functional requirements (FR-001 to FR-015) work
- [x] All success criteria (SC-001 to SC-005) met
- [x] All tests pass
- [x] No console errors
- [x] Performance goals met

---

## File Structure Reference

```
src/                                    # Frontend
├── components/
│   ├── AddDownloadForm.tsx            # NEW
│   └── DownloadList.tsx               # EXISTING
├── pages/
│   ├── MainPage.tsx                   # MODIFIED
│   ├── AddDownloadPage.tsx            # NEW
│   └── SettingsPage.tsx               # EXISTING
├── hooks/
│   ├── useClipboard.ts                # NEW
│   ├── useNavigation.ts               # NEW
│   └── useDownloadPath.ts             # NEW
├── stores/
│   ├── navigationStore.ts             # NEW
│   ├── clipboardStore.ts              # NEW (optional)
│   └── settingsStore.ts               # EXISTING
├── utils/
│   ├── urlDetector.ts                 # NEW
│   └── pathValidator.ts               # NEW (optional)
└── App.tsx                            # MODIFIED

src-tauri/src/                         # Backend
├── commands/
│   ├── clipboard.rs                   # NEW
│   └── download.rs                    # MODIFIED
├── modules/
│   └── url_detector/                  # NEW
│       ├── mod.rs                     # Trait definition
│       └── builtin.rs                 # Builtin implementation
└── lib.rs                             # MODIFIED (register commands)
```

---

## Common Tasks

### Add a new Tauri command

1. Define command in `src-tauri/src/commands/`:
   ```rust
   #[tauri::command]
   pub async fn my_command() -> Result<String, String> {
       Ok("Success".to_string())
   }
   ```

2. Register in `src-tauri/src/lib.rs`:
   ```rust
   .invoke_handler(tauri::generate_handler![
       // ... existing commands
       my_command
   ])
   ```

3. Use in frontend:
   ```typescript
   const result = await invoke<string>('my_command');
   ```

### Add a new page

1. Create page component in `src/pages/`:
   ```tsx
   export default function MyPage() {
       return <Container>...</Container>;
   }
   ```

2. Update `PageType` in stores/navigationStore.ts:
   ```typescript
   type PageType = 'main' | 'settings' | 'add-download' | 'my-page';
   ```

3. Add rendering in `App.tsx`:
   ```tsx
   {currentPage === 'my-page' && <MyPage />}
   ```

### Add a new Jotai atom

1. Create or update store file in `src/stores/`:
   ```typescript
   export const myAtom = atom<MyType>(initialValue);
   ```

2. Use in component:
   ```typescript
   const [myValue, setMyValue] = useAtom(myAtom);
   // or
   const myValue = useAtomValue(myAtom);
   const setMyValue = useSetAtom(myAtom);
   ```

---

## Debugging Tips

### Frontend Debugging

**Enable React DevTools:**
- Install React DevTools browser extension
- Inspect component state and props

**Enable Vite Debug Logs:**
```bash
DEBUG=vite:* pnpm run dev
```

**Check Jotai State:**
```typescript
import { useAtomValue } from 'jotai';
import { useHydrateAtoms } from 'jotai/utils';

// In component
const allState = useAtomValue(/* your atom */);
console.log('Current state:', allState);
```

### Backend Debugging

**Enable Tauri Debugging:**
```rust
// In src-tauri/src/lib.rs
#[cfg(debug_assertions)]
{
    println!("Debug mode enabled");
    dbg!(some_variable);
}
```

**Run with logs:**
```bash
RUST_LOG=debug cargo tauri dev
```

**Unit test specific function:**
```bash
cd src-tauri
cargo test test_name -- --nocapture
```

### Common Issues

**Issue: Clipboard permission denied**
- **Solution**: Check OS permission settings, ensure app has clipboard access

**Issue: Path picker not opening**
- **Solution**: Verify `@tauri-apps/plugin-dialog` is installed and configured

**Issue: URL detection not working**
- **Solution**: Check Rust module registry initialization, verify detector is registered

**Issue: Navigation not working**
- **Solution**: Check `currentPageAtom` value, verify page rendering logic in App.tsx

---

## Performance Profiling

### Frontend Profiling

**Use React DevTools Profiler:**
1. Open React DevTools
2. Go to Profiler tab
3. Click record
4. Perform actions
5. Stop and analyze flamegraph

**Measure render times:**
```typescript
useEffect(() => {
  const start = performance.now();
  // Your code
  const end = performance.now();
  console.log(`Render took ${end - start}ms`);
}, []);
```

### Backend Profiling

**Use Criterion for benchmarks:**
```rust
#[cfg(test)]
mod benches {
    use criterion::{black_box, criterion_group, criterion_main, Criterion};

    fn bench_detect_url(c: &mut Criterion) {
        c.bench_function("detect_url", |b| {
            b.iter(|| {
                // Your code
            });
        });
    }

    criterion_group!(benches, bench_detect_url);
    criterion_main!(benches);
}
```

**Run benchmarks:**
```bash
cd src-tauri
cargo bench
```

---

## Testing

### Frontend Tests (Vitest)

**Run all tests:**
```bash
pnpm run test
```

**Run specific test file:**
```bash
pnpm run test useClipboard.test.ts
```

**Run with coverage:**
```bash
pnpm run test:coverage
```

**Example test:**
```typescript
import { renderHook, act } from '@testing-library/react';
import { useNavigation } from '@/hooks/useNavigation';

describe('useNavigation', () => {
  it('should navigate to add-download page', () => {
    const { result } = renderHook(() => useNavigation());

    act(() => {
      result.current.navigateTo('add-download');
    });

    expect(result.current.currentPage).toBe('add-download');
  });
});
```

### Backend Tests (Cargo)

**Run all tests:**
```bash
cd src-tauri
cargo test
```

**Run specific test:**
```bash
cargo test test_builtin_detector
```

**Run with output:**
```bash
cargo test -- --nocapture
```

**Example test:**
```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_builtin_detector_zip() {
        let detector = BuiltinFileDetector::new();
        assert!(detector.can_handle("https://example.com/file.zip"));
    }

    #[test]
    fn test_builtin_detector_invalid() {
        let detector = BuiltinFileDetector::new();
        assert!(!detector.can_handle("https://example.com/page"));
    }
}
```

---

## Next Steps After Phase 1

一旦基本功能完成，可以考慮以下增強功能：

1. **Phase 2 Enhancements**:
   - Add more URL detector modules (YouTube, Google Drive, etc.)
   - Implement clipboard hint UI (optional toast notification)
   - Add animation to page transitions
   - Implement keyboard shortcuts (Ctrl+N for new download)

2. **Performance Optimizations**:
   - Optimize clipboard polling frequency based on activity
   - Cache URL detection results
   - Lazy load AddDownloadPage component

3. **User Experience Improvements**:
   - Add progress indicator during URL detection
   - Show recently used download paths
   - Auto-complete filename from URL

---

## Resources

**Documentation**:
- [Tauri Docs](https://tauri.app/v1/guides/)
- [React Docs](https://react.dev/)
- [Jotai Docs](https://jotai.org/)
- [Material UI Docs](https://mui.com/)

**Project-specific**:
- [spec.md](./spec.md) - Feature specification
- [plan.md](./plan.md) - Implementation plan
- [data-model.md](./data-model.md) - Data structures
- [contracts/](./contracts/) - API contracts

**Getting Help**:
- Check existing code in `001-core-downloader` feature for patterns
- Read constitution.md for project principles
- Consult research.md for technical decisions

---

## Summary

### Estimated Time

| Phase | Task | Time |
|-------|------|------|
| 1 | Setup Navigation | 2-3h |
| 2 | Create Add Download UI | 3-4h |
| 3 | Backend Commands | 4-5h |
| 4 | Clipboard Monitoring | 3-4h |
| 5 | Integration & Testing | 2-3h |
| **Total** | | **14-19h** |

### Success Criteria Verification

確認以下所有項目通過：
- [ ] SC-001: 操作步驟減少至 2 步
- [ ] SC-002: 90% 首次成功率（手動測試）
- [ ] SC-003: 剪貼簿偵測 < 500ms
- [ ] SC-004: 附檔名檢測準確率 > 95%（測試案例）
- [ ] SC-005: 完整流程 < 5 秒

### Final Checklist

開發完成前確認：
- [ ] All files created as per file structure
- [ ] All commands registered in lib.rs
- [ ] All tests passing (frontend + backend)
- [ ] No TypeScript errors (`pnpm run build`)
- [ ] No Rust warnings (`cargo clippy`)
- [ ] Performance goals met
- [ ] Manual testing completed for all scenarios
- [ ] Code reviewed for security issues
- [ ] Documentation updated (if public APIs changed)

祝開發順利！🚀
