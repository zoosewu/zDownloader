# 瀏覽器擴充重寫規格

- 狀態：已完成（2026-11-15）
- 修正：review D-1（無法載入）、D-2（native messaging 未接線）、D-3（content script 改每個頁面）、SEC-1 剩餘風險（任何擴充都能呼叫本機 HTTP 入口）；CAND-4 的「擴充只用 native messaging」
- 已決定（2026-11-14）：功能為右鍵送出、附帶登入 Cookie、接管瀏覽器下載；支援 Chrome 與 Edge；先以開發人員模式載入，不上架

## 目標

1. 在連結（或圖片、影片）上按右鍵「用 Zoo Downloader 下載」，app 用和瀏覽器相同的 Referer、User-Agent、Cookie 下載，需要登入的檔案也抓得到。
2. 開啟「接管下載」時，瀏覽器開始的下載改由 app 下載；app 沒回應就讓瀏覽器照常下載，不會讓下載消失。
3. 只有我們的擴充能把下載交給 app：刪除本機 HTTP 入口（port 45677），網頁、其他擴充都碰不到。

不做：在網頁裡插入任何東西、掃描頁面上的下載連結、Firefox、上架商店、下載進度回傳給擴充。

## 架構

```
Chrome / Edge
  擴充（MV3 service worker）
    │ chrome.runtime.sendNativeMessage("com.zoo.downloader", {...})
    ▼ 每則訊息瀏覽器啟動一次 host（stdin/stdout，4 byte 長度 + JSON）
zoo-native-host（新的小執行檔，放在 app 旁邊）
    │ 本機 socket：Windows named pipe / Unix domain socket（只有同一個使用者能連）
    │ 連不上 → 啟動旁邊的 app、最多等 10 秒
    ▼ 一行 JSON 請求 / 一行 JSON 回應
Zoo Downloader（shell 的 browser_bridge 模組）
    └─ Downloads::add（與 downloads_add 同一條路：檔名清理、URL 驗證、預設下載目錄）
```

為什麼 host 是獨立執行檔：瀏覽器每則訊息都會啟動 host 一次，host 只轉送，不載入 Tauri、WebView。用主程式當 host 會在每次右鍵時啟動整個 GUI 執行檔。

為什麼不保留 HTTP：任何本機程式與擴充都能連 127.0.0.1；native messaging 只有 host manifest 的 `allowed_origins` 列出的擴充 ID 能啟動 host。

### 新增與刪除

| 項目 | 內容 |
|---|---|
| `crates/zoo-native-host`（新） | lib：協定型別、socket 名稱、擴充 ID（app 與 host 共用）；bin：native messaging 讀寫、轉送到 app、app 未執行時啟動它。只依賴 `interprocess`、`serde_json` |
| shell `browser_bridge.rs`（新） | 啟動時監聽 socket；處理 `ping` / `add`；註冊 host manifest |
| 刪除 | `native_messaging/`（`http_server.rs`、舊 `mod.rs`）、`main.rs` 的 host 模式、`warp` 依賴、`DISABLE_NATIVE_MESSAGING_SERVER`、`chrome-extension/content.js`、`IMPLEMENTATION.md`、`native-messaging/install-*`；擴充改名資料夾為 `browser-extension/` |

## 協定

擴充 → host（native messaging 訊息，也是 host → app 的一行 JSON，原樣轉送）：

```json
{ "v": 1, "type": "ping" }
{ "v": 1, "type": "add", "url": "https://…", "filename": "a.zip",
  "referer": "https://…", "userAgent": "Mozilla/5.0 …", "cookie": "a=1; b=2" }
```

- `filename`、`referer`、`userAgent`、`cookie` 都可省略。`filename` 只是建議，app 仍會清理（只取 basename、拒絕 `..`）。
- app 把 `referer`、`userAgent`、`cookie` 轉成任務的 `headers`（`Referer`、`User-Agent`、`Cookie`），走 zoo-core 既有的 header 檢查。
- 下載到設定的預設目錄，立即開始（遵守同時下載上限）。

