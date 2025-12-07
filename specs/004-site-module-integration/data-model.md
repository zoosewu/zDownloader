# Data Model: 下載網站串接模組系統

**Date**: 2025-12-04
**Feature**: 004-site-module-integration
**Phase**: Phase 1 - Design & Contracts

## 概述

此文件定義模組系統的所有資料實體、欄位、關係和驗證規則。所有實體設計都考慮了安全性、擴展性和跨平台相容性。

---

## 核心實體

### 1. SiteModule (網站模組)

JavaScript 模組的執行時表示，封裝特定網站的下載邏輯。

**欄位：**

| 欄位名稱 | 型別 | 必填 | 說明 | 驗證規則 |
|---------|------|------|------|---------|
| `id` | String | ✅ | 模組唯一識別碼 (如 "k2s") | 小寫英數字和連字號，3-50 字元 |
| `manifest` | ModuleManifest | ✅ | 模組 manifest 資訊 | 必須通過 manifest 驗證 |
| `code` | String | ✅ | JavaScript 原始碼 | 非空，< 1MB |
| `runtime` | JsRuntimeHandle | ✅ | JavaScript runtime 控制碼 | 有效的 runtime 實例 |
| `status` | ModuleStatus | ✅ | 模組狀態 | 見 ModuleStatus enum |
| `load_time` | DateTime | ✅ | 載入時間戳 | ISO 8601 format |
| `error_message` | Option\<String\> | ❌ | 錯誤訊息 (如果狀態為 Failed) | < 1000 字元 |

**狀態轉換：**
```
Pending → Loading → Active ↔ Suspended → Unloaded
             ↓
           Failed
```

**Rust 定義：**
```rust
#[derive(Debug, Clone)]
pub struct SiteModule {
    pub id: String,
    pub manifest: ModuleManifest,
    pub code: String,
    pub runtime: JsRuntimeHandle,
    pub status: ModuleStatus,
    pub load_time: DateTime<Utc>,
    pub error_message: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum ModuleStatus {
    Pending,
    Loading,
    Active,
    Failed,
    Suspended,  // 偵測到可疑行為時暫停
    Unloaded,
}
```

---

### 2. ModuleManifest (模組 Manifest)

模組的元資訊和權限聲明，JSON 格式儲存。

**欄位：**

| 欄位名稱 | 型別 | 必填 | 說明 | 驗證規則 |
|---------|------|------|------|---------|
| `id` | String | ✅ | 模組 ID | 與模組資料夾名稱一致 |
| `name` | String | ✅ | 模組顯示名稱 | 1-100 字元 |
| `version` | String | ✅ | 語義化版本號 | 符合 semver (如 "1.0.0") |
| `author` | String | ✅ | 作者名稱或組織 | 1-100 字元 |
| `description` | String | ✅ | 模組描述 | 1-500 字元 |
| `homepage` | Option\<String\> | ❌ | 模組首頁 URL | 有效的 HTTP/HTTPS URL |
| `url_patterns` | Vec\<String\> | ✅ | 支援的 URL 模式 | 至少 1 個，每個 < 200 字元 |
| `permissions` | ModulePermissions | ✅ | 權限聲明 | 見 ModulePermissions |

**JSON 範例：**
```json
{
  "id": "k2s",
  "name": "Keep2Share Downloader",
  "version": "1.0.0",
  "author": "Zoo Downloader Community",
  "description": "Download files from Keep2Share (k2s.cc) with automatic authentication and CAPTCHA handling",
  "homepage": "https://github.com/zoo-downloader/modules/k2s",
  "urlPatterns": [
    "https://k2s.cc/file/*",
    "https://*.k2s.cc/file/*"
  ],
  "permissions": {
    "network": {
      "allowDomains": [
        "k2s.cc",
        "*.k2s.cc",
        "api.keep2share.cc"
      ]
    },
    "credentials": {
      "required": true,
      "credentialKeys": ["username", "password", "apiKey"]
    }
  }
}
```

**Rust 定義：**
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ModuleManifest {
    pub id: String,
    pub name: String,
    pub version: String,
    pub author: String,
    pub description: String,
    pub homepage: Option<String>,
    pub url_patterns: Vec<String>,
    pub permissions: ModulePermissions,
}

impl ModuleManifest {
    pub fn validate(&self) -> Result<(), String> {
        // ID 驗證
        if !self.id.chars().all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-') {
            return Err("Invalid module ID".to_string());
        }

