# Tauri Commands API契約：模組管理

**Date**: 2025-12-04
**Feature**: 004-site-module-integration
**Version**: 1.0.0

## 概述

此文件定義模組管理相關的 Tauri commands API 契約。所有 API 遵循 Tauri 的 command 模式，使用 `#[tauri::command]` 標記。

---

## 模組管理 Commands

### 1. get_all_modules

取得所有已載入模組的元資訊列表。

**Command Name**: `get_all_modules`

**請求參數**: 無

**回應型別**: `Result<Vec<ModuleMetadataDto>, String>`

**ModuleMetadataDto**:
```typescript
interface ModuleMetadataDto {
  id: string;
  name: string;
  version: string;
  author: string;
  description: string;
  homepage?: string;
  urlPatterns: string[];
  status: 'pending' | 'loading' | 'active' | 'failed' | 'suspended' | 'unloaded';
  errorMessage?: string;
  loadTime: string; // ISO 8601
  permissions: {
    network: {
      allowDomains: string[];
    };
    credentials?: {
      required: boolean;
      credentialKeys: string[];
    };
  };
}
```

**成功回應範例**:
```json
[
  {
    "id": "k2s",
    "name": "Keep2Share Downloader",
    "version": "1.0.0",
    "author": "Zoo Downloader Community",
    "description": "Download files from Keep2Share (k2s.cc)",
    "homepage": "https://github.com/zoo-downloader/modules/k2s",
    "urlPatterns": [
      "https://k2s.cc/file/*",
      "https://*.k2s.cc/file/*"
    ],
    "status": "active",
    "loadTime": "2025-12-04T10:00:00Z",
    "permissions": {
      "network": {
        "allowDomains": ["k2s.cc", "*.k2s.cc"]
      },
      "credentials": {
        "required": true,
        "credentialKeys": ["username", "password"]
      }
    }
  }
]
```

**錯誤回應範例**:
```json
"Failed to retrieve modules: Registry lock error"
```

**Rust 簽章**:
```rust
#[tauri::command]
pub async fn get_all_modules(
    state: State<'_, ModuleRegistryState>
) -> Result<Vec<ModuleMetadataDto>, String>
```

---

### 2. get_module_detail

取得特定模組的詳細資訊。

**Command Name**: `get_module_detail`

**請求參數**:
```typescript
{
  moduleId: string;
}
```

**回應型別**: `Result<ModuleDetailDto, String>`

**ModuleDetailDto**:
```typescript
interface ModuleDetailDto extends ModuleMetadataDto {
  statistics: {
    totalDownloads: number;
    successfulDownloads: number;
    failedDownloads: number;
    lastUsed?: string; // ISO 8601
  };
  securityEvents: {
    totalViolations: number;
    lastViolation?: string; // ISO 8601
  };
}
```

**成功回應範例**:
```json
{
  "id": "k2s",
  "name": "Keep2Share Downloader",
  // ... (其他 ModuleMetadataDto 欄位)
  "statistics": {
    "totalDownloads": 150,
    "successfulDownloads": 145,
    "failedDownloads": 5,
    "lastUsed": "2025-12-04T15:30:00Z"
  },
  "securityEvents": {
    "totalViolations": 0,
    "lastViolation": null
  }
}
```

**錯誤回應**:
- `"Module not found: {moduleId}"` (模組不存在)
- `"Failed to retrieve module detail: {error}"` (內部錯誤)

**Rust 簽章**:
```rust
#[tauri::command]
pub async fn get_module_detail(
    module_id: String,
    state: State<'_, ModuleRegistryState>
) -> Result<ModuleDetailDto, String>
```

---

### 3. reload_module

重新載入指定的模組。

**Command Name**: `reload_module`

**請求參數**:
```typescript
{
  moduleId: string;
}
```

**回應型別**: `Result<(), String>`

**成功回應**: `Ok(())`

**錯誤回應**:
- `"Module not found: {moduleId}"`
- `"Failed to reload module: {error}"`
- `"Manifest validation failed: {error}"`

**Rust 簽章**:
```rust
#[tauri::command]
pub async fn reload_module(
    module_id: String,
    state: State<'_, ModuleRegistryState>
) -> Result<(), String>
```

---

### 4. unload_module

卸載指定的模組。

**Command Name**: `unload_module`

**請求參數**:
```typescript
{
  moduleId: string;
}
```

**回應型別**: `Result<(), String>`

**成功回應**: `Ok(())`

**錯誤回應**:
- `"Module not found: {moduleId}"`
- `"Failed to unload module: {error}"`

**Rust 簽章**:
```rust
#[tauri::command]
pub async fn unload_module(
    module_id: String,
    state: State<'_, ModuleRegistryState>
) -> Result<(), String>
```

---

### 5. scan_modules

掃描模組目錄並載入所有模組。

**Command Name**: `scan_modules`

**請求參數**: 無

**回應型別**: `Result<ScanResultDto, String>`