回應：

```json
{ "ok": true, "name": "a.zip" }
{ "ok": false, "code": "invalid" | "appNotRunning" | "unsupportedVersion" | "internal", "message": "…" }
```

- `v` 不是 1 時回 `unsupportedVersion`，擴充與 app 版本不合時看得出原因。
- 回應永遠很小，不會碰到 host → 瀏覽器 1 MB 的上限。

## 安全

- **誰能啟動 host**：host manifest 的 `allowed_origins` 只列我們的擴充 ID（`zoo-native-host` 的 `EXTENSION_IDS`：未封裝載入的版本與 Edge 商店版）。擴充 manifest 放固定的 `key`，未封裝載入時 ID 也固定；商店版的 ID 由商店決定。host 另外檢查瀏覽器傳入的 origin 參數是否為其中之一，不是就拒絕。
- **誰能連 app 的 socket**：
  - 名稱含使用者名稱，避免多使用者電腦互相連到；Unix socket 放在 app 資料夾，app 啟動時把它設成只有本人能進（0700）。不設 socket 檔本身的權限：macOS 不支援（`fchmod` 失敗，0.2.7 以前 bridge 因此在 macOS 從未啟動，擴充永遠回退給瀏覽器）。
  - Windows：拒絕遠端連線（`PIPE_REJECT_REMOTE_CLIENTS`），DACL 只給目前使用者；app 用 first-instance 建立，名稱被搶就記錄錯誤、不聽。
  - host 連上後在 Windows 檢查 pipe 另一端的程序路徑是 app 本身（`GetNamedPipeServerProcessId`，透過 interprocess 的 `peer_creds`），防止別的程式先佔名稱騙取 Cookie。
  - host 啟動 app 前把自己的 std handle 設為不可繼承：否則 app 會一直握著瀏覽器給 host 的 pipe。
- **Cookie**：只在送出當下讀取 `chrome.cookies.getAll({ url })`，交給 app 後存在任務的 headers（下載紀錄明文，和手動輸入 Cookie 相同，UI 已有提示）。不記 log。
- **無痕／InPrivate 視窗**（2026-11-17 使用者決定）：擴充只有在使用者於瀏覽器允許它在 InPrivate 執行時才會在那裡運作，這個瀏覽器開關就是唯一的控制；允許後行為與一般視窗相同（接管、右鍵），Cookie 讀該視窗自己的儲存區（`cookies.getAllCookieStores` 找分頁所屬的 store；下載只有 `incognito` 時用非 "0" 的 store），找不到 InPrivate 的 store 就不附 Cookie，絕不退回讀一般視窗的。app 不分辨私密與否：這些下載照常留在 app 的下載紀錄（隱私權政策已說明）。

## 擴充

MV3，純 JavaScript ES module（不需建置），可測的邏輯放在獨立模組，用 vitest 加 chrome API 假物件測試。

- `manifest.json`：
  - `permissions`：`nativeMessaging`、`contextMenus`、`downloads`、`cookies`、`storage`、`notifications`
  - `host_permissions`：`<all_urls>`（讀 Cookie 需要）
  - 無 `content_scripts`
  - 固定 `key`
  - 圖示由 app 圖示產生 16/32/48/128
- **右鍵選單**：`contexts: link, image, video, audio`，在 `runtime.onInstalled` 建立一次（修正重複建立）。送出：網址是 `linkUrl` 或 `srcUrl`，Referer 是分頁網址，User-Agent 是 `navigator.userAgent`，再附上 Cookie。
  - 成功：工具列圖示 badge 顯示 ✓ 兩秒。
  - 失敗：系統通知，說明原因（app 未安裝、版本不合、網址或 header 無效）。
