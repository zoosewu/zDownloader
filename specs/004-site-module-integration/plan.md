# Implementation Plan: 下載網站串接模組系統

**Branch**: `004-site-module-integration` | **Date**: 2025-12-04 | **Spec**: [spec.md](./spec.md)
**Input**: Feature specification from `/specs/004-site-module-integration/spec.md`

**Note**: This template is filled in by the `/speckit.plan` command. See `.specify/templates/commands/plan.md` for the execution workflow.

## Summary

建立可擴展的網站模組系統，讓應用程式能支援特定下載網站的整合（如 Keep2Share）。模組以 JavaScript 編寫並在沙箱環境中執行，透過 manifest 聲明權限和允許存取的網域。系統提供認證代理 API 確保認證資訊安全，並實作全面的安全防護機制（沙箱隔離、資源限制、SSRF 防護、審計日誌）。第一個實作的模組是 K2S (Keep2Share)。

## Technical Context

**Language/Version**:
- 前端: TypeScript 5.8.3, React 18+
- 後端: Rust 1.75+, Tauri 2.x
- 模組語言: JavaScript/TypeScript (透過嵌入式 runtime 執行)

**Primary Dependencies**:
- 前端: Material UI, Jotai (狀態管理), react-i18next (國際化)
- 後端: Tauri 2.x, reqwest (HTTP), tokio (async runtime)
- JavaScript Runtime: deno_core 0.250.0 (V8-based, 選擇理由見 research.md)
- 安全儲存: tauri-plugin-keyring (跨平台 OS keychain 整合)
- 額外: wildmatch 2.1 (網域匹配), tracing 0.1 (審計日誌)

**Storage**:
- JSON 檔案 (模組 manifest, 審計日誌)
- 作業系統安全儲存 (Keychain/Credential Manager - 認證資訊)
- 檔案系統 (模組 JavaScript 檔案)

**Testing**:
- 前端: Vitest
- 後端: cargo test
- 安全測試: 專門的惡意測試模組 (測試沙箱隔離和權限控制)

**Target Platform**: Windows, Linux, macOS (Tauri 跨平台應用)

**Project Type**: Desktop application (Tauri-based)

**Performance Goals**:
- 連結解析到開始下載: <30 秒
- 模組單次請求 CPU 時間: ≤5 秒
- 模組記憶體使用: ≤50MB
- 網路請求頻率: ≤60 請求/分鐘

**Constraints**:
- 模組系統效能開銷: ≤15% (相對通用下載)
- 認證資訊存取延遲: <1 秒
- 模組管理頁面載入時間: <1 秒
- 應用程式大小增加: ~20-30MB (JavaScript runtime)

**Scale/Scope**:
- 初期支援: 1 個模組 (K2S)
- 設計容量: ≥10 個並行模組
- 模組開發時間目標: <4 小時 (熟悉系統的開發者)

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

### I. 模組化優先 (Modularity First) - ✅ PASS

**評估**: 完全符合。此功能的核心就是建立模組系統，所有網站特定邏輯都透過模組實作，核心下載器不包含硬編碼邏輯。

**證據**:
- FR-001: 定義標準模組介面
- FR-002: 自動發現和載入機制
- FR-004: 回退到通用下載模式
- User Story 2 專注於可擴展架構

### II. 效能與可靠性 (Performance & Reliability) - ✅ PASS

**評估**: 符合。有明確的效能目標和資源限制，確保模組不會影響核心下載性能。

**證據**:
- SC-008: 模組系統效能開銷 ≤15%
- FR-028: 資源限制 (CPU ≤5s, Memory ≤50MB)
- SC-002: 連結解析 <30 秒
- FR-035: 自動暫停有問題的模組

### III. 使用者體驗至上 (User Experience First) - ✅ PASS

**評估**: 符合。模組對使用者透明，錯誤訊息友善，支援多語言。

**證據**:
- Clarification: 模組應對使用者透明，除非發生錯誤
- FR-016: 友善的錯誤訊息，包含模組名稱
- FR-021: 處理驗證碼提示
- FR-024: 模組狀態和錯誤訊息顯示

### IV. 跨平台一致性 (Cross-Platform Consistency) - ✅ PASS

**評估**: 符合。使用 Tauri 確保跨平台一致性，認證儲存考慮平台差異。

**證據**:
- Technical Context: Windows, Linux, macOS 支援
- FR-014: 使用作業系統提供的安全儲存機制
- 設定檔使用 JSON 格式 (可跨平台移植)

### V. 安全與隱私 (Security & Privacy) - ✅ PASS

**評估**: 完全符合。此功能高度重視安全，實作全面的安全防護機制。

**證據**:
- FR-013: 認證代理 API，模組無法直接存取認證
- FR-014: 認證加密儲存
- FR-027-035: 9 項安全需求 (沙箱、資源限制、SSRF 防護等)
- User Story 5: 專門的安全測試 (Priority P1)
- SC-010-018: 9 項安全成功標準

