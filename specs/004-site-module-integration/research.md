# Research Report: 下載網站串接模組系統

**Date**: 2025-12-04
**Feature**: 004-site-module-integration
**Phase**: Phase 0 - Technical Research

## 執行摘要

本研究報告解決了模組系統實作的所有技術不確定性。關鍵決策包括：選擇 **deno_core** 作為 JavaScript runtime、使用 **tauri-plugin-keyring** 進行認證儲存、以及採用 Rust 標準庫的 IP 驗證方法進行 SSRF 防護。所有技術選擇都優先考慮安全性、效能和跨平台相容性。

---

## 1. JavaScript Runtime 選擇

### 決策：選擇 deno_core

**選擇理由：**
- **成熟且經過實戰驗證**：Deno 在生產環境中廣泛使用，包括 LangChain Sandbox 和 LMStudio 等知名專案
- **內建安全機制**：預設沙箱執行，無 OS 存取權限，除非明確授權
- **效能最佳**：基於 V8 引擎，是最快的 JavaScript 引擎之一
- **良好的 Rust 整合**：提供 `JsRuntime` 抽象和 ops 機制，方便綁定 Rust 函數
- **2025 年趨勢**：LLM-generated code 執行的首選工具

**參考實作：**
- [js-sandbox](https://github.com/Bromeon/js-sandbox): 基於 Deno 的安全 JavaScript 沙箱庫
- [Running JavaScript in Rust with Deno](https://austinpoor.com/blog/js-in-rs): 實用教學
- [Roll your own JavaScript runtime](https://deno.com/blog/roll-your-own-javascript-runtime-pt2): 官方指南

### 替代方案分析

#### Boa (Pure Rust Engine)
- **優點**：純 Rust 實作，無需 C++ 依賴，94% ECMAScript 規範符合度
- **缺點**：效能較 V8 差，生態系統較小，仍在發展中
- **拒絕理由**：效能不足以滿足 <30 秒連結解析的需求

#### rquickjs (QuickJS Rust Bindings)
- **優點**：輕量級，適合嵌入式場景，簡單易用
- **缺點**：效能較差，缺乏企業級支援，安全特性較少
- **拒絕理由**：不符合企業級安全和效能需求

#### 直接使用 rusty_v8
- **優點**：最大控制權，直接存取 V8 API
- **缺點**：需要自行實作所有安全機制，開發複雜度高
- **拒絕理由**：deno_core 已提供良好的抽象，無需重新發明輪子

### 實作細節

**Cargo 依賴：**
```toml
[dependencies]
deno_core = "0.250.0"  # 使用最新穩定版本
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
```

**基本架構：**
```rust
use deno_core::{JsRuntime, RuntimeOptions};

// 建立沙箱 runtime
let mut runtime = JsRuntime::new(RuntimeOptions {
    // 禁用所有內建 API
    extensions: vec![],
    // 啟用 inspector (開發時)
    inspector: false,
    ..Default::default()
});

// 執行模組程式碼
runtime.execute_script("[module_name]", module_code)?;
```

**資源影響：**
- 應用程式大小增加：約 25-30MB (V8 引擎)
- 記憶體基礎開銷：約 10-15MB per runtime
- 啟動時間影響：首次初始化約 50-100ms

**Sources:**
- [deno_core - crates.io](https://crates.io/crates/deno_core)
- [GitHub - Bromeon/js-sandbox](https://github.com/Bromeon/js-sandbox)
- [Deno Security and Permissions](https://docs.deno.com/runtime/fundamentals/security/)

---

## 2. 安全認證儲存機制

### 決策：使用 tauri-plugin-keyring

**選擇理由：**
- **跨平台原生支援**：自動整合各平台的原生認證儲存
  - macOS: Keychain
  - Windows: Credential Manager
  - Linux: GNOME Keyring / Secret Service
- **簡單 API**：易於整合到 Tauri 應用中
- **持久化**：即使應用解除安裝後重新安裝，認證資訊仍保留
- **社群支援**：活躍維護，文件完整

**參考專案：**
- [tauri-plugin-keyring](https://github.com/HuakunShen/tauri-plugin-keyring): 基於 rust keyring crate 的 Tauri 插件
- [Building a Password Manager with Tauri](https://mhmtsr.medium.com/building-a-local-first-password-manager-tauri-rust-sqlx-and-sqlcipher-09d0134db5bc): 實戰案例

### 替代方案分析

#### Tauri Stronghold Plugin
- **優點**：IOTA 開發的安全儲存引擎，加密強度高
- **缺點**：不使用系統 keychain，實作複雜，過於重量級
- **拒絕理由**：不符合「使用 OS 提供的安全儲存」的需求

#### tauri-plugin-secure-storage
- **優點**：基於系統 keyring
- **缺點**：較少維護，文件不完整
- **拒絕理由**：tauri-plugin-keyring 更活躍且成熟

#### 自訂加密檔案儲存
- **優點**：完全控制
- **缺點**：需要自行處理加密、密鑰管理、跨平台差異
- **拒絕理由**：違反「不要重新發明輪子」原則，安全風險高

### 實作細節

**Cargo 依賴：**
```toml
[dependencies]
tauri-plugin-keyring = "0.1"
```

**API 使用範例：**
```rust
// 儲存認證
tauri_plugin_keyring::set_password("zoo-downloader", "k2s-username", "user@example.com")?;
tauri_plugin_keyring::set_password("zoo-downloader", "k2s-password", "secure_password")?;

// 讀取認證
let username = tauri_plugin_keyring::get_password("zoo-downloader", "k2s-username")?;
let password = tauri_plugin_keyring::get_password("zoo-downloader", "k2s-password")?;

// 刪除認證
tauri_plugin_keyring::delete_password("zoo-downloader", "k2s-username")?;
```

**認證資料結構：**
```rust
#[derive(Serialize, Deserialize)]
struct ModuleCredentials {
    module_id: String,
    credentials: HashMap<String, String>, // 靈活的 key-value 儲存
}
```

**Sources:**
- [tauri-plugin-keyring - GitHub](https://github.com/HuakunShen/tauri-plugin-keyring)
- [Tauri Discussion: How to store user credentials](https://github.com/tauri-apps/tauri/discussions/1222)
- [Building a Local-First Password Manager with Tauri](https://mhmtsr.medium.com/building-a-local-first-password-manager-tauri-rust-sqlx-and-sqlcipher-09d0134db5bc)

---

## 3. SSRF 防護與內網 IP 檢測

### 決策：使用 Rust 標準庫 `std::net` + 自訂驗證邏輯

**選擇理由：**
- **CVE 已修復**：Rust 1.53.0+ 已修復 octal/hexadecimal IP 解析漏洞 (CVE-2021-29922)
- **內建方法**：`Ipv4Addr::is_private()` 和 `!Ipv4Addr::is_global()` 可正確識別私有 IP
- **高效能**：無需外部依賴，零開銷
- **完整覆蓋**：支援 IPv4 和 IPv6

**關鍵 CVE 警告（已修復）：**
Rust 1.52.0 及以下版本存在嚴重的 IP 解析漏洞：
- `010.8.8.8` (octal) 會被解析為 `8.8.8.8` 而非 `10.8.8.8`
- 攻擊者可利用此漏洞繞過 IP 白名單/黑名單檢查

**解決方案：** 確保專案使用 Rust 1.53.0 或更高版本

### 實作細節

**內網 IP 範圍檢測：**
```rust
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};

fn is_internal_ip(ip: &IpAddr) -> bool {
    match ip {
        IpAddr::V4(ipv4) => {
            // RFC 1918 私有網路
            ipv4.is_private()
            // 額外檢查
            || ipv4.is_loopback()         // 127.0.0.0/8
            || ipv4.is_link_local()       // 169.254.0.0/16
            || ipv4.is_documentation()    // 測試用 IP
            || ipv4.octets()[0] == 0      // 0.0.0.0/8
        },
        IpAddr::V6(ipv6) => {
            ipv6.is_loopback()            // ::1
            || ipv6.is_unicast_link_local() // fe80::/10
            || !ipv6.is_global()
            // 檢查 IPv4-mapped IPv6 (::ffff:192.168.1.1)
            || matches!(ipv6.octets()[..12], [0,0,0,0,0,0,0,0,0,0,0xff,0xff])
        }
    }
}

fn validate_url_for_ssrf(url: &str) -> Result<(), String> {
    let parsed = url::Url::parse(url)
        .map_err(|e| format!("Invalid URL: {}", e))?;

    // 檢查協議
    match parsed.scheme() {
        "http" | "https" => {},
        _ => return Err("Only HTTP/HTTPS protocols allowed".to_string()),
    }

    // 解析主機名
    let host = parsed.host_str()
        .ok_or("Missing host")?;

    // DNS 解析前驗證
    if let Ok(ip) = host.parse::<IpAddr>() {
        if is_internal_ip(&ip) {
            return Err("Internal IP address not allowed".to_string());
        }
    }

    // DNS 解析後驗證 (防止 DNS rebinding)
    use std::net::ToSocketAddrs;
    let addrs = format!("{}:80", host)
        .to_socket_addrs()
        .map_err(|e| format!("DNS resolution failed: {}", e))?;

    for addr in addrs {
        if is_internal_ip(&addr.ip()) {
            return Err("DNS resolved to internal IP".to_string());
        }
    }

    Ok(())
}
```

**私有 IP 範圍覆蓋：**
- `10.0.0.0/8` ✅
- `172.16.0.0/12` (172.16.0.0 - 172.31.255.255) ✅
- `192.168.0.0/16` ✅
- `127.0.0.0/8` (Loopback) ✅
- `169.254.0.0/16` (Link-local) ✅
- `100.64.0.0/10` (Shared Address Space) ✅
- IPv6 loopback (::1) ✅
- IPv6 link-local (fe80::/10) ✅
- IPv4-mapped IPv6 (::ffff:127.0.0.1) ✅

**Sources:**
- [Go, Rust net library affected by critical IP validation vulnerability](https://www.bleepingcomputer.com/news/security/go-rust-net-library-affected-by-critical-ip-address-validation-vulnerability/)
- [CVE-2021-29922 - Rust SSRF vulnerability](https://sick.codes/sick-2021-015/)
- [OWASP SSRF Prevention Cheat Sheet](https://cheatsheetseries.owasp.org/cheatsheets/Server_Side_Request_Forgery_Prevention_Cheat_Sheet.html)
- [Rust std::net::Ipv4Addr Documentation](https://doc.rust-lang.org/std/net/struct.Ipv4Addr.html)

---

## 4. 通配符網域匹配

### 決策：使用 wildmatch crate

**選擇理由：**
- **專為通配符設計**：不需要轉換為正則表達式
- **效能優異**：比 regex 快，線性時間複雜度，零記憶體配置
- **簡單 API**：直觀易用
- **Cloudflare 使用**：經過大規模生產驗證

**參考：**
- [Cloudflare: Wildcard support and open-source crate](https://blog.cloudflare.com/wildcard-rules/): Cloudflare 的實戰經驗

### 替代方案分析

#### domain-lookup-tree
- **優點**：專為網域查找優化的樹狀結構
- **缺點**：使用前導點 (`.twitter.com`) 而非星號語法
- **拒絕理由**：語法不直觀，與常見的 `*.example.com` 慣例不符

#### glob-match / globset
- **優點**：通用 glob 匹配，支援複雜模式
- **缺點**：功能過於複雜，針對檔案路徑設計
- **拒絕理由**：對網域匹配而言過於重量級

### 實作細節

**Cargo 依賴：**
```toml
[dependencies]
wildmatch = "2.1"
```

**使用範例：**
```rust
use wildmatch::WildMatch;

fn domain_matches_pattern(domain: &str, pattern: &str) -> bool {
    let matcher = WildMatch::new(pattern);
    matcher.matches(domain)
}

// 測試案例
assert!(domain_matches_pattern("k2s.cc", "k2s.cc"));
assert!(domain_matches_pattern("cdn.k2s.cc", "*.k2s.cc"));
assert!(domain_matches_pattern("api.cdn.k2s.cc", "*.k2s.cc")); // 匹配所有子網域
assert!(!domain_matches_pattern("k2s.cc", "*.k2s.cc")); // 不匹配頂級網域本身

// 多層通配符
assert!(domain_matches_pattern("a.b.c.example.com", "*.example.com"));
```

**Manifest 網域白名單範例：**
```json
{
  "permissions": {
    "network": {
      "allow_domains": [
        "k2s.cc",
        "*.k2s.cc",
        "api.keep2share.cc"
      ]
    }
  }
}
```

**Sources:**
- [wildmatch - crates.io](https://crates.io/crates/wildmatch)
- [GitHub - becheran/wildmatch](https://github.com/becheran/wildmatch)
- [Cloudflare: Wildcard Rules](https://blog.cloudflare.com/wildcard-rules/)

---

## 5. 審計日誌系統

### 決策：使用 tracing + tracing-appender + tracing-subscriber (JSON 格式)

**選擇理由：**
- **現代化標準**：2025 年 Rust 生態系統的推薦選擇
- **結構化日誌**：原生支援 JSON 輸出
- **非阻塞**：tracing-appender 支援非同步寫入，不影響主執行緒
- **豐富生態**：與 Tokio、Tauri 完美整合
- **可擴展**：支援多種輸出格式和目標

### 替代方案分析

#### log4rs
- **優點**：內建日誌輪轉、複雜配置、類似 Log4j
- **缺點**：較舊的 API，非結構化為主
- **拒絕理由**：不符合現代化架構，結構化日誌支援較弱

#### slog
- **優點**：效能優異，結構化日誌
- **缺點**：學習曲線陡峭，社群較 tracing 小
- **拒絕理由**：tracing 有更好的生態和文件

### 實作細節

**Cargo 依賴：**
```toml
[dependencies]
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["json", "env-filter"] }
tracing-appender = "0.2"
serde_json = "1.0"
```

**日誌系統架構：**
```rust
use tracing_subscriber::{fmt, prelude::*, EnvFilter};
use tracing_appender::{non_blocking, rolling};

// 初始化審計日誌
fn init_audit_log() {
    // 每日輪轉，保留 30 天
    let file_appender = rolling::daily("./logs/audit", "audit.log");
    let (non_blocking, _guard) = non_blocking(file_appender);

    tracing_subscriber::registry()
        .with(EnvFilter::new("info"))
        .with(
            fmt::layer()
                .json() // JSON 格式
                .with_writer(non_blocking)
                .with_target(true)
                .with_thread_ids(true)
        )
        .init();
}

// 審計事件記錄
#[derive(Serialize)]
struct AuditEvent {
    module_id: String,
    event_type: String,
    details: serde_json::Value,
}

fn log_security_violation(module_id: &str, violation: &str) {
    tracing::warn!(
        target: "audit",
        module_id = module_id,
        event_type = "security_violation",
        violation = violation,
        "Security violation detected"
    );
}
```

**JSON 輸出範例：**
```json
{
  "timestamp": "2025-12-04T10:30:45.123Z",
  "level": "WARN",
  "target": "audit",
  "fields": {
    "message": "Security violation detected",
    "module_id": "k2s",
    "event_type": "security_violation",
    "violation": "unauthorized_domain_access",
    "domain": "malicious.com"
  },
  "thread_id": "12345"
}
```

**日誌輪轉策略：**
- **頻率**：每日輪轉 (daily)
- **保留期限**：30 天
- **檔案命名**：`audit.log.2025-12-04`
- **位置**：`./logs/audit/`
- **大小限制**：單檔最大 100MB (可配置)

**日誌級別分類：**
- **INFO**: 正常操作 (模組載入、URL 解析成功)
- **WARN**: 可疑行為 (頻繁請求、接近限制)
- **ERROR**: 安全違規 (權限拒絕、SSRF 嘗試、沙箱逃逸)

**Sources:**
- [Logging in Rust (2025) | Shuttle](https://www.shuttle.dev/blog/2023/09/20/logging-in-rust)
- [Production-Grade Logging in Rust Applications](https://medium.com/better-programming/production-grade-logging-in-rust-applications-2c7fffd108a6)
- [tracing - Rust](https://docs.rs/tracing)

---

## 6. 架構決策：雙模組系統共存

### 背景

專案中已存在基於 Rust 動態函式庫的模組系統 (`modules/example-downloader`)，使用 `libloading` 載入 `.so/.dll/.dylib` 檔案。然而新規格要求使用 JavaScript 模組。

### 決策：保留 Rust 模組，新增 JavaScript 模組支援

**選擇理由：**
1. **向後相容**：不破壞現有的 Rust 模組架構
2. **靈活性**：允許效能關鍵模組使用 Rust，社群貢獻使用 JavaScript
3. **漸進遷移**：可以逐步將 Rust 模組改寫為 JavaScript
4. **測試覆蓋**：現有的 Rust 模組測試可以驗證介面設計

**統一介面設計：**
```rust
// 統一的模組 trait
pub enum ModuleType {
    Rust(Box<dyn WebsiteModule>),
    JavaScript(JsModule),
}

impl ModuleType {
    pub fn metadata(&self) -> ModuleMetadata {
        match self {
            ModuleType::Rust(module) => module.metadata(),
            ModuleType::JavaScript(module) => module.metadata(),
        }
    }

    pub fn can_handle(&self, url: &str) -> bool {
        match self {
            ModuleType::Rust(module) => module.can_handle(url),
            ModuleType::JavaScript(module) => module.can_handle(url),
        }
    }

    // ... 其他方法
}
```

---

## 7. 額外研究：資源監控

### 決策：使用 tokio::time + custom limiter

**CPU 時間限制：**
```rust
use tokio::time::{timeout, Duration};

async fn execute_with_timeout<F, T>(
    future: F,
    max_duration: Duration,
) -> Result<T, String>
where
    F: Future<Output = T>,
{
    timeout(max_duration, future)
        .await
        .map_err(|_| "Execution timeout".to_string())
}
```

**記憶體監控：**
- 使用 deno_core 的內建 heap statistics
- 定期檢查 `JsRuntime::heap_statistics()`
- 超過限制時強制終止

---

## 8. 實作順序建議

根據依賴關係，建議實作順序：

1. **Phase 1**：基礎設施
   - Manifest 解析 (`manifest.rs`)
   - JavaScript Runtime 管理 (`js_runtime.rs`)
   - 基本模組包裝器 (`js_module.rs`)

2. **Phase 2**：安全層
   - 沙箱環境 (`sandbox.rs`)
   - 權限系統 (`permission.rs`)
   - URL 驗證器 (`url_validator.rs`)

3. **Phase 3**：認證與監控
   - 認證代理 (`credential_proxy.rs`)
   - 資源限制器 (`resource_limiter.rs`)
   - 審計系統 (`audit.rs`)
   - 網路監控 (`network_monitor.rs`)

4. **Phase 4**：整合
   - 擴展 ModuleRegistry 支援 JS 模組
   - 整合到下載命令
   - Tauri commands (`module_management.rs`)

5. **Phase 5**：K2S 模組實作

6. **Phase 6**：前端 UI

7. **Phase 7**：安全測試

---

## 9. 風險與緩解措施

| 風險 | 影響 | 緩解措施 |
|------|------|----------|
| deno_core API 變更 | 高 | 鎖定版本，關注 changelog |
| V8 引擎大小影響應用程式大小 | 中 | 接受權衡，提供文件說明 |
| JavaScript 沙箱逃逸 | 高 | 全面安全測試，bug bounty |
| 跨平台 keyring 差異 | 中 | 完整的跨平台測試 |
| DNS rebinding 攻擊 | 中 | 雙重 IP 驗證 (解析前後) |
| 效能開銷超過 15% | 中 | 效能測試，優化熱路徑 |

---

## 10. 結論

所有技術不確定性已解決。選擇的技術堆疊能夠滿足規格中的所有功能和安全需求：

- ✅ **JavaScript 模組執行**：deno_core + V8
- ✅ **安全認證儲存**：tauri-plugin-keyring
- ✅ **SSRF 防護**：std::net + 自訂驗證
- ✅ **網域匹配**：wildmatch
- ✅ **審計日誌**：tracing + JSON
- ✅ **向後相容**：雙模組系統共存

可以安全地進入 Phase 1 (Design & Contracts) 階段。