        // 版本驗證
        semver::Version::parse(&self.version)
            .map_err(|_| "Invalid semver version".to_string())?;

        // URL patterns 驗證
        if self.url_patterns.is_empty() {
            return Err("At least one URL pattern required".to_string());
        }

        // 權限驗證
        self.permissions.validate()?;

        Ok(())
    }
}
```

---

### 3. ModulePermissions (模組權限)

模組聲明的權限集合，強制執行安全邊界。

**欄位：**

| 欄位名稱 | 型別 | 必填 | 說明 | 驗證規則 |
|---------|------|------|------|---------|
| `network` | NetworkPermission | ✅ | 網路存取權限 | 必須有至少 1 個允許網域 |
| `credentials` | CredentialPermission | ❌ | 認證存取權限 | 如果 required=true，需指定 keys |

**子結構：**

#### NetworkPermission
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NetworkPermission {
    pub allow_domains: Vec<String>,  // 允許的網域，支援通配符
}

impl NetworkPermission {
    pub fn is_domain_allowed(&self, domain: &str) -> bool {
        self.allow_domains.iter().any(|pattern| {
            WildMatch::new(pattern).matches(domain)
        })
    }
}
```

#### CredentialPermission
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CredentialPermission {
    pub required: bool,
    pub credential_keys: Vec<String>,  // 需要的認證欄位
}
```

---

### 4. ModuleCredentials (模組認證資訊)

模組使用的認證資料，由主程式安全儲存，永不暴露給模組。

**欄位：**

| 欄位名稱 | 型別 | 必填 | 說明 | 驗證規則 |
|---------|------|------|------|---------|
| `module_id` | String | ✅ | 模組 ID | 必須是已註冊的模組 |
| `credentials` | HashMap\<String, String\> | ✅ | 認證鍵值對 | 至少 1 個鍵值對 |
| `created_at` | DateTime | ✅ | 建立時間 | ISO 8601 format |
| `updated_at` | DateTime | ✅ | 最後更新時間 | ISO 8601 format |

**儲存格式：**
- **鍵名前綴**：`zoo-downloader:module:{module_id}:{key}`
- **儲存位置**：OS keychain (macOS Keychain, Windows Credential Manager, Linux Secret Service)

**範例：**
```rust
ModuleCredentials {
    module_id: "k2s",
    credentials: {
        "username": "user@example.com",
        "password": "encrypted_password",
        "apiKey": "api_key_value"
    },
    created_at: "2025-12-04T10:00:00Z",
    updated_at: "2025-12-04T10:00:00Z"
}
```

**Rust 定義：**
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ModuleCredentials {
    pub module_id: String,
    pub credentials: HashMap<String, String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl ModuleCredentials {
    // 儲存到 OS keychain
    pub async fn save(&self) -> Result<(), String> {
        for (key, value) in &self.credentials {
            let keychain_key = format!("zoo-downloader:module:{}:{}", self.module_id, key);
            tauri_plugin_keyring::set_password("zoo-downloader", &keychain_key, value)?;
        }
        Ok(())
    }

    // 從 OS keychain 載入
    pub async fn load(module_id: &str, keys: &[String]) -> Result<Self, String> {
        let mut credentials = HashMap::new();
        for key in keys {
            let keychain_key = format!("zoo-downloader:module:{}:{}", module_id, key);
            let value = tauri_plugin_keyring::get_password("zoo-downloader", &keychain_key)?;
            credentials.insert(key.clone(), value);
        }
        Ok(Self {
            module_id: module_id.to_string(),
            credentials,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        })
    }
}
```

---

### 5. DownloadTask (擴展)

擴展現有的 DownloadTask 以支援模組系統。

**新增欄位：**

| 欄位名稱 | 型別 | 必填 | 說明 | 驗證規則 |
|---------|------|------|------|---------|
| `module_id` | Option\<String\> | ❌ | 使用的模組 ID | 如果使用模組，必須是有效 ID |
| `module_metadata` | Option\<ModuleMetadata\> | ❌ | 模組元資訊快照 | 僅用於錯誤報告 |

