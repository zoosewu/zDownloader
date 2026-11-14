# Zoo Downloader 日誌系統說明

## 日誌位置

Zoo Downloader 會將所有日誌記錄到以下位置：

### Windows
```
%APPDATA%\zoo-downloader\logs\zoo-downloader.log
```
完整路徑範例：
```
C:\Users\您的用戶名\AppData\Roaming\zoo-downloader\logs\zoo-downloader.log
```

### macOS
```
~/Library/Logs/zoo-downloader/zoo-downloader.log
```

### Linux
```
~/.local/share/zoo-downloader/logs/zoo-downloader.log
```

## 快速存取方式

### Windows
1. 按 `Win + R` 開啟執行對話框
2. 輸入 `%APPDATA%\zoo-downloader\logs`
3. 按 Enter

### macOS
1. 開啟 Finder
2. 按 `Cmd + Shift + G`
3. 輸入 `~/Library/Logs/zoo-downloader`
4. 按 Enter

### Linux
```bash
cd ~/.local/share/zoo-downloader/logs
tail -f zoo-downloader.log  # 即時查看日誌
```

## 日誌內容

日誌會記錄以下資訊：

### 前端（瀏覽器控制台）
- Tauri API 初始化狀態
- API 呼叫失敗的錯誤（參數不記錄，可能含帳密）
- 表單驗證過程
- 錯誤堆疊追蹤

### 後端（日誌檔案）
- 應用程式啟動訊息
- 命令接收和執行狀態
- URL 驗證結果
- 檔案系統操作（路徑驗證、權限檢查）
- 下載任務建立和狀態變更
- HTTP 伺服器啟動狀態
- 錯誤詳情

## 日誌級別

目前設定為 `INFO` 級別，會記錄：
- ✓ 成功操作
- ✗ 錯誤訊息
- ⚠ 警告訊息
- 一般資訊

## 除錯建議

如果遇到問題，請執行以下步驟：

1. **重現問題**
   - 開啟應用程式
   - 執行導致問題的操作
   - 注意問題發生的確切時間

2. **收集日誌**
   - 前端日誌：開啟開發者工具（F12），查看 Console 標籤
   - 後端日誌：開啟日誌檔案（參考上方路徑）
   - 複製問題發生前後的所有日誌訊息

3. **提供資訊**
   在回報問題時，請提供：
   - 操作系統版本
   - 問題描述
   - 重現步驟
   - 完整的日誌訊息（前端 + 後端）

## 前端開發者工具

### Windows / Linux
按 `F12` 或 `Ctrl + Shift + I`

### macOS
按 `Cmd + Option + I`

## 日誌檔案管理

日誌檔案會持續增長，建議定期清理：

### 手動清理
刪除或重新命名舊的日誌檔案

### 自動輪換
日誌檔案會自動按日期命名，舊檔案不會被覆蓋

## 常見問題

### Q: 為什麼我看不到日誌？
A:
- 確認應用程式已完全啟動
- 檢查日誌目錄是否存在
- 確認有寫入權限

### Q: 前端控制台沒有日誌？
A:
- 確認開啟了開發者工具
- 檢查 Console 過濾器設定
- 確認沒有清空控制台

### Q: 日誌檔案太大怎麼辦？
A:
- 可以安全地刪除舊日誌檔案
- 應用程式會自動建立新的日誌檔案

## 隱私注意事項

日誌檔案可能包含：
- 下載的 URL
- 檔案路徑
- 錯誤訊息

在分享日誌時，請檢查並移除任何敏感資訊。