**結論**: 此功能完全符合所有憲法核心原則，無需特殊豁免。✅ PASS ALL GATES

## Project Structure

### Documentation (this feature)

```text
specs/[###-feature]/
├── plan.md              # This file (/speckit.plan command output)
├── research.md          # Phase 0 output (/speckit.plan command)
├── data-model.md        # Phase 1 output (/speckit.plan command)
├── quickstart.md        # Phase 1 output (/speckit.plan command)
├── contracts/           # Phase 1 output (/speckit.plan command)
└── tasks.md             # Phase 2 output (/speckit.tasks command - NOT created by /speckit.plan)
```

### Source Code (repository root)

```text
# Backend (Rust/Tauri)
src-tauri/src/
├── modules/                      # 模組系統核心 (擴展現有架構)
│   ├── mod.rs                    # 模組入口
│   ├── interface.rs              # 現有: WebsiteModule trait (保留用於 Rust 模組)
│   ├── registry.rs               # 現有: 模組註冊表 (擴展支援 JS 模組)
│   ├── loader.rs                 # 現有: Rust 動態庫載入器
│   ├── js_runtime.rs             # [新增] JavaScript runtime 管理
│   ├── js_module.rs              # [新增] JavaScript 模組包裝器
│   ├── manifest.rs               # [新增] Manifest 解析和驗證
│   ├── sandbox.rs                # [新增] 沙箱環境和 API 限制
│   ├── permission.rs             # [新增] 權限系統和網域白名單
│   ├── credential_proxy.rs       # [新增] 認證代理 API
│   ├── resource_limiter.rs       # [新增] 資源監控和限制
│   ├── audit.rs                  # [新增] 審計日誌系統
│   ├── url_validator.rs          # [新增] URL 驗證 (SSRF 防護)
│   ├── network_monitor.rs        # [新增] 網路請求監控
│   └── url_detector/             # 現有: URL 偵測器
│       ├── mod.rs
│       └── builtin.rs
├── commands/
│   ├── mod.rs
│   ├── download.rs               # 擴展: 整合模組系統
│   └── module_management.rs     # [新增] 模組管理 Tauri commands
└── storage/
    └── credentials.rs            # [新增] 安全認證儲存

# Frontend (React/TypeScript)
src/
├── pages/
│   └── Settings.tsx              # 擴展: 新增模組管理區域
├── components/
│   ├── ModuleList.tsx            # [新增] 模組列表組件
│   ├── ModuleCard.tsx            # [新增] 模組資訊卡片
│   ├── ModulePermissions.tsx     # [新增] 權限顯示組件
│   └── CredentialManager.tsx     # [新增] 認證資訊管理
├── api/
│   └── modules.ts                # [新增] 模組管理 API 呼叫
├── types/
│   └── module.ts                 # [新增] 模組相關 TypeScript 型別
└── locales/
    ├── zh-TW.json                # 擴展: 模組系統翻譯
    ├── en.json
    └── ja.json

# JavaScript 模組 (獨立於應用程式)
modules/
├── k2s/                          # [新增] Keep2Share 模組
│   ├── manifest.json             # 模組 manifest
│   ├── index.js                  # 模組主程式
│   └── README.md
├── example-downloader/           # 現有: Rust 範例 (保留作為參考)
│   └── ...
└── _template/                    # [新增] 模組開發模板
    ├── manifest.json
    ├── index.js
    └── README.md

# 測試 (安全測試)
src-tauri/src/modules/tests/
├── malicious_modules/            # [新增] 惡意測試模組
│   ├── unauthorized_domain.js    # 測試網域限制
│   ├── resource_exhaustion.js    # 測試資源限制
│   ├── ssrf_attempt.js           # 測試 SSRF 防護
│   └── sandbox_escape.js         # 測試沙箱逃逸
└── security_tests.rs             # [新增] 安全測試套件
```

**Structure Decision**:

此功能採用 **Tauri 桌面應用架構**，前端使用 React/TypeScript，後端使用 Rust。

**關鍵架構決策**：
1. **雙模組系統共存**：保留現有的 Rust 動態庫模組系統，新增 JavaScript 模組支援。這允許過渡期和靈活性。
2. **模組存放位置**：JavaScript 模組存放在專案根目錄的 `modules/` 資料夾，與應用程式程式碼分離。
3. **安全優先**：所有安全相關程式碼（沙箱、權限、審計）都在後端 Rust 實作，確保無法被繞過。
4. **測試隔離**：惡意測試模組獨立存放，確保不會被誤載入到生產環境。

## Complexity Tracking

> **Fill ONLY if Constitution Check has violations that must be justified**

**無需追蹤**：此功能完全符合所有憲法原則，沒有需要特別豁免或解釋的複雜度增加。
