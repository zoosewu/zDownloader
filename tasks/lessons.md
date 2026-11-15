# Lessons

## 回覆語言一律使用繁體中文（2026-10-31）

- 錯誤：Phase 2 完成時的總結用英文撰寫，使用者要求「使用中文和我討論」。
- 原因：長時間連續執行工具後，總結時沒有回頭檢查語言設定。
- 規則：每則給使用者的訊息（包含總結、問題、錯誤說明）送出前確認是繁體中文；只有程式碼、識別字、指令維持原文。

## UI 讓版面自己說明（2026-10-31，全域規則新增）

- 規則：新增 UI 時不加多餘的次要標籤或說明文字（例如「留空表示立即開始」、格數統計）。標籤本身已能表達的就不再補充。
- 例外：沒有它就無法理解畫面的資訊（例如分段顏色圖例）。

## 在真的 app 裡操作過才算驗證完（2026-11-12）

- 錯誤：4e2 的 captcha 視窗只有單元測試，假設「capability 只給 main 視窗，外部網頁就沒有 IPC」。4e3 實際操作才發現兩件事：
  - Tauri 2 的 capability 不管 app 自己的 command，外部頁面能呼叫 `modules_list`；
  - 同步 command 建立視窗在 Windows 會 deadlock。
- 規則：
  - 涉及視窗、IPC、權限邊界的功能，要在真的 app 裡測：WebView2 加 `--remote-debugging-port`，用 CDP 操作並截圖。
  - 安全假設要寫成可以被推翻的檢查，例如從外部頁面呼叫 command，確認被拒絕。
  - Tauri command 要建立視窗時一律寫成 `async`。

## 要入庫的產物先確認沒被 .gitignore 吃掉（2026-11-13）

- 錯誤：4f 的 k2s `dist/index.js` 是 app 要執行的檔案，但根目錄 `.gitignore` 的 `dist` 讓它沒進 commit。本機測試全過，因為檔案在工作目錄裡。
- 規則：
  - 新增「要入庫的建置產物」時，`git check-ignore -v <檔案>` 確認沒被忽略，commit 後看 `git show --stat` 有沒有它。
  - 測試通過不代表 commit 完整：本機有、repo 沒有的檔案只有乾淨的 checkout 才會暴露。

## 含反斜線的內容不要經過 python／shell 字串改寫（2026-11-14）

- 錯誤：用 python heredoc 寫入 `'C:\dl\a (1).bin'` 與正規表示式 `/[\/]/`，反斜線被吃掉，變成 `'C:\dl...'` 與 `/[\/]/`。測試抓到了，但這是第二次（11-12 的 todo 路徑也是）。
- 規則：程式碼或文字裡有反斜線（Windows 路徑、正規表示式、跳脫字元）時，一律用 Edit／Write 工具，不用 python、sed 或 PowerShell 字串取代。改完 grep 一次確認。

## 不要截整個螢幕（2026-11-14）

- 錯誤：為了看 Windows 系統通知，截了整個螢幕，拍到使用者正在用的其他程式（全螢幕遊戲）。已立即刪除。
- 規則：
  - 驗證 app 只截 app 自己的畫面（CDP `Page.captureScreenshot`），不截桌面。
  - 系統層級的東西（通知、工作列）看不到時，記為「需要人工確認」，不要用截整個螢幕代替。
  - 使用者可能正在用電腦：剪貼簿、焦點、彈出視窗都會打擾到人，用之前想一下，用完還原。

## 自動檢查時不要點會開系統對話框的按鈕（2026-11-14）

- 錯誤：逐頁掃描畫面時點了「批次匯入」，它會開系統的檔案選擇視窗。那個視窗在 app 外面、出現在使用者桌面上，CDP 也關不掉。
- 規則：自動點擊前先看按鈕做什麼；會開系統對話框的（選檔案、選資料夾、儲存）不要點，改直接呼叫後端 command 或只檢查按鈕本身。

## Tauri setup 不在 Tokio runtime 裡（2026-11-15）

- 錯誤：在 `.setup()` 裡直接建立 interprocess 的 tokio named pipe listener，app 一啟動就 panic（`there is no reactor running`）。單元測試在 `#[tokio::test]` 裡跑，所以全部通過。
- 規則：需要 Tokio reactor 的東西（tokio 的 socket、pipe、timer）一律放進 `tauri::async_runtime::spawn` 裡建立。啟動相關的改動一定要實際開 app 看 stderr，log 檔不會記 panic。

## Windows 子程序會繼承所有可繼承的 handle（2026-11-15）

- 錯誤：native messaging host 啟動 app 時把 stdio 設成 null，但 app 仍繼承了 host 自己的 stdin/stdout/stderr（瀏覽器的 pipe），呼叫端一直等到 app 結束。
- 規則：要比自己活得久的子程序（啟動 app、daemon），在 Windows 上先把自己的 std handle 設為不可繼承（`SetHandleInformation(.., HANDLE_FLAG_INHERIT, 0)`）。測試時用「呼叫端是否在 host 結束後立刻返回」驗證，不只看回應內容。

## 實機測試前重建所有 sidecar（2026-11-15）

- 錯誤：改了 `zoo-native-host` 後只跑了 `cargo test`，實機測試用的 debug host 還是舊版，popup 的 ping 把 app 叫了起來，差點誤判成擴充的 bug。
- 規則：實機驗證前 `cargo build` 所有會被執行的 binary（app、`zoo-module`、`zoo-native-host`），`cargo test` 不會更新它們。

## 「獨立」瀏覽器 profile 會自動登入並同步（2026-11-15）

- 錯誤：用新的 `--user-data-dir` 開 Edge，Edge 自動以使用者的帳號登入並同步，使用者的擴充（NDM、FDM）全部出現並搶走下載事件。在這種 profile 改擴充設定可能同步回使用者平常用的瀏覽器。
- 規則：自動化用的 Chrome／Edge 一律加 `--disable-sync`，開好後先用 `chrome.management.getAll()` 確認只有預期的擴充；發現同步已發生就關掉並刪除 profile 重來，不要在裡面改任何設定。
