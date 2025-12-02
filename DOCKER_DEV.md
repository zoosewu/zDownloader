# Docker 開發環境設置

本文檔說明如何在 Docker 容器內開發 Tauri 應用，並通過本機瀏覽器訪問前端。

## 架構說明

### 生產環境
```
前端 (React) + 後端 (Rust Tauri)
     ↓
單一桌面應用程式
     ↓
使用 window.__TAURI__.invoke() 進行 IPC 通訊
```

### 開發環境（Docker）
```
本機瀏覽器 ←→ HTTP Bridge (port 3030) ←→ Docker 容器內的 Tauri 後端
   (前端)                                      (Rust + Tauri commands)
```

## 設置步驟

### 1. 設置環境變數

開發環境使用 `.env.development`：

```bash
# .env.development
VITE_DEV_MODE=true
VITE_TAURI_HTTP_URL=http://localhost:3030
```

生產環境使用 `.env.production`：

```bash
# .env.production
VITE_DEV_MODE=false
```

### 2. 啟動 Tauri 後端（在 Docker 容器內）

設置環境變數啟用 HTTP Bridge：

```bash
export TAURI_HTTP_BRIDGE=true
cargo run
```

或者在 Docker Compose 中：

```yaml
# docker-compose.yml
services:
  zoo-downloader:
    build: .
    ports:
      - "3030:3030"  # HTTP Bridge port
      - "5173:5173"  # Vite dev server port
    environment:
      - TAURI_HTTP_BRIDGE=true
    volumes:
      - .:/app
```

### 3. 啟動前端開發伺服器

```bash
# 開發模式（使用 HTTP Bridge）
npm run dev

# 或明確指定模式
npm run dev -- --mode development
```

### 4. 訪問應用

在本機瀏覽器打開：`http://localhost:5173`

前端會通過 HTTP Bridge (`http://localhost:3030`) 與 Docker 容器內的 Tauri 後端通訊。

## 工作原理

### 前端（src/utils/tauri.ts）

```typescript
// 根據環境變數自動切換通訊方式
const isDevelopment = import.meta.env.VITE_DEV_MODE === 'true';

export async function invoke<T>(cmd: string, args?: Record<string, unknown>): Promise<T> {
  if (isDevelopment) {
    // 開發模式：使用 HTTP Bridge
    return invokeViaHttp<T>(cmd, args);
  } else {
    // 生產模式：使用 Tauri IPC
    return invokeViaTauri<T>(cmd, args);
  }
}
```

### 後端（src-tauri/src/http_bridge.rs）

HTTP Bridge 提供 REST API endpoints：

- `GET /health` - 健康檢查
- `POST /invoke` - 執行 Tauri command

請求格式：
```json
{
  "cmd": "greet",
  "args": {
    "name": "World"
  }
}
```

回應格式：
```json
{
  "success": true,
  "data": "Hello, World!",
  "error": null
}
```

## 支援的 Commands

HTTP Bridge 目前支援以下 commands：

- `greet` - 測試用
- `get_clipboard_content` - 取得剪貼簿內容
- `detect_url` - URL 檢測
- `create_download_with_path` - 建立下載任務
- `start_download` - 開始下載
- `get_download_list` - 取得下載列表
- `get_settings` - 取得設定

### 新增更多 Commands

在 `src-tauri/src/http_bridge.rs` 的 `handle_invoke` 函數中新增：

```rust
match request.cmd.as_str() {
    "your_command" => {
        // 提取參數
        let param = request.args
            .as_ref()
            .and_then(|v| v.get("paramName"))
            .and_then(|v| v.as_str())
            .ok_or_else(|| "Missing parameter".to_string())?;

        // 呼叫 command handler
        match commands::your_module::your_command(param.to_string(), state).await {
            Ok(result) => Ok(serde_json::to_value(result).unwrap()),
            Err(e) => Err(e),
        }
    }
    // ... 其他 commands
}
```

## 除錯

### 檢查 HTTP Bridge 是否運行

```bash
curl http://localhost:3030/health
# 應該返回：{"status":"ok"}
```

### 測試 invoke endpoint

```bash
curl -X POST http://localhost:3030/invoke \
  -H "Content-Type: application/json" \
  -d '{"cmd":"greet","args":{"name":"Test"}}'

# 應該返回：
# {"success":true,"data":"Hello, Test!","error":null}
```

### 瀏覽器 Console 日誌

開發模式下，Console 會顯示：

```
[Tauri] Development mode - checking HTTP bridge at http://localhost:3030
[Tauri] HTTP Bridge is ready
[HTTP Bridge] Invoking 'greet' via http://localhost:3030/invoke
[HTTP Bridge] Command 'greet' succeeded
```

生產模式下，Console 會顯示：

```
[Tauri] Waiting for API... (1/50)
[Tauri] API is ready. Available methods: {hasDirectInvoke: true, ...}
[Tauri] Invoking 'greet' via __TAURI__.invoke
```

## 生產環境建置

生產環境不使用 HTTP Bridge，使用標準 Tauri 建置：

```bash
# 建置前端（生產模式）
npm run build

# 建置 Tauri 應用（不設置 TAURI_HTTP_BRIDGE）
cd src-tauri
cargo build --release

# 或使用 Tauri CLI
npm run tauri build
```

生產環境的 binary 中 HTTP Bridge 不會啟動，使用原生 Tauri IPC 通訊。

## 安全注意事項

⚠️ **HTTP Bridge 僅供開發使用**

- 不要在生產環境啟用 `TAURI_HTTP_BRIDGE`
- HTTP Bridge 沒有認證機制
- 僅在受信任的開發環境中使用
- 確保 port 3030 不對外開放

## 故障排除

### 問題：CORS 錯誤

確保 `src-tauri/src/http_bridge.rs` 中的 CORS 設定允許你的前端 origin。

### 問題：連接被拒絕

1. 檢查 HTTP Bridge 是否啟動：`curl http://localhost:3030/health`
2. 確認 `TAURI_HTTP_BRIDGE=true` 環境變數已設置
3. 檢查 Docker port mapping 是否正確：`-p 3030:3030`

### 問題：前端仍嘗試使用 window.__TAURI__

檢查 `.env.development` 中 `VITE_DEV_MODE=true` 是否正確設置。

Vite 重新啟動後才會讀取新的環境變數：
```bash
# 停止 Vite
Ctrl+C

# 重新啟動
npm run dev
```