- **接管下載**：
  - 在 `downloads.onDeterminingFilename` 處理，這時已知瀏覽器決定的檔名（含 Content-Disposition）。
  - 符合條件就把網址、檔名、Referer、Cookie 送給 app。成功就 `cancel` 並 `erase`，瀏覽器的下載列表不留紀錄；失敗或逾時（15 秒）就照瀏覽器的建議檔名繼續下載。
  - 不接管的情況：
    - 開關關閉；
    - `blob:`、`data:`、`file:` 網址；
    - 其他擴充發起的下載（`byExtensionId`）；
    - 這次是使用者對 app 失敗後回退的同一個網址（避免循環）。
  - 限制（寫在 popup 說明裡）：POST 表單產生的下載，app 用 GET 重抓可能失敗。這種下載會在 app 裡顯示錯誤，使用者可以關掉接管後重試。
- **Popup**：兩列，一列是連線狀態（ping 結果：已連線、app 未開啟、app 未安裝、版本不合；ping 不會啟動 app），一列是「接管瀏覽器下載」開關（`chrome.storage.local`，預設開）。

## app 端

- `browser_bridge::serve`：app 啟動時開始監聽；每個連線讀一行、回一行後關閉。
- `add` 的處理與 `downloads_add` 共用同一個函式（預設目錄、檔名清理），不另寫一份。新任務照常出現在列表，透過既有的事件通知前端。
- **註冊 host**：app 每次啟動時寫一次，內容沒變就不動檔案與登錄。
  - 寫 `host-manifest.json` 到 app 資料夾，`path` 指向 app 旁的 `zoo-native-host(.exe)`。用 `serde_json` 產生，修正 D-2 的反斜線未跳脫問題。
  - Windows：`HKCU\Software\Google\Chrome\NativeMessagingHosts\com.zoo.downloader` 與 `HKCU\Software\Microsoft\Edge\NativeMessagingHosts\com.zoo.downloader` 的預設值指向該檔（`winreg`）。
  - macOS／Linux：把同一份 JSON 寫到 Chrome、Edge 各自的 `NativeMessagingHosts` 目錄。
  - 註冊失敗只記 log，不影響 app。
- **打包**：`zoo-native-host` 和 `zoo-module` 一樣用 `externalBin` 放在 app 旁，`pnpm build:sidecars`（原 `build:module-host`）一併編譯。

## 測試

- **zoo-native-host**：
  - native messaging 框架讀寫（含長度錯誤、EOF）；
  - 對假的 socket server 轉送 `ping` / `add`；
  - socket 不存在時嘗試啟動 app，app 路徑不存在則回 `appNotRunning`；
  - origin 參數不符時拒絕。
- **shell**：
  - 請求 JSON 的 camelCase 與往返測試；
  - `add` 轉成 `AddRequest` 的 headers；
  - 無效網址、header 回 `invalid`，未知版本回 `unsupportedVersion`；
  - host manifest 內容（Windows 路徑正確跳脫、`allowed_origins` 是固定 ID）。
- **擴充（vitest）**：
  - 接管條件（每一種不接管的情況）；
  - 送出訊息的組成（Cookie 串接；InPrivate 用自己的 Cookie，找不到就不附，絕不用一般視窗的）；
  - app 失敗時回退繼續下載；
  - 右鍵選單只建立一次。
- **實機**：
  - 我用 Edge 的獨立 profile（`--load-extension`，不碰你的瀏覽器）跑完整流程：
    - 右鍵送出需要 Cookie 的檔案（測試伺服器檢查 Cookie 與 Referer）；
    - 接管開、關各一次；
    - app 沒開時，由 host 啟動 app；
    - 解除安裝後回退到瀏覽器下載。
  - Chrome：使用者沒有安裝，視為與 Edge 相同（2026-11-15 決定）。右鍵選單是原生 UI，CDP 點不到，由使用者在 Edge 手動試一次。

## 階段

1. **協定與 app 端**：`browser_bridge`、共用的 add 路徑、host manifest 註冊；刪除 HTTP 入口與 `warp`。
2. **`zoo-native-host`**：轉送、啟動 app、pipe 擁有者檢查；打包設定。
3. **擴充**：manifest、右鍵、接管、popup、圖示；刪除舊檔。
4. **實機驗證與文件**：README 的安裝步驟、review 標註 D-1..D-3、SEC-1。

每個階段各自 commit。