**ScanResultDto**:
```typescript
interface ScanResultDto {
  totalFound: number;
  successfullyLoaded: number;
  failed: Array<{
    moduleId: string;
    error: string;
  }>;
}
```

**成功回應範例**:
```json
{
  "totalFound": 3,
  "successfullyLoaded": 2,
  "failed": [
    {
      "moduleId": "broken-module",
      "error": "Manifest validation failed: Invalid version"
    }
  ]
}
```

**錯誤回應**:
- `"Module directory not found: {path}"`
- `"Failed to scan modules: {error}"`

**Rust 簽章**:
```rust
#[tauri::command]
pub async fn scan_modules(
    state: State<'_, ModuleRegistryState>
) -> Result<ScanResultDto, String>
```

---

## 認證管理 Commands

### 6. save_module_credentials

儲存模組的認證資訊到 OS keychain。

**Command Name**: `save_module_credentials`

**請求參數**:
```typescript
{
  moduleId: string;
  credentials: Record<string, string>;
}
```

**回應型別**: `Result<(), String>`

**請求範例**:
```json
{
  "moduleId": "k2s",
  "credentials": {
    "username": "user@example.com",
    "password": "secure_password",
    "apiKey": "optional_api_key"
  }
}
```

**成功回應**: `Ok(())`

**錯誤回應**:
- `"Module not found: {moduleId}"`
- `"Module does not require credentials"`
- `"Missing required credential keys: {keys}"`
- `"Failed to save credentials: {error}"`

**Rust 簽章**:
```rust
#[tauri::command]
pub async fn save_module_credentials(
    module_id: String,
    credentials: HashMap<String, String>,
    state: State<'_, ModuleRegistryState>
) -> Result<(), String>
```

---

### 7. get_module_credentials_status

取得模組認證資訊的狀態（不返回實際認證）。

**Command Name**: `get_module_credentials_status`

**請求參數**:
```typescript
{
  moduleId: string;
}
```

**回應型別**: `Result<CredentialStatusDto, String>`

**CredentialStatusDto**:
```typescript
interface CredentialStatusDto {
  moduleId: string;
  required: boolean;
  configured: boolean;  // 是否已設定
  credentialKeys: string[];
  configuredKeys: string[];  // 已設定的 keys
  missingKeys: string[];     // 缺少的 keys
}
```

**成功回應範例**:
```json
{
  "moduleId": "k2s",
  "required": true,
  "configured": true,
  "credentialKeys": ["username", "password", "apiKey"],
  "configuredKeys": ["username", "password"],
  "missingKeys": ["apiKey"]
}
```

**錯誤回應**:
- `"Module not found: {moduleId}"`
- `"Failed to check credentials: {error}"`

**Rust 簽章**:
```rust
#[tauri::command]
pub async fn get_module_credentials_status(
    module_id: String,
    state: State<'_, ModuleRegistryState>
) -> Result<CredentialStatusDto, String>
```

---

### 8. delete_module_credentials

刪除模組的認證資訊。

**Command Name**: `delete_module_credentials`

**請求參數**:
```typescript
{
  moduleId: string;
}
```

**回應型別**: `Result<(), String>`

**成功回應**: `Ok(())`

**錯誤回應**:
- `"Module not found: {moduleId}"`
- `"No credentials found for module: {moduleId}"`
- `"Failed to delete credentials: {error}"`

**Rust 簽章**:
```rust
#[tauri::command]
pub async fn delete_module_credentials(
    module_id: String,
    state: State<'_, ModuleRegistryState>
) -> Result<(), String>
```

---

## URL 偵測 Commands

### 9. detect_url_module

自動偵測 URL 對應的模組。

**Command Name**: `detect_url_module`

**請求參數**:
```typescript
{
  url: string;
}
```

**回應型別**: `Result<UrlDetectionResultDto, String>`

**UrlDetectionResultDto**:
```typescript
interface UrlDetectionResultDto {
  url: string;
  moduleId?: string;
  moduleName?: string;
  isSupported: boolean;
  requiresAuth: boolean;
}
```

**成功回應範例 (有模組支援)**:
```json
{
  "url": "https://k2s.cc/file/abc123",
  "moduleId": "k2s",
  "moduleName": "Keep2Share Downloader",
  "isSupported": true,
  "requiresAuth": true
}
```

**成功回應範例 (無模組支援)**:
```json
{
  "url": "https://example.com/file.zip",
  "moduleId": null,
  "moduleName": null,
  "isSupported": false,
  "requiresAuth": false
}
```

**錯誤回應**:
- `"Invalid URL: {url}"`
- `"Failed to detect module: {error}"`

**Rust 簽章**:
```rust
#[tauri::command]
pub async fn detect_url_module(
    url: String,
    state: State<'_, ModuleRegistryState>
) -> Result<UrlDetectionResultDto, String>
```

---

## 審計日誌 Commands

### 10. get_module_audit_logs

取得模組的審計日誌。

**Command Name**: `get_module_audit_logs`

