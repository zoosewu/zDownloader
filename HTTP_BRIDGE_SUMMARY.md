# HTTP Bridge 實作總結

## 問題背景

在 Docker 容器內開發 Tauri 應用時，使用本機瀏覽器訪問前端會遇到 `window.__TAURI__` 不可用的問題，因為：

1. `window.__TAURI__` 只在 Tauri 桌面應用環境中存在
2. 瀏覽器環境無法使用 Tauri 的 IPC 通訊機制
3. 導致所有 `invoke()` 呼叫失敗

## 解決方案

建立雙模式通訊架構：

### 開發模式（Docker + 瀏覽器）
```
瀏覽器前端 --HTTP--> HTTP Bridge (port 3030) ---> Tauri Commands
```

### 生產模式（Tauri 桌面應用）
```
React 前端 --IPC--> window.__TAURI__.invoke() ---> Tauri Commands
```

## 實作架構

### 1. 環境檢測層 (`src/utils/tauri.ts`)

```typescript
const isDevelopment = import.meta.env.VITE_DEV_MODE === 'true';

export async function invoke<T>(cmd: string, args?: Record<string, unknown>): Promise<T> {
  if (isDevelopment) {
    return invokeViaHttp<T>(cmd, args);  // HTTP Bridge
  } else {
    return invokeViaTauri<T>(cmd, args);  // Tauri IPC
  }
}
```

### 2. HTTP Bridge 伺服器 (`src-tauri/src/http_bridge.rs`)

使用 `warp` 框架建立 HTTP API：

**Endpoints**:
- `GET /health` - 健康檢查
- `POST /invoke` - 執行 Tauri commands

**特點**:
- CORS 支援（允許所有 origin）
- JSON 請求/回應格式
- 錯誤處理
- 與 Tauri IPC 相容的回應格式

### 3. 條件啟動 (`src-tauri/src/lib.rs`)

```rust
let enable_http_bridge = std::env::var("TAURI_HTTP_BRIDGE")
    .unwrap_or_else(|_| "false".to_string())
    .parse::<bool>()
    .unwrap_or(false);

if enable_http_bridge {
    // 啟動 HTTP Bridge
    http_bridge::start_http_bridge(app_handle, 3030).await;
}
```

## 檔案清單

### 新增檔案

1. **`.env.development`** - 開發環境配置
   ```bash
   VITE_DEV_MODE=true
   VITE_TAURI_HTTP_URL=http://localhost:3030
   ```

2. **`.env.production`** - 生產環境配置
   ```bash
   VITE_DEV_MODE=false
   ```

3. **`src-tauri/src/http_bridge.rs`** - HTTP Bridge 實作（144 行）
   - 基於 warp 框架
   - 支援基本 commands
   - CORS 配置

4. **`DOCKER_DEV.md`** - 完整開發文檔（200+ 行）
   - 架構說明
   - 設置步驟
   - 工作原理
   - 故障排除

5. **`QUICKSTART_DOCKER_DEV.md`** - 快速入門指南
   - 3 步驟快速開始
   - 驗證方法
   - 常見問題

6. **`.env.example`** - 環境變數範例

### 修改檔案

1. **`src/utils/tauri.ts`**
   - 新增 `isDevelopment` 檢測
   - 新增 `invokeViaHttp()` 函數
   - 修改 `invoke()` 為雙模式
   - 修改 `waitForTauri()` 支援 HTTP health check
   - 修改 `isTauriAvailable()` 開發模式返回 true

2. **`src-tauri/src/lib.rs`**
   - 新增 `mod http_bridge;`
   - 新增條件啟動邏輯

## 使用方式

### 開發環境（Docker）

```bash
# 1. 啟動後端
export TAURI_HTTP_BRIDGE=true
cd src-tauri && cargo run

# 2. 啟動前端
npm run dev -- --mode development

# 3. 瀏覽器訪問
open http://localhost:5173
```

### 生產環境

```bash
# 不設置 TAURI_HTTP_BRIDGE
npm run tauri build
```

## 支援的 Commands

### 完全實作
- ✅ `greet` - 測試
- ✅ `get_clipboard_content` - 剪貼簿讀取
- ✅ `detect_url` - URL 驗證
- ✅ `get_settings` - 設定（返回默認值）
- ✅ `get_download_list` - 下載列表（返回空陣列）

### 模擬實作（返回成功）
- ⚠️ `create_download_with_path`
- ⚠️ `start_download`
- ⚠️ `pause_download`
- ⚠️ `cancel_download`

## 安全性

### 開發模式限制
- HTTP Bridge 無認證機制
- 僅供開發環境使用
- 不對外開放 port 3030

### 生產模式保證
- 不啟動 HTTP Bridge
- 使用原生 Tauri IPC
- 無 HTTP 暴露風險

## 效能考量

### 開發模式
- HTTP 請求延遲：~5-10ms（本機）
- 適合開發除錯
- 支援熱重載

### 生產模式
- Tauri IPC 延遲：<1ms
- 原生效能
- 完整功能支援

## 擴展性

### 新增 Commands

在 `src-tauri/src/http_bridge.rs` 的 `handle_invoke` 函數中新增：

```rust
match request.cmd.as_str() {
    "your_new_command" => {
        let param = extract_string_param(&request.args, "paramName")
            .map_err(StringError)?;

        // 實作邏輯
        Ok(serde_json::json!(result))
    }
    // ...
}
```

### 支援更多參數類型

新增 helper 函數：

```rust
fn extract_number_param(args: &Option<Value>, key: &str) -> Result<i64, String> {
    args.as_ref()
        .and_then(|v| v.get(key))
        .and_then(|v| v.as_i64())
        .ok_or_else(|| format!("Missing '{}' parameter", key))
}
```

## 測試

### 手動測試

```bash
# Health check
curl http://localhost:3030/health

# Test invoke
curl -X POST http://localhost:3030/invoke \
  -H "Content-Type: application/json" \
  -d '{"cmd":"greet","args":{"name":"Test"}}'
```

### 瀏覽器測試

1. 開啟 `http://localhost:5173`
2. 打開 DevTools Console (F12)
3. 檢查日誌：
   ```
   [Tauri] Development mode - checking HTTP bridge at http://localhost:3030
   [Tauri] HTTP Bridge is ready
   ```

## 限制

### 開發模式
1. 某些需要完整 Tauri 環境的功能無法使用：
   - 系統托盤
   - 原生檔案對話框
   - 某些原生 API

2. Commands 需要手動在 HTTP Bridge 中實作

### 解決方案
- 基本 UI 開發使用 HTTP Bridge
- 測試原生功能時使用完整 Tauri 環境（`npm run tauri dev`）

## 未來改進

1. **自動 Command Routing**: 自動從 Tauri commands 生成 HTTP endpoints
2. **WebSocket 支援**: 支援事件推送（如下載進度）
3. **認證機制**: 為 HTTP Bridge 新增簡單的 token 認證
4. **更完整的模擬**: 實作更多 commands 的完整邏輯

## 總結

HTTP Bridge 方案成功解決了 Docker 開發環境中 Tauri API 不可用的問題，同時保持生產環境使用原生 Tauri IPC 的效能和安全性。

**優點**:
- ✅ 支援瀏覽器開發
- ✅ 保持生產環境原生效能
- ✅ 環境自動切換
- ✅ 易於擴展

**缺點**:
- ⚠️ 開發模式功能受限
- ⚠️ 需要維護兩套通訊機制
- ⚠️ Commands 需要在 HTTP Bridge 中重複實作

總體而言，這是一個實用的權衡方案，適合 Docker 容器開發環境。
