# Research: Download UI Improvements

**Feature**: Download UI Improvements
**Date**: 2025-12-01
**Status**: Research Complete

## Overview

此文件記錄針對 Download UI Improvements 功能的技術研究結果，解決實作計畫中的所有 NEEDS CLARIFICATION 項目。

## Research Topics

### 1. 路由解決方案（Routing Solution）

#### 問題
Technical Context 中標記為 NEEDS CLARIFICATION：選擇何種路由方案來實現主畫面和新增下載頁面之間的切換？

#### 研究發現

**評估選項：**

1. **React Router 7.x（官方路由庫）**
   - **優點**：
     - 業界標準，文檔完整，社群支持強大
     - 支援 TypeScript
     - 提供 `createBrowserRouter` 和 declarative routes
     - 與 React 19 相容
   - **缺點**：
     - 需要額外依賴（~100KB）
     - 對於僅 2-3 個頁面的簡單桌面應用可能過於複雜
     - 需要調整現有的 App 結構

2. **狀態管理導航（Current Approach）**
   - **優點**：
     - 已在現有程式碼中使用（App.tsx 的 `currentPage` state）
     - 零額外依賴
     - 簡單直接，易於理解和維護
     - 完全滿足當前需求（僅 3 個頁面：main, settings, add-download）
   - **缺點**：
     - 無 URL 導航支援（桌面應用中不重要）
     - 無瀏覽器歷史管理（桌面應用中不需要）
     - 擴展性較差（但當前規模不需要擴展）

3. **TanStack Router**
   - **優點**：
     - 100% TypeScript 類型推斷
     - 更現代化的設計
   - **缺點**：
     - 較新，社群較小
     - 同樣引入額外複雜度

#### 決策

**選擇：延續現有的狀態管理導航方案**

**理由：**
1. **符合憲法原則 III（使用者體驗至上）**：簡單直接，不引入不必要的複雜度
2. **YAGNI 原則**：React Router 提供的功能（URL 路由、深層連結、歷史管理）在桌面應用中不需要
3. **維護性**：與現有架構一致，開發者無需學習新的路由 API
4. **效能**：零額外依賴，應用包大小不增加
5. **當前規模**：3 個頁面的簡單導航完全可以用狀態管理實現

**實作方式：**
- 擴展現有的 `currentPage` state，新增 `'add-download'` 選項
- 在 App.tsx 中條件渲染對應頁面
- 使用 Jotai atom 管理導航狀態（可選，用於跨組件導航）

**替代方案被拒絕的原因：**
- React Router：引入不必要的複雜度和依賴，桌面應用不需要 URL 路由
- TanStack Router：同樣的複雜度問題，且社群支持不如狀態管理方案的簡單性

#### 最佳實踐

參考現有的 App.tsx 實作（第 32 行）：
```typescript
const [currentPage, setCurrentPage] = useState<'main' | 'settings'>('main');
```

擴展為：
```typescript
const [currentPage, setCurrentPage] = useState<'main' | 'settings' | 'add-download'>('main');
```

配合條件渲染：
```typescript
<Box sx={{ flex: 1, overflow: 'auto' }}>
  {currentPage === 'main' && <MainPage />}
  {currentPage === 'settings' && <SettingsPage />}
  {currentPage === 'add-download' && <AddDownloadPage />}
</Box>
```

---

### 2. 前端測試框架（Frontend Testing）

#### 問題
Technical Context 中標記為 NEEDS CLARIFICATION：選擇何種前端測試框架？

#### 研究發現

**評估選項：**

1. **Vitest + React Testing Library**
   - **優點**：
     - Vitest 與 Vite 深度整合（專案已使用 Vite）
     - 快速啟動和執行
     - Jest-compatible API，遷移成本低
     - React Testing Library 是業界標準
   - **缺點**：
     - 需要配置

2. **Jest + React Testing Library**
   - **優點**：
     - 最成熟的選擇
     - 豐富的生態系統
   - **缺點**：
     - 與 Vite 整合需要額外配置（vite-jest）
     - 啟動較慢

#### 決策

**選擇：Vitest + React Testing Library**

**理由：**
1. **與現有技術棧整合**：專案已使用 Vite（package.json 第 33 行），Vitest 是天然選擇
2. **效能**：啟動速度快，測試執行迅速
3. **開發體驗**：Hot Module Reload 支援，watch mode 高效
4. **相容性**：Jest-compatible API，開發者學習曲線低

**實作方式：**
- 安裝依賴：`vitest`, `@testing-library/react`, `@testing-library/user-event`, `jsdom`
- 配置 `vitest.config.ts`
- 為關鍵組件編寫測試（UrlInput, AddDownloadForm, 剪貼簿監控邏輯）

**替代方案被拒絕的原因：**
- Jest：與 Vite 整合需要額外配置，啟動較慢，不如 Vitest 自然

