# Docker 開發環境快速入門

## 🚀 快速開始

### 1. 啟動 Tauri 後端（在 Docker 容器內）

```bash
# 設置環境變數啟用 HTTP Bridge
export TAURI_HTTP_BRIDGE=true

# 啟動 Tauri 應用
cd /nodejs/zoo-downloader/src-tauri
cargo run
```

你應該會看到：

```
[Tauri] HTTP Bridge enabled for development
[HTTP Bridge] Starting on port 3030
[HTTP Bridge] Server ready at http://0.0.0.0:3030
[HTTP Bridge] Endpoint: POST /invoke
[HTTP Bridge] Health check: GET /health
```

### 2. 啟動前端開發伺服器（在 Docker 容器內或本機）

開啟新的終端：

```bash
cd /nodejs/zoo-downloader

# 使用開發模式啟動
npm run dev -- --mode development
```

### 3. 訪問應用

在**本機瀏覽器**打開：`http://localhost:5173`

## 🔍 驗證設置

### 檢查 HTTP Bridge 是否運行

```bash
curl http://localhost:3030/health
```

應該返回：
```json
{"status":"ok"}
```

### 測試 invoke endpoint

```bash
curl -X POST http://localhost:3030/invoke \
  -H "Content-Type: application/json" \
  -d '{"cmd":"greet","args":{"name":"Docker"}}'
```

應該返回：
```json
{
  "success": true,
  "data": "Hello, Docker! You've been greeted from Rust!",
  "error": null
}
```

### 檢查前端模式

打開瀏覽器 Console（F12），應該看到：

```
[Tauri] Development mode - checking HTTP bridge at http://localhost:3030
[Tauri] HTTP Bridge is ready
[useClipboard] Tauri ready: true
[useClipboard] Clipboard monitoring enabled: true
[useClipboard] Starting clipboard polling with 400ms interval
```

## 📝 環境變數

確保 `.env.development` 檔案存在且包含：

```bash
VITE_DEV_MODE=true
VITE_TAURI_HTTP_URL=http://localhost:3030
```

## 🐛 故障排除

### 問題：前端無法連接到後端

**檢查 1**: HTTP Bridge 是否啟動？
```bash
curl http://localhost:3030/health
```

**檢查 2**: Docker port 是否正確 mapping？
```bash
# 如果使用 docker-compose，確保有：
ports:
  - "127.0.0.1:3030:3030"
  - "5173:5173"
```

**檢查 3**: 環境變數是否設置？
```bash
echo $TAURI_HTTP_BRIDGE  # 應該顯示 "true"
```

### 問題：CORS 錯誤

HTTP Bridge 已經配置允許所有 origin。如果仍有問題，檢查瀏覽器 Console 的完整錯誤訊息。

### 問題：剪貼簿監控不工作

開發模式下，某些功能（如剪貼簿監控）可能需要完整的 Tauri 環境。HTTP Bridge 提供基本的 `get_clipboard_content` 支援，但可能在純瀏覽器環境中受限。

## 🎯 支援的功能（開發模式）

✅ **完全支援**：
- `greet` - 測試用
- `get_clipboard_content` - 取得剪貼簿（需要 Docker 環境支援）
- `get_settings` - 返回默認設定
- `get_download_list` - 返回空列表

⚠️ **部分支援**（返回模擬數據）：
- `create_download_with_path`
- `start_download`
- `pause_download`
- `cancel_download`

❌ **不支援**（需要完整 Tauri 環境）：
- 系統托盤功能
- 檔案對話框
- 某些原生 API

## 📦 生產環境建置

當準備建置生產版本時，**不要**設置 `TAURI_HTTP_BRIDGE` 環境變數：

```bash
# 清除開發環境變數
unset TAURI_HTTP_BRIDGE

# 建置前端（生產模式）
npm run build

# 建置 Tauri 應用
npm run tauri build
```

生產版本會使用原生 Tauri IPC，不會啟動 HTTP Bridge。

## 💡 提示

1. **開發效率**：前端修改會自動熱重載，但後端修改需要重新 `cargo run`

2. **日誌查看**：兩個終端分別顯示前端和後端日誌，方便除錯

3. **環境切換**：
   - 開發模式：`npm run dev -- --mode development`
   - 生產模式預覽：`npm run dev -- --mode production`（需要 Tauri 桌面環境）

4. **Port 衝突**：如果 3030 port 被佔用，修改：
   - `src-tauri/src/lib.rs` 的 port 參數
   - `.env.development` 的 `VITE_TAURI_HTTP_URL`
