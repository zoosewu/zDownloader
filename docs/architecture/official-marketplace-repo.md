# 官方 marketplace 獨立成 repo（2026-11-18）

## 決定

- 官方 marketplace 移到 `zoosewu/zoo-downloader-marketplace`，app 從 GitHub 讀，不再打包進 app。
- 官方模組可以不跟 app 發版就更新；舊 app 靠 API level 判斷能不能用（`module-versions.md`）。
- app 不保留內建的一份：沒有網路就裝不了新模組，已安裝的模組照常離線執行。

## app 端

1. **官方來源**：`OFFICIAL_MARKETPLACE = "zoosewu/zoo-downloader-marketplace"`（GitHub，經
   `raw.githubusercontent.com/.../HEAD/zoo-marketplace.json`），每次啟動 `set_official` 註冊，信任等級
   official，不能移除。catalog 的 `name` 維持 `official`，已安裝的 `baidu@official` 等不必改 key。
   - 遠端 marketplace 每個模組都要 `sha256`（現有規則）。
   - 開發版可用環境變數 `ZOO_OFFICIAL_MARKETPLACE=<資料夾>` 指向本機 clone，模組改了即時重新載入
     （現有的本機來源機制）。
2. **遷移已安裝的官方模組**：現在官方模組在原地（app 的 resources 資料夾）執行，app 更新後那些檔案
   會消失。啟動時，對官方模組中檔案不在的：
   - 從遠端 catalog 下載同一模組（`plan_install`）；權限沒有增加就直接裝回去（帳號、開關、順序都保留）；
     權限增加則標成「需要同意」，等使用者按。
   - 沒網路時保留紀錄、標成「需要重新下載」，下次啟動或重新整理再試；這段時間該模組不參與解析。
   - 之後一律「下載並複製」到 app 資料夾（遠端來源本來就是這樣），不再有原地執行的官方模組。
3. 移除 `tauri.conf.json` 的 `marketplace/*` resources、`site_modules::official_dir`、
   `the_official_marketplace_installs_and_is_bundled` 測試；改成測試「指定的資料夾當官方來源」。
4. 檢查更新：沿用「啟動時檢查更新」開關與 `refresh`。

## 新 repo（zoo-downloader-marketplace）

```
zoo-marketplace.json      # name: official；每個模組 version、apiVersion、source、sha256
baidu/  k2s/              # 現有的模組專案原樣搬過去（src、dist、tests/cassettes、types、AGENTS.md）
scripts/build.mjs         # 由 build-official-modules.mjs 改來：打包、檢查 dist 是最新
scripts/catalog.mjs       # 由各模組 manifest 與 dist 產生 catalog（version、apiVersion、sha256）
scripts/sync-sdk.mjs      # 從 app repo 的某個 tag 取 sdk/ 到各模組 types/
.github/workflows/ci.yml  # 型別檢查、dist 與 catalog 是最新、types 是 SDK、zoo-module test（cassette）
```

- `zoo-module` CLI：app 的 release workflow 另外附上 `zoo-module-<target>` 執行檔，marketplace 的 CI
  下載對應版本來跑 `zoo-module test`，不必編譯 v8。
- 發佈模組：改版本、`pnpm build && pnpm catalog`、推送；推上 `main` 就生效（app 讀 `HEAD`）。
  要先試再發，可在分支上用「新增 marketplace → 該分支的 catalog 網址」測。

## app repo 留下

- SDK（`crates/zoo-module/sdk`、`template`）、`zoo-module` CLI、host、API level 定義。
- `src/__tests__/official-modules.test.ts` 與 `build-official-modules.mjs` 搬到新 repo；app repo 改用
  一個最小的範例模組測 SDK 範本（已有 `module-sdk.test.ts`）。

## 進度

1. ✅ 新 repo 推上 baidu 1.1.1、k2s 1.0.0 與 catalog（含 sha256），CI 通過。
2. ✅ app：官方來源改遠端（`site_modules::OFFICIAL_MARKETPLACE`）、`restore_missing` 遷移、移除打包；
   app repo 刪除 `src-tauri/marketplace/`。
3. ✅ release workflow 附 `zoo-module-<target>` 執行檔（v0.2.5 起）；新 repo 的 CI `replay` job 下載最新 release
   的 Linux 版，在各模組跑 `zoo-module test`。