---

### 3. 剪貼簿監控實作方式

#### 問題
如何實現跨平台的剪貼簿監控，並確保效能目標（500ms 內偵測）？

#### 研究發現

**現有實作：**
專案已使用 `tauri-plugin-clipboard-manager`（Cargo.toml 第 23 行）

**實作策略：**

1. **輪詢 vs 事件監聽**
   - Tauri clipboard-manager 使用輪詢機制
   - 建議輪詢間隔：300-500ms（符合 SC-003 的 500ms 要求）
   - 避免過於頻繁的輪詢導致 CPU 使用率上升

2. **URL 驗證策略**
   - **前端驗證**：基本 URL 格式檢查（regex）
   - **後端驗證**：附檔名檢測 + 模組化網站檢測
   - **非阻塞**：使用 async 避免 UI 凍結

#### 決策

**實作方式：**
- 在 React 中使用 `useEffect` + `setInterval` 輪詢剪貼簿（interval: 400ms）
- 呼叫 Tauri command 取得剪貼簿內容
- Rust 後端進行 URL 驗證和檢測
- 偵測成功後自動設定導航狀態並預填 URL

**效能考量：**
- 輪詢間隔 400ms 可確保在 500ms 內偵測到變化
- 使用 debounce 避免重複處理相同內容
- 僅在應用程式 focused 時啟用監控（節省資源）

---

### 4. URL 檢測模組系統架構

#### 問題
如何設計可擴充的 URL 檢測模組系統？

#### 研究發現

**參考設計：**
根據憲法「借鏡設計模式（k2s-Downloaderm）」（constitution.md 第 109-128 行），網站模組應採用模組化設計。

**架構設計：**

1. **Trait-based Module System（Rust）**
   ```rust
   pub trait UrlDetector {
       fn name(&self) -> &str;
       fn can_handle(&self, url: &str) -> bool;
       fn validate(&self, url: &str) -> Result<DownloadInfo, Error>;
   }
   ```

2. **內建檢測器**
   - `BuiltinFileDetector`：基於附檔名檢測（.zip, .pdf, .exe 等）
   - 支援的附檔名列表在 spec.md 假設中定義（第 146 行）

3. **動態載入**
   - 使用 `libloading` crate（已在 Cargo.toml 第 32 行）
   - 模組存放於 `modules/` 目錄
   - 啟動時自動載入所有模組

#### 決策

**實作方式：**
- 定義 `UrlDetector` trait
- 實作 `BuiltinFileDetector` 支援常見附檔名
- 提供模組載入器（`ModuleLoader`）
- 依序執行檢測器直到找到匹配

**擴展性：**
- 第三方可實作 `UrlDetector` trait 建立自訂模組
- 無需修改核心程式碼即可新增網站支援

---

## Summary

### 已解決的 NEEDS CLARIFICATION

1. ✅ **路由解決方案**：使用狀態管理導航（擴展現有 `currentPage` state）
2. ✅ **前端測試框架**：Vitest + React Testing Library
3. ✅ **剪貼簿監控**：400ms 輪詢間隔，非阻塞驗證
4. ✅ **URL 檢測模組**：Trait-based 模組系統，內建附檔名檢測器

### 技術決策總結

| 決策項 | 選擇 | 理由 |
|--------|------|------|
| 路由方案 | 狀態管理導航 | 簡單、零依賴、符合桌面應用需求 |
| 測試框架 | Vitest + RTL | 與 Vite 深度整合、效能優異 |
| 剪貼簿監控 | 400ms 輪詢 | 滿足 500ms 效能目標、跨平台相容 |
| URL 檢測 | Trait-based 模組 | 模組化、可擴充、符合憲法原則 |

### 依賴變更

**新增依賴（開發）：**
- `vitest`: ^2.0.0
- `@testing-library/react`: ^16.0.0
- `@testing-library/user-event`: ^14.0.0
- `jsdom`: ^25.0.0

**無需新增執行時依賴**（路由使用狀態管理）

### 風險評估

**低風險：**
- 所有技術決策都基於現有技術棧
- 使用已驗證的模式（狀態管理、輪詢、trait 系統）
- 無重大架構變更

**中風險：**
- 剪貼簿監控的 CPU 使用率需要實際測試驗證
- URL 檢測模組的效能需要在多模組情況下測試

**緩解措施：**
- 實作時加入效能監控
- 提供設定選項讓使用者調整輪詢間隔
- URL 檢測採用 early-return 優化

---

## Next Steps

研究階段完成，所有 NEEDS CLARIFICATION 已解決。可以進入 Phase 1：設計階段。

**Phase 1 任務：**
1. 生成 `data-model.md`（實體模型定義）
2. 生成 `contracts/`（Tauri commands API 定義）
3. 生成 `quickstart.md`（開發者快速啟動指南）
4. 更新 agent context（CLAUDE.md）
