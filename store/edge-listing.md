# Edge 附加元件商店上架資料

上架到 Microsoft Edge Add-ons（Partner Center）要填的內容與步驟。Chrome 線上應用程式商店之後可沿用大部分文字。

## 0. 先準備

| 項目 | 狀態 |
|---|---|
| Microsoft 合作夥伴中心的開發人員帳號（Edge 免費） | 使用者註冊：<https://partner.microsoft.com/dashboard/microsoftedge/> |
| 上架用壓縮檔 | `pnpm package:extension` → `dist-store/zoo-downloader-extension-<版本>.zip`（不含 manifest `key`、測試、README） |
| 隱私權政策網址 | `PRIVACY.md` 推送後：<https://github.com/zoosewu/zDownloader/blob/main/PRIVACY.md> |
| 商店標誌 300×300 | `store/logo-300.png`（`pwsh scripts/make-store-assets.ps1` 產生） |
| 小型宣傳圖塊 440×280（選填） | `store/tile-440x280.png` |
| 螢幕擷取畫面 1280×800 或 640×400（選填，建議 1–3 張） | 使用者擷取，見第 5 節 |

## 1. 上架順序（擴充 ID 會改變）

商店會給擴充一個新的 ID（和手動載入的不同），app 必須認得它才能接受這個擴充的連線：

1. Partner Center → **建立新的擴充功能**，上傳 zip，先不要送審。
2. 在擴充的總覽頁抄下商店給的**擴充 ID**（32 個小寫字母）。
3. 把 ID 交給開發（Claude）：加進 `zoo-native-host` 的允許清單與 host manifest 的 `allowed_origins`，app 發一版。（已完成：`bhgmngdlfelamchfhlahkkclffhpgfma`，商店識別碼 0RDCKBKMXKR6。）
4. 回 Partner Center 填完下面各頁，**送審**（審查通常數個工作天）。
5. 通過後：設定頁的「瀏覽器擴充」改成直接連到商店頁面（開發）。

## 2. 屬性（Properties）

- **類別（Category）**：Productivity（生產力）
- **隱私權政策**：此擴充功能**會**存取個人資訊（Cookie）→ 填上方隱私權政策網址
- **網站**：<https://github.com/zoosewu/zDownloader>
- **支援聯絡**：<https://github.com/zoosewu/zDownloader/issues>
- **成熟度**：一般（無成人內容）

## 3. 商店清單（Store listing，每種語言）

描述與搜尋字詞不提任何網站模組或網站的名稱，也不提 InPrivate（使用者決定，2026-11-17；InPrivate 的行為寫在 PRIVACY.md）。

擴充名稱取自 manifest（`Zoo Downloader`），不用填。

### English

**Description**

> Zoo Downloader sends your downloads to the Zoo Downloader desktop app, which downloads them faster and more reliably: several connections per file, pause and resume, automatic retries, scheduling, categories, and site modules for file-sharing sites.
>
> - Right-click a link, image, video or audio and choose "Download with Zoo Downloader".
> - Or let it take over the downloads the browser starts (on by default; switch it off in the popup). If the app does not answer, the browser downloads as usual.
> - The site's cookies, Referer and User-Agent go along, so downloads that need a sign-in work.
>
> Requires the free Zoo Downloader app for Windows, macOS or Linux: https://github.com/zoosewu/zDownloader/releases
>
> The extension talks only to the app on your own computer (Native Messaging). It makes no network requests, collects nothing and sends nothing to us or anyone else.

**Search terms**（最多 7 個）：`download manager`, `downloader`, `accelerator`, `resume download`, `batch download`, `multi-connection download`

### 繁體中文

**描述**

> Zoo Downloader 把你的下載交給 Zoo Downloader 桌面 app，下載得更快、更穩：單一檔案多條連線、暫停與續傳、自動重試、排程、分類，也能透過網站模組處理分享網站的下載。
>
> - 在連結、圖片、影片或音訊上按右鍵，選「用 Zoo Downloader 下載」。
> - 也可以讓它接管瀏覽器開始的下載（預設開啟，可在彈出視窗關閉）。app 沒有回應時，照常由瀏覽器下載。
> - 會一併帶上網站的 Cookie、Referer 與 User-Agent，需要登入的下載也能成功。
>
> 需要免費的 Zoo Downloader app（Windows、macOS、Linux）：https://github.com/zoosewu/zDownloader/releases
>
> 擴充功能只和你電腦上的 app 溝通（原生訊息），本身不連網路、不收集任何資料，也不會把資料送給我們或任何人。

**搜尋字詞**：`下載管理員`, `下載器`, `續傳`, `批次下載`, `加速下載`, `多線程下載`

### 日本語

**説明**

> Zoo Downloader は、ダウンロードを Zoo Downloader デスクトップ アプリに渡し、より速く確実にダウンロードします。1 ファイルを複数の接続で取得、一時停止と再開、自動リトライ、スケジュール、カテゴリに対応し、サイト モジュールで共有サイトのダウンロードも扱えます。
>
> - リンク、画像、動画、音声を右クリックして「Zoo Downloader でダウンロード」を選びます。
> - ブラウザーが始めたダウンロードを引き継ぐこともできます（既定でオン、ポップアップでオフにできます）。アプリが応答しない場合は、通常どおりブラウザーがダウンロードします。
> - サイトの Cookie、Referer、User-Agent も渡すので、ログインが必要なダウンロードも成功します。
>
> 無料の Zoo Downloader アプリ（Windows、macOS、Linux）が必要です: https://github.com/zoosewu/zDownloader/releases
>
> 拡張機能は同じコンピューター上のアプリとだけ通信し（ネイティブ メッセージング）、ネットワークには接続せず、データを収集・送信しません。

**検索語句**：`ダウンロード マネージャー`, `ダウンローダー`, `レジューム`, `一括ダウンロード`, `高速ダウンロード`

## 4. 給審查人員的說明（Notes for certification）

> This extension hands downloads to a companion desktop app through Native Messaging (host name `com.zoo.downloader`); without the app installed it only shows "Zoo Downloader not found" in its popup.
>
> To test:
> 1. Install the Windows app (zoo-downloader_<version>_x64-setup.exe) from https://github.com/zoosewu/zDownloader/releases/latest and start it once; it registers the native messaging host for Edge.
> 2. Install this extension. Its popup shows "Connected".
> 3. On the releases page above, right-click any asset link (for example the browser extension .zip) → "Download with Zoo Downloader": the app comes forward with the link in its Add downloads dialog; click "Add" and it downloads.
> 4. Click an asset link normally: with "Take over browser downloads" on (the default), the browser's download is cancelled and removed, and the link goes to the app the same way.
>
> Permissions: nativeMessaging (talk to the app), contextMenus (the right-click entry), downloads (take over and remove browser downloads), cookies + <all_urls> (cookies of the URL being downloaded, so sign-in downloads work; files can come from any site), storage (the take-over switch), notifications (why a hand-over failed). The extension makes no network requests and collects no data.

## 5. 螢幕擷取畫面（建議）

1280×800，**用示範資料**（不要露出自己的下載清單、帳號或網址）：

1. 網頁上的連結按右鍵，選單中有「用 Zoo Downloader 下載」。
2. 擴充彈出視窗顯示「已連線」與「接管瀏覽器下載」。
3. app 的下載清單收到該下載。

## 6. 之後更新

- 修改 `browser-extension/manifest.json` 的 `version`（商店要求每次上傳版本號遞增），`pnpm package:extension`，在 Partner Center 上傳新版送審。
- 改了權限（`permissions`、`host_permissions`）要同步更新 `PRIVACY.md` 的權限表與第 4 節。
