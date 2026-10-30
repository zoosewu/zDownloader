# Glossary

zoo-downloader 的領域術語。程式碼、文件、討論都使用這些名稱。

## 下載

- **Download task（下載任務）**：使用者要求下載的一個檔案，從加入到完成、失敗或取消為止。以 `TaskId` 識別。
- **Segment（片段）**：任務中的一段 byte range，可由一條連線獨立下載。
- **PartFile**：下載中的目標檔（`<name>.<task-id>.part`），連同已完成區段的紀錄（`.part.json`）；完成時 rename 成最終檔名。
- **Transfer**：實際執行一個任務的傳輸工作（探測、分段或串流、重試），可被取消。
- **DownloadManager**：唯一擁有所有任務狀態並決定何時執行的 module（`zoo-core`）。
- **Task state（任務狀態）**：Scheduled → Queued → Running → Completed / Failed / Paused / Cancelled。
- **rev（版本）**：任務每次變更遞增的序號；同一任務的兩份資料，以 rev 較大者為準。
- **ConnectionSource**：每個請求取得連線的來源，有直連（Direct）與代理池（ProxyPool）兩個 adapter。

## 網站模組

- **Hoster（檔案空間）**：需要額外步驟才能取得真實下載連結的網站，例如 k2s。
- **Site module（網站模組）**：針對特定 hoster 的 JS 程式，把分享頁 URL 解析成可直接下載的資訊。為避免與架構術語的「module」混淆，一律寫成「網站模組 / site module」。
- **Resolve（解析）**：網站模組把 URL 轉成 `{真實 URL, headers, 檔名}` 的動作。
- **SiteModuleHost**：載入並執行網站模組、對外提供 list / match_url / resolve / reload 的 module。（目標設計，見 CAND-3）
- **Policy**：網站模組每次對外 I/O 都必須通過的檢查：網域 allowlist、SSRF、速率限制、稽核，以及 credential 替換。（目標設計）
- **Marketplace**：一份由使用者新增的網站模組來源，提供一份 catalog。見 [ADR-0001](docs/adr/0001-site-module-marketplace.md)。
- **Catalog**：marketplace 中列出可安裝網站模組的清單檔。
- **Installed module（已安裝模組）**：從某個 marketplace 安裝、鎖定版本的網站模組，以 `id@marketplace` 識別。

## 其他

- **Ingress（入口）**：外部進入下載流程的來源，例如 UI、Chrome 擴充、剪貼簿。所有入口都經過 `Downloads::add`。（目標設計，見 CAND-4）
