# 模組系統使用指南

## 📦 模組載入

### 1. 模組目錄結構

```
zoo-downloader/
├── modules/                    # 模組根目錄
│   ├── k2s/                   # Keep2Share 模組
│   │   ├── manifest.json      # 模組配置
│   │   ├── index.js          # 模組程式碼
│   │   └── README.md         # 說明文件
│   ├── rapidgator/           # 其他網站模組...
│   └── _template/            # 模組範本（會被忽略）
└── src-tauri/
    └── src/modules/          # Rust 模組系統
```

### 2. 自動載入（啟動時）

#### 方法 A：在 Tauri 啟動時載入

在 `src-tauri/src/main.rs` 或 `src-tauri/src/lib.rs` 中：

```rust
use zoo_downloader_lib::modules::registry::ModuleRegistry;
use std::sync::{Arc, Mutex};

fn main() {
    // 創建模組註冊表
    let mut registry = ModuleRegistry::new();

    // 掃描並載入所有模組
    let modules_dir = std::path::PathBuf::from("./modules");
    let loaded_count = registry.scan_and_register(&modules_dir);

    println!("✅ 載入了 {} 個模組", loaded_count);

    // 包裝成 Arc<Mutex> 供 Tauri 使用
    let registry = Arc::new(Mutex::new(registry));

    tauri::Builder::default()
        .manage(registry)  // 註冊為全域狀態
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
```

### 3. 手動載入特定模組

```rust
use zoo_downloader_lib::modules::loader::ModuleLoader;
use zoo_downloader_lib::modules::registry::ModuleRegistry;

// 創建載入器
let loader = ModuleLoader::new("./modules");

// 載入單一模組
let k2s_module = loader.load_module("./modules/k2s")?;
println!("載入模組: {} v{}",
    k2s_module.manifest.name,
    k2s_module.manifest.version
);

// 註冊到註冊表
let mut registry = ModuleRegistry::new();
registry.register_js_module(k2s_module)?;
```

### 4. 查詢已載入的模組

#### 取得所有模組列表

```rust
// 在 Tauri command 中
#[tauri::command]
async fn list_modules(
    registry: State<'_, Arc<Mutex<ModuleRegistry>>>
) -> Result<Vec<String>, String> {
    let registry = registry.lock().unwrap();
    let modules = registry.list_all_modules();

    Ok(modules.iter()
        .map(|m| format!("{} ({})", m.name, m.id))
        .collect())
}
```

#### 檢查 URL 是否被支援

```rust
#[tauri::command]
async fn check_url_support(
    url: String,
    registry: State<'_, Arc<Mutex<ModuleRegistry>>>
) -> Result<bool, String> {
    let registry = registry.lock().unwrap();
    Ok(registry.find_module_for_url(&url).is_some())
}
```

---

## 🐛 模組除錯

### 1. 日誌輸出

#### 在模組中使用 Context API

```javascript
// modules/k2s/index.js
class K2SModule {
    async extractDownloadInfo(url, context) {
        // 使用 context.log 輸出除錯訊息
        context.log.debug(`開始處理 URL: ${url}`);

        try {
            const fileId = this.parseFileId(url);
            context.log.info(`解析到檔案 ID: ${fileId}`);

            // 認證
            context.log.info('正在進行認證...');
            await this.authenticate(context);
            context.log.info('認證成功');

            // ... 其他處理

        } catch (error) {
            context.log.error(`處理失敗: ${error.message}`);
            throw error;
        }
    }
}
```

#### 在 Rust 端查看日誌

設定 `RUST_LOG` 環境變數：

```bash
# 開發模式 - 顯示所有日誌
RUST_LOG=debug cargo run

# 只顯示模組相關日誌
RUST_LOG=zoo_downloader::modules=debug cargo run

# 在 Windows PowerShell
$env:RUST_LOG="debug"; cargo run
```

### 2. 模組載入除錯

#### 檢查模組是否正確載入

```rust
// 在程式啟動時添加除錯輸出
let loader = ModuleLoader::new("./modules");
let modules = loader.load_all_modules();

for module in &modules {
    println!("📦 載入模組:");
    println!("  ID: {}", module.id);
    println!("  名稱: {}", module.manifest.name);
    println!("  版本: {}", module.manifest.version);
    println!("  狀態: {:?}", module.status());
    println!("  URL 模式: {:?}", module.manifest.url_patterns);
}
```

#### 檢查 manifest.json 解析

```bash
# 驗證 JSON 格式
cd modules/k2s
cat manifest.json | jq .

# 檢查必要欄位
cat manifest.json | jq '{id, name, version, urlPatterns}'
```

