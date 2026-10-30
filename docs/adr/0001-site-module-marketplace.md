# ADR-0001：網站模組以「使用者自訂 marketplace」散布

- 狀態：方向已接受（2026-10-30），細節待設計
- 相關：[架構檢視 2026-10-30](../architecture/review-2026-10-30.md) CAND-3、B-7、SPEC-5、SEC-3..6

## 背景

目前網站模組是 repo 根目錄 `modules/` 下的資料夾，release 版以相對於 CWD 的 `./modules` 尋找（B-7），沒有安裝、更新或來源概念。spec 004 FR-026 要求「模組對使用者不可見」，但後續又做了模組管理 UI（SPEC-5）。

## 決策

網站模組的取得方式比照 Claude Code 的 plugin marketplace：

- 使用者自行**新增 marketplace 來源**。來源可以是 GitHub `owner/repo`、git URL、指向 catalog 檔的 HTTPS URL，或本機目錄。
- 每個 marketplace 是一份 **catalog**（例如 `zoo-marketplace.json`），列出它提供的網站模組：id、來源位置、版本、說明、URL patterns、權限摘要。
- 使用者從 catalog **安裝 / 停用 / 移除 / 更新**網站模組，以 `模組id@marketplace名稱` 識別，避免不同來源的 id 衝突。
- 內建模組（例如 k2s）以 app 內附的官方 marketplace 提供，不再特別處理。

參考：Claude Code 的作法是 `/plugin marketplace add <repo|url|path>`、`.claude-plugin/marketplace.json`、`/plugin install name@marketplace`。實作前需再查證其最新格式。

## 影響

1. **網站模組變成第三方未受信任的程式碼**。沙箱、資源限制與網路 policy（CAND-3 的 `SiteModuleHost` + `Policy`）從「加分」變成必要條件；JS runtime 選擇應以可強制 memory limit 與中斷執行為首要條件（傾向 QuickJS / rquickjs）。
2. **權限在安裝時顯示並由使用者同意**。更新時如果權限擴大，要再次徵求同意。這讓 spec 004 FR-033（權限對話框）有了明確的觸發點。
3. **模組安裝到 app data 目錄**，例如 `<app_data>/modules/<marketplace>/<id>/`，版本鎖定在 commit SHA 或內容雜湊。這樣就消除了 B-7 的 CWD 相對路徑問題。
4. spec 004 FR-026「模組不可見」作廢，改為「模組管理是一級功能」。
5. 模組開發者介面（d.ts、template）需要隨 marketplace 規格一起發佈，並與 host functions 由同一份來源產生（D-4）。

## 待設計問題

- catalog 格式：沿用 Claude Code 的欄位命名，還是自訂？
- 版本與完整性：git ref 鎖定、內容雜湊、是否支援簽章？
- 更新策略：手動 / 自動檢查 / 自動安裝？
- 網站模組是否可以有建置步驟（TypeScript、打包），還是 catalog 只接受已建置的單一 JS 檔？
- 多個模組同時匹配同一個 URL 時，優先順序如何決定？
- 離線時要怎麼處理：已安裝的模組都要能離線運作。
- credential 歸屬：綁定 `id@marketplace`，換來源時是否沿用？