**請求參數**:
```typescript
{
  moduleId: string;
  limit?: number;      // 預設 100
  offset?: number;     // 預設 0
  severity?: 'info' | 'warn' | 'error' | 'critical';  // 過濾
}
```

**回應型別**: `Result<AuditLogsDto, String>`

**AuditLogsDto**:
```typescript
interface AuditLogsDto {
  total: number;
  logs: AuditLogEntryDto[];
}

interface AuditLogEntryDto {
  timestamp: string; // ISO 8601
  moduleId: string;
  eventType: string;
  severity: 'INFO' | 'WARN' | 'ERROR' | 'CRITICAL';
  details: any; // JSON object
}
```

**成功回應範例**:
```json
{
  "total": 150,
  "logs": [
    {
      "timestamp": "2025-12-04T15:30:45.123Z",
      "moduleId": "k2s",
      "eventType": "network_request",
      "severity": "INFO",
      "details": {
        "url": "https://k2s.cc/api/login",
        "method": "POST",
        "statusCode": 200
      }
    },
    {
      "timestamp": "2025-12-04T15:25:10.456Z",
      "moduleId": "k2s",
      "eventType": "permission_violation",
      "severity": "ERROR",
      "details": {
        "attemptedDomain": "evil.com",
        "allowedDomains": ["k2s.cc", "*.k2s.cc"]
      }
    }
  ]
}
```

**錯誤回應**:
- `"Module not found: {moduleId}"`
- `"Failed to retrieve audit logs: {error}"`

**Rust 簽章**:
```rust
#[tauri::command]
pub async fn get_module_audit_logs(
    module_id: String,
    limit: Option<usize>,
    offset: Option<usize>,
    severity: Option<String>,
    state: State<'_, AuditState>
) -> Result<AuditLogsDto, String>
```

---

## 擴展的下載 Commands

### 11. create_download_with_module (擴展現有)

使用模組建立下載任務（擴展現有的 create_download）。

**Command Name**: `create_download_with_module`

**請求參數**:
```typescript
{
  url: string;
  filename?: string;
  downloadPath?: string;
  moduleId?: string;  // 可選，如果不提供則自動偵測
}
```

**回應型別**: `Result<String, String>` (返回 task_id)

**成功回應**: `"task-uuid-here"`

**錯誤回應**:
- `"Invalid URL: {url}"`
- `"Module not found: {moduleId}"`
- `"Module requires authentication but no credentials configured"`
- `"Module is not active"`
- `"Failed to create download: {error}"`

**Rust 簽章**:
```rust
#[tauri::command]
pub async fn create_download_with_module(
    url: String,
    filename: Option<String>,
    download_path: Option<String>,
    module_id: Option<String>,
    download_state: State<'_, DownloadState>,
    module_state: State<'_, ModuleRegistryState>
) -> Result<String, String>
```

---

## 前端使用範例

### TypeScript API Client

```typescript
// src/api/modules.ts
import { invoke } from '@tauri-apps/api';

export interface ModuleMetadata {
  id: string;
  name: string;
  version: string;
  // ... 其他欄位
}

export class ModuleAPI {
  static async getAllModules(): Promise<ModuleMetadata[]> {
    return await invoke<ModuleMetadata[]>('get_all_modules');
  }

  static async getModuleDetail(moduleId: string) {
    return await invoke('get_module_detail', { moduleId });
  }

  static async saveCredentials(
    moduleId: string,
    credentials: Record<string, string>
  ): Promise<void> {
    return await invoke('save_module_credentials', {
      moduleId,
      credentials
    });
  }

  static async detectUrlModule(url: string) {
    return await invoke('detect_url_module', { url });
  }
}
```

### React Hook 範例

```typescript
// src/hooks/useModules.ts
import { useQuery } from '@tanstack/react-query';
import { ModuleAPI } from '../api/modules';

export function useModules() {
  return useQuery({
    queryKey: ['modules'],
    queryFn: () => ModuleAPI.getAllModules()
  });
}

export function useModuleDetail(moduleId: string) {
  return useQuery({
    queryKey: ['module', moduleId],
    queryFn: () => ModuleAPI.getModuleDetail(moduleId),
    enabled: !!moduleId
  });
}
```

---

## 錯誤處理模式

所有 commands 使用統一的錯誤處理模式：

1. **參數驗證錯誤**: `"Invalid parameter: {reason}"`
2. **實體不存在**: `"{Entity} not found: {id}"`
3. **權限錯誤**: `"Permission denied: {reason}"`
4. **狀態錯誤**: `"Invalid state: {reason}"`
5. **內部錯誤**: `"Failed to {action}: {error}"`

前端應該根據錯誤訊息的前綴進行分類處理。

---

## 安全考量

1. **認證隔離**: 認證資訊永不返回到前端，僅返回狀態
2. **輸入驗證**: 所有使用者輸入必須驗證（URL, module_id 等）
3. **權限檢查**: 模組操作前檢查模組權限
4. **審計記錄**: 所有敏感操作記錄到審計日誌

---

## 版本控制

API 版本: **1.0.0**

變更時更新版本號並記錄 breaking changes。