### 3. 執行時除錯

#### 方法 A：使用 Rust 測試

創建測試檔案 `src-tauri/src/modules/tests/debug_module.rs`：

```rust
#[cfg(test)]
mod debug_tests {
    use crate::modules::loader::ModuleLoader;
    use crate::modules::context_api::ModuleContext;

    #[test]
    fn test_k2s_module_loading() {
        let loader = ModuleLoader::new("../modules");
        let result = loader.load_module("../modules/k2s");

        match result {
            Ok(module) => {
                println!("✅ 模組載入成功");
                println!("  ID: {}", module.id);
                println!("  程式碼長度: {} bytes", module.code.len());
            }
            Err(e) => {
                println!("❌ 模組載入失敗: {}", e);
                panic!("模組載入測試失敗");
            }
        }
    }

    #[tokio::test]
    async fn test_k2s_url_detection() {
        let loader = ModuleLoader::new("../modules");
        let module = loader.load_module("../modules/k2s")
            .expect("無法載入 K2S 模組");

        // 測試 URL 檢測（這需要執行 JS 程式碼）
        let test_url = "https://k2s.cc/file/abc123";
        println!("測試 URL: {}", test_url);

        // TODO: 實際執行 canHandle() 需要 deno_core runtime
    }
}
```

執行測試：

```bash
cd src-tauri
cargo test debug_tests -- --nocapture
```

#### 方法 B：使用 Node.js 測試模組邏輯

創建 `modules/k2s/test.mjs`：

```javascript
// 直接測試模組的 JavaScript 邏輯
import { K2SModule } from './index.js';

// 模擬 Context API
const mockContext = {
    moduleId: 'k2s',
    http: {
        get: async (url, headers) => {
            console.log('Mock HTTP GET:', url);
            return JSON.stringify({ status: 'success' });
        },
        post: async (url, headers, body) => {
            console.log('Mock HTTP POST:', url);
            return JSON.stringify({
                status: 'success',
                auth_token: 'mock_token_123'
            });
        }
    },
    credentials: {
        has: (key) => {
            console.log('Check credential:', key);
            return true;
        },
        get: (key) => '[HIDDEN]'
    },
    log: {
        info: (msg) => console.log('ℹ️', msg),
        warn: (msg) => console.warn('⚠️', msg),
        error: (msg) => console.error('❌', msg),
        debug: (msg) => console.log('🐛', msg)
    },
    captcha: {
        request: async (url) => {
            console.log('CAPTCHA requested:', url);
            return 'mock_captcha_solution';
        }
    }
};

// 測試模組
async function testModule() {
    const module = new K2SModule();

    console.log('=== 測試 canHandle ===');
    console.log('k2s.cc URL:', module.canHandle('https://k2s.cc/file/abc123'));
    console.log('other URL:', module.canHandle('https://example.com/file'));

    console.log('\n=== 測試 parseFileId ===');
    try {
        const fileId = module.parseFileId('https://k2s.cc/file/abc123def456/test.zip');
        console.log('File ID:', fileId);
    } catch (error) {
        console.error('Error:', error.message);
    }

    console.log('\n=== 測試認證流程 ===');
    try {
        await module.authenticate(mockContext);
        console.log('認證成功，session:', module.session);
    } catch (error) {
        console.error('認證失敗:', error.message);
    }
}

testModule().catch(console.error);
```

執行測試：

```bash
cd modules/k2s
node test.mjs
```

### 4. 網路請求除錯

#### 監控模組的 HTTP 請求

在 `context_api.rs` 的 `handle_http_request` 中添加詳細日誌：

```rust
pub async fn handle_http_request(
    &mut self,
    method: &str,
    url: &str,
    headers: Option<HashMap<String, String>>,
    body: Option<String>,
) -> Result<String, String> {
    // 添加請求日誌
    log::info!("📡 模組 HTTP 請求:");
    log::info!("  方法: {}", method);
    log::info!("  URL: {}", url);
    log::info!("  Headers: {:?}", headers);
    if let Some(ref b) = body {
        log::info!("  Body (前100字): {}", &b[..b.len().min(100)]);
    }

    // Step 1: Validate URL
    self.url_validator.validate(url)?;

    // ... 其他處理

    // 添加回應日誌
    log::info!("✅ HTTP 請求成功，回應長度: {} bytes", response.len());

    Ok(response)
}
```

### 5. 斷點除錯

#### 使用 VS Code 除錯 Rust 程式碼

創建 `.vscode/launch.json`：

