# 在 Windows Docker Container 中運行 Tauri 的指南

## 問題說明

Tauri 需要圖形界面環境（X11 或 Wayland），但 Docker container 預設是無頭（headless）環境。

## 解決方案

### 方案 1：只構建不運行（最簡單）

如果您只需要構建應用：

```bash
# Build frontend
npm run build

# Build Tauri app
npm run tauri build
```

構建完成後，執行檔會在 `src-tauri/target/release/` 目錄中。

---

### 方案 2：使用 X11 轉發（推薦開發使用）

#### 步驟 1：在 Windows 上安裝 X11 服務器

選擇以下其中一個：
- **VcXsrv**（免費）：https://sourceforge.net/projects/vcxsrv/
- **X410**（付費，更穩定）：https://x410.dev/
- **Xming**（免費）：https://sourceforge.net/projects/xming/

#### 步驟 2：啟動 X11 服務器

**使用 VcXsrv：**
1. 執行 `XLaunch`
2. 選擇 "Multiple windows"
3. Display number 設為 `0`
4. Start no client
5. **重要**：勾選 "Disable access control"
6. 完成並啟動

**使用 X410：**
1. 啟動 X410
2. 在系統托盤右鍵點擊 X410 圖標
3. 選擇 "Windowed Apps" 模式

#### 步驟 3：在 Docker Container 中配置環境

```bash
# 使用提供的設置腳本
source ./setup-x11.sh

# 運行 Tauri 開發模式
npm run tauri dev
```

#### 步驟 4：測試 X11 連接

安裝測試工具：
```bash
apt-get update && apt-get install -y x11-apps
```

測試 X11 是否工作：
```bash
xclock
```

如果看到一個時鐘窗口，表示 X11 配置成功。

---

### 方案 3：在 Windows 本地開發（最佳體驗）

如果 X11 轉發有問題，建議直接在 Windows 上開發：

#### 步驟 1：安裝 Rust 和依賴

在 Windows PowerShell 中：
```powershell
# 安裝 Rust
winget install Rustlang.Rustup

# 安裝 WebView2
# (通常 Windows 11 已預裝)
```

#### 步驟 2：在 Windows 上運行

```powershell
# 在專案目錄中
npm install
npm run tauri dev
```

---

## 常見問題排除

### 問題：`Failed to initialize GTK`

**解決方法：**
```bash
# 確認 DISPLAY 變數
echo $DISPLAY

# 測試 X11 連接
apt-get install -y x11-apps
xclock
```

### 問題：`Connection refused`

**解決方法：**
1. 確認 Windows 防火牆允許 X11 服務器的連接（TCP port 6000）
2. 確認 X11 服務器已啟動且設置為 "Disable access control"
3. 檢查 Docker 網路設定

### 問題：`Locale not supported`

**解決方法：**
```bash
export LC_ALL=C.UTF-8
export LANG=C.UTF-8
```

---

## Docker Compose 配置範例

如果使用 Docker Compose，可以這樣配置：

```yaml
version: '3.8'
services:
  tauri-dev:
    build: .
    environment:
      - DISPLAY=host.docker.internal:0.0
      - LC_ALL=C.UTF-8
      - LANG=C.UTF-8
      - NO_AT_BRIDGE=1
    volumes:
      - .:/app
      - /tmp/.X11-unix:/tmp/.X11-unix
    network_mode: host
```

---

## 推薦方案總結

| 方案 | 優點 | 缺點 | 適用場景 |
|------|------|------|----------|
| 方案 1（只構建） | 最簡單，無需配置 | 無法即時預覽 | CI/CD、正式構建 |
| 方案 2（X11 轉發） | 可在 Docker 開發 | 配置複雜，可能有延遲 | 需要保持 Docker 環境 |
| 方案 3（Windows 本地） | 最佳效能，原生體驗 | 需要在 Windows 安裝工具 | 日常開發 |

**建議：** 日常開發使用方案 3（Windows 本地），CI/CD 使用方案 1（只構建）。