**Rust 定義：**
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DownloadTask {
    // ... 現有欄位 ...

    // 新增欄位
    pub module_id: Option<String>,
    pub module_metadata: Option<ModuleMetadata>,
}
```

---

## 安全相關實體

### 6. ModuleSandbox (模組沙箱)

隔離的 JavaScript 執行環境。

**欄位：**

| 欄位名稱 | 型別 | 必填 | 說明 |
|---------|------|------|------|
| `runtime` | JsRuntime | ✅ | Deno core runtime 實例 |
| `allowed_apis` | HashSet\<String\> | ✅ | 允許的 API 清單 |
| `blocked_globals` | Vec\<String\> | ✅ | 被阻止的全域物件 |

**被阻止的 API：**
- `eval`
- `Function` constructor
- `require`
- `process`
- `__proto__`
- Dynamic `import()`
- `WebAssembly`

**Rust 定義：**
```rust
pub struct ModuleSandbox {
    runtime: JsRuntime,
    allowed_apis: HashSet<String>,
}

impl ModuleSandbox {
    pub fn new() -> Self {
        let mut runtime = JsRuntime::new(RuntimeOptions {
            extensions: vec![],  // 空的擴展 = 無內建 API
            inspector: false,
            ..Default::default()
        });

        // 注入安全的 API
        runtime.execute_script(
            "sandbox_init",
            r#"
                // 刪除危險的全域物件
                delete globalThis.eval;
                delete globalThis.Function;
                delete globalThis.__proto__;
                // 凍結原型
                Object.freeze(Object.prototype);
                Object.freeze(Array.prototype);
            "#
        ).unwrap();

        Self {
            runtime,
            allowed_apis: HashSet::new(),
        }
    }
}
```

---

### 7. ResourceLimits (資源限制)

模組的資源使用限制。

**欄位：**

| 欄位名稱 | 型別 | 預設值 | 說明 |
|---------|------|--------|------|
| `max_cpu_time_ms` | u64 | 5000 | 單次請求最大 CPU 時間 (毫秒) |
| `max_memory_mb` | u64 | 50 | 最大記憶體使用 (MB) |
| `max_heap_mb` | u64 | 100 | 最大堆積大小 (MB) |
| `max_requests_per_minute` | u32 | 60 | 每分鐘最大網路請求數 |
| `max_response_size_mb` | u64 | 10 | 單次回應最大大小 (MB) |

**Rust 定義：**
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ResourceLimits {
    pub max_cpu_time_ms: u64,
    pub max_memory_mb: u64,
    pub max_heap_mb: u64,
    pub max_requests_per_minute: u32,
    pub max_response_size_mb: u64,
}

impl Default for ResourceLimits {
    fn default() -> Self {
        Self {
            max_cpu_time_ms: 5000,
            max_memory_mb: 50,
            max_heap_mb: 100,
            max_requests_per_minute: 60,
            max_response_size_mb: 10,
        }
    }
}
```

---

### 8. AuditLogEntry (審計日誌條目)

記錄模組行為的日誌條目。

**欄位：**

| 欄位名稱 | 型別 | 必填 | 說明 |
|---------|------|------|------|
| `timestamp` | DateTime | ✅ | 事件時間戳 |
| `module_id` | String | ✅ | 模組 ID |
| `event_type` | AuditEventType | ✅ | 事件類型 |
| `severity` | Severity | ✅ | 嚴重程度 |
| `details` | serde_json::Value | ✅ | 事件詳細資訊 (JSON) |