```json
{
    "version": "0.2.0",
    "configurations": [
        {
            "type": "lldb",
            "request": "launch",
            "name": "Debug Tauri App",
            "cargo": {
                "args": [
                    "build",
                    "--manifest-path=./src-tauri/Cargo.toml",
                    "--no-default-features"
                ]
            },
            "args": [],
            "cwd": "${workspaceFolder}"
        },
        {
            "type": "lldb",
            "request": "launch",
            "name": "Debug Module Tests",
            "cargo": {
                "args": [
                    "test",
                    "--lib",
                    "--manifest-path=./src-tauri/Cargo.toml",
                    "--",
                    "--nocapture"
                ],
                "filter": {
                    "name": "k2s_tests",
                    "kind": "test"
                }
            }
        }
    ]
}
```

在程式碼中設定斷點，然後按 F5 開始除錯。

---

## 🔍 常見問題排查

### 問題 1：模組無法載入

**症狀**：`load_module()` 返回錯誤

**排查步驟**：

```bash
# 1. 檢查檔案是否存在
ls -la modules/k2s/

# 2. 檢查 manifest.json 格式
cat modules/k2s/manifest.json | jq .

# 3. 檢查 index.js 是否為空
wc -l modules/k2s/index.js

# 4. 執行載入測試
cd src-tauri
RUST_LOG=debug cargo test test_k2s_module_loading -- --nocapture
```

### 問題 2：模組載入但無法執行

**症狀**：模組載入成功但 `extractDownloadInfo()` 失敗

**可能原因**：
- JS 語法錯誤
- 缺少必要的依賴
- Context API 未正確連接

**排查**：

```javascript
// 在模組開頭添加自檢程式碼
class K2SModule {
    constructor() {
        console.log('✅ K2S 模組建構子執行');
        this.selfCheck();
    }

    selfCheck() {
        const required = ['session', 'accountType', 'apiBaseUrl'];
        for (const prop of required) {
            if (!(prop in this)) {
                console.error(`❌ 缺少屬性: ${prop}`);
            }
        }
        console.log('✅ 自檢完成');
    }
}
```

### 問題 3：Context API 調用失敗

**症狀**：`context.http.get()` 等方法拋出錯誤

**原因**：deno_core ops 尚未完全註冊

**臨時解決方案**：

目前 Context API 的 handler 邏輯已實作，但 deno_core ops 註冊需要額外整合。可以：

1. **直接從 Rust 測試 handler**：

```rust
#[tokio::test]
async fn test_context_api_handlers() {
    let mut context = ModuleContext::new("test-module".to_string());

    // 添加測試認證
    context.credential_proxy_mut()
        .insert_credential_for_test(
            "username".to_string(),
            "test_user".to_string()
        );

    // 測試 HTTP 請求 handler
    let result = context.handle_http_request(
        "GET",
        "https://httpbin.org/get",
        None,
        None
    ).await;

    match result {
        Ok(response) => println!("✅ HTTP 請求成功: {}", response),
        Err(e) => println!("❌ HTTP 請求失敗: {}", e),
    }
}
```

2. **使用 mock Context**（如前述 Node.js 測試）

---

## 📝 除錯檢查清單

### 模組開發時

- [ ] manifest.json 格式正確（使用 `jq` 驗證）
- [ ] index.js 語法正確（使用 Node.js 測試）
- [ ] 導出的類別名稱正確
- [ ] 實作了必要的方法（`canHandle`, `extractDownloadInfo`）
- [ ] 使用 Context API 而非直接網路請求
- [ ] 添加了充足的日誌輸出

### 整合測試時

- [ ] 模組目錄在正確位置
- [ ] ModuleLoader 指向正確的目錄
- [ ] ModuleRegistry 已正確初始化
- [ ] URL 模式匹配正確
- [ ] 權限配置正確

### 執行時

- [ ] RUST_LOG 環境變數已設定
- [ ] 查看 Tauri 開發者工具的 Console
- [ ] 檢查 Rust 端的日誌輸出
- [ ] 驗證網路請求是否被正確攔截

---

## 🚀 快速除錯命令

```bash
# 編譯並顯示詳細日誌
RUST_LOG=debug cargo run

# 只執行模組測試
cargo test k2s_tests -- --nocapture

# 驗證 JSON 配置
jq . modules/k2s/manifest.json

# 測試 JavaScript 邏輯
node modules/k2s/test.mjs

# 檢查模組檔案
tree modules/k2s/
```

---

**提示**：建議在開發新模組時，先使用 Node.js 測試 JavaScript 邏輯，確認無誤後再整合到 Rust 環境中進行完整測試。
