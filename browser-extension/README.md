# Zoo Downloader 瀏覽器擴充

Chrome 與 Edge（Manifest V3）。設計與協定見 [docs/architecture/browser-extension.md](../docs/architecture/browser-extension.md)。

- 右鍵「用 Zoo Downloader 下載」：連結、圖片、影片、音訊。附上網站的 Cookie、Referer 與 User-Agent。
- 接管瀏覽器下載（popup 開關，預設開）：app 接手的下載會從瀏覽器取消並移出下載紀錄；app 沒回應就留給瀏覽器。無痕視窗、`blob:`／`data:`／`file:` 網址與其他擴充發起的下載不接管。

## 安裝（開發人員模式）

1. 安裝並開啟一次 Zoo Downloader：app 啟動時會向 Chrome、Edge 登記 `zoo-native-host`。
2. 開啟 `chrome://extensions`（Edge：`edge://extensions`），打開「開發人員模式」。
3. 「載入未封裝項目」，選這個資料夾。

`manifest.json` 的 `key` 固定了擴充 ID（`oibblcgobgdenncecbfgdcpeegliejcj`），app 只接受這個 ID。不要刪掉或更換 `key`；若要更換，同步改 `src-tauri/crates/zoo-native-host/src/lib.rs` 的 `EXTENSION_ID`（測試會檢查兩者一致）。

## 結構

- `background.js`：service worker，只負責把瀏覽器事件接到 `src/core.js`。
- `src/core.js`：所有邏輯；`chrome` 以參數傳入，測試用假物件。
- `popup.*`：連線狀態與「接管瀏覽器下載」開關。
- `_locales/`：英文、繁體中文、日文。

測試：在專案根目錄執行 `pnpm test:run`（`test/core.test.js`）。