**事件類型：**
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AuditEventType {
    ModuleLoaded,
    ModuleUnloaded,
    ModuleSuspended,
    NetworkRequest,
    PermissionViolation,
    ResourceLimitExceeded,
    SsrfAttempt,
    SandboxEscape,
    CredentialAccess,
    SuspiciousBehavior,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum Severity {
    Info,
    Warn,
    Error,
    Critical,
}
```

**JSON 範例：**
```json
{
  "timestamp": "2025-12-04T10:30:45.123Z",
  "moduleId": "k2s",
  "eventType": "permission_violation",
  "severity": "ERROR",
  "details": {
    "attemptedDomain": "malicious.com",
    "allowedDomains": ["k2s.cc", "*.k2s.cc"],
    "stackTrace": "..."
  }
}
```

---

### 9. UrlValidationResult (URL 驗證結果)

URL 驗證的結果。

**欄位：**

| 欄位名稱 | 型別 | 說明 |
|---------|------|------|
| `is_valid` | bool | 是否通過驗證 |
| `url` | String | 原始 URL |
| `resolved_ip` | Option\<IpAddr\> | DNS 解析的 IP |
| `violation` | Option\<String\> | 違規原因 (如果 is_valid=false) |

**Rust 定義：**
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UrlValidationResult {
    pub is_valid: bool,
    pub url: String,
    pub resolved_ip: Option<IpAddr>,
    pub violation: Option<String>,
}
```

---

### 10. NetworkMonitorStats (網路監控統計)

模組的網路請求統計。

**欄位：**

| 欄位名稱 | 型別 | 說明 |
|---------|------|------|
| `module_id` | String | 模組 ID |
| `window_start` | DateTime | 統計視窗開始時間 |
| `request_count` | u32 | 請求數量 |
| `unique_domains` | HashSet\<String\> | 請求的唯一網域集合 |
| `total_bytes_sent` | u64 | 總發送字節數 |
| `total_bytes_received` | u64 | 總接收字節數 |

**Rust 定義：**
```rust
#[derive(Debug, Clone)]
pub struct NetworkMonitorStats {
    pub module_id: String,
    pub window_start: DateTime<Utc>,
    pub request_count: u32,
    pub unique_domains: HashSet<String>,
    pub total_bytes_sent: u64,
    pub total_bytes_received: u64,
}

impl NetworkMonitorStats {
    // 檢測可疑行為
    pub fn is_suspicious(&self) -> bool {
        // 短時間內請求多個不同網域 (資料外洩徵兆)
        if self.unique_domains.len() > 3 {
            return true;
        }

        // 超過頻率限制
        let duration = Utc::now() - self.window_start;
        if duration.num_seconds() < 60 && self.request_count > 60 {
            return true;
        }

        false
    }
}
```

---

## 實體關係圖

```
ModuleRegistry
    ├─ 1:N ─> SiteModule
    │            ├─ 1:1 ─> ModuleManifest
    │            │            └─ 1:1 ─> ModulePermissions
    │            ├─ 1:1 ─> ModuleSandbox
    │            └─ 1:1 ─> ResourceLimits
    │
    ├─ 1:N ─> ModuleCredentials
    │
    └─ 1:N ─> AuditLogEntry

DownloadTask
    └─ N:1 ─> SiteModule (optional)

NetworkMonitor
    └─ 1:N ─> NetworkMonitorStats
                  └─ N:1 ─> SiteModule
```

---

## 資料驗證規則總結

### 必填欄位驗證
- 所有標記為必填 (✅) 的欄位在建立實體時必須提供
- 使用 Rust 的 type system 強制執行 (Option<T> vs T)

### 字串長度驗證
- 模組 ID: 3-50 字元
- 顯示名稱: 1-100 字元
- 描述: 1-500 字元
- 錯誤訊息: < 1000 字元
- URL patterns: < 200 字元/個

### 格式驗證
- 版本號: 符合 semver (使用 `semver` crate 驗證)
- URL: 有效的 HTTP/HTTPS URL (使用 `url` crate 驗證)
- 時間戳: ISO 8601 format (使用 `chrono` crate)

### 業務邏輯驗證
- 模組 ID 必須唯一 (在 ModuleRegistry 層級檢查)
- URL patterns 至少要有 1 個
- 網域白名單至少要有 1 個
- 如果 credentials.required=true，必須提供 credential_keys

---

## 序列化注意事項

**重要**：所有與前端通訊的資料結構必須使用 `#[serde(rename_all = "camelCase")]`，確保 JSON 欄位名稱為 camelCase 格式。

**範例：**
```rust
#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ModuleMetadata {
    pub module_id: String,      // JSON: "moduleId"
    pub display_name: String,   // JSON: "displayName"
    pub is_active: bool,        // JSON: "isActive"
}
```

詳見 CLAUDE.md 中的「API 命名規範與序列化」章節。

---

## 遷移策略

由於專案已有 Rust 動態庫模組系統，需要支援兩種模組類型：

```rust
pub enum ModuleType {
    Rust(Box<dyn WebsiteModule>),      // 現有的 Rust 模組
    JavaScript(SiteModule),              // 新的 JavaScript 模組
}
```

ModuleRegistry 需要同時管理兩種類型，提供統一的介面。

---

此資料模型設計確保了：
- ✅ 安全性：認證隔離、沙箱執行、權限控制
- ✅ 可擴展性：支援新增欄位和權限類型
- ✅ 跨平台：使用標準格式和平台無關的抽象
- ✅ 可測試性：清晰的邊界和驗證規則
- ✅ 可維護性：結構化設計和文件完整
