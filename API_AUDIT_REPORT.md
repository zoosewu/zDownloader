# API Serialization Audit Report
**日期**: 2025-12-03
**檢查項目**: 前後端 API 序列化規範符合性

## 🔍 檢查方法

```bash
# 找出所有缺少 rename_all 的序列化結構
cd src-tauri/src
rg -A 1 '#\[derive.*Serialize' | rg -B 1 'pub (struct|enum)' | rg -v 'rename_all'
```

## ❌ 發現的問題

### 嚴重問題（前端會使用的 API）

| 檔案 | 結構 | 類型 | 狀態 | 影響 |
|------|------|------|------|------|
| `models/settings.rs` | `Theme` | enum | ❌ 缺少 rename_all | 前端設定頁面 |
| `models/settings.rs` | `SpeedLimitMode` | enum | ❌ 缺少 rename_all | 前端設定頁面 |
| `models/download_task.rs` | `DownloadStatus` | enum | ❌ 缺少 rename_all | **高優先級** - 下載列表 |
| `models/download_segment.rs` | `SegmentStatus` | enum | ❌ 缺少 rename_all | 下載進度顯示 |
| `models/proxy_info.rs` | `ProxyInfo` | struct | ❌ 缺少 rename_all | 代理列表 |
| `models/proxy_info.rs` | `ProxyType` | enum | ❌ 缺少 rename_all | 代理設定 |
| `models/proxy_info.rs` | `ProxyStatus` | enum | ❌ 缺少 rename_all | 代理狀態 |
| `commands/captcha.rs` | `ChallengeInfo` | struct | ❌ 缺少 rename_all | CAPTCHA 對話框 |

### 中等問題（內部使用，但可能暴露給前端）

| 檔案 | 結構 | 類型 | 狀態 |
|------|------|------|------|
| `modules/interface.rs` | `ModuleMetadata` | struct | ❌ 缺少 rename_all |
| `modules/interface.rs` | `DownloadInfo` | struct | ❌ 缺少 rename_all |
| `modules/interface.rs` | `CaptchaChallenge` | struct | ❌ 缺少 rename_all |
| `modules/interface.rs` | `CaptchaSolution` | struct | ❌ 缺少 rename_all |
| `modules/interface.rs` | `AuthCredentials` | struct | ❌ 缺少 rename_all |
| `modules/interface.rs` | `ModuleError` | enum | ❌ 缺少 rename_all |

### 低優先級（主要用於日誌/錯誤）

| 檔案 | 結構 | 類型 | 狀態 |
|------|------|------|------|
| `utils/error.rs` | `AppError` | struct | ❌ 缺少 rename_all |
| `utils/events.rs` | `CaptchaRequiredEvent` | struct | ❌ 缺少 rename_all |
| `utils/events.rs` | `CaptchaSolvedEvent` | struct | ❌ 缺少 rename_all |
| `utils/events.rs` | `CaptchaCancelledEvent` | struct | ❌ 缺少 rename_all |

### ✅ 已符合規範

| 檔案 | 結構 | 狀態 |
|------|------|------|
| `models/download_task.rs` | `DownloadTask` | ✅ 有 rename_all |
| `models/settings.rs` | `AppSettings` | ✅ 有 rename_all |
| `modules/url_detector/mod.rs` | `UrlDetectionInfo` | ✅ 有 rename_all |
| `modules/url_detector/mod.rs` | `DetectionMode` | ✅ 有 rename_all |

## 🎯 修復優先級

### P0 - 立即修復（已在使用，會導致前端錯誤）

1. **DownloadStatus** - 下載狀態 enum
   - 影響：所有下載列表功能
   - 前端已使用：✅
   - 測試：❌ 缺少

2. **Theme / SpeedLimitMode** - 設定相關 enum
   - 影響：設定頁面
   - 前端已使用：✅
   - 測試：❌ 缺少

3. **ProxyInfo / ProxyType / ProxyStatus** - 代理相關
   - 影響：代理管理功能
   - 前端已使用：可能
   - 測試：❌ 缺少

### P1 - 盡快修復（即將使用）

4. **SegmentStatus** - 下載片段狀態
5. **ChallengeInfo** - CAPTCHA 相關

### P2 - 建議修復（預防性）

6. `modules/interface.rs` 中的所有結構
7. Events 結構
8. `AppError`

## 🔧 建議修復方案

### 方案 A：立即全部修復（推薦）

**優點**：
- 一次性解決所有潛在問題
- 防止未來的 bug
- 符合最佳實踐

**工作量**：~1小時

### 方案 B：分階段修復

**階段 1** (P0): DownloadStatus, Theme, SpeedLimitMode
**階段 2** (P1): ProxyInfo 相關, SegmentStatus
**階段 3** (P2): 其餘結構

**優點**：
- 可以優先處理關鍵問題
- 分散風險

## 📝 修復檢查清單

針對每個需要修復的結構：

- [ ] 添加 `#[serde(rename_all = "camelCase")]`
- [ ] 運行 `cargo test` 確保沒有破壞現有功能
- [ ] 檢查前端是否有對應的 TypeScript interface
- [ ] 如果前端有 interface，確認命名一致
- [ ] 添加序列化測試（參考 url_detector 的測試）
- [ ] 更新 `test-fixtures/api-samples.json`

## 🚨 風險評估

### 如果不修復

1. **DownloadStatus**: 前端可能無法正確顯示下載狀態
   ```typescript
   // 前端期望: "downloading"
   // 實際收到: "Downloading" (PascalCase)
   // 結果: 狀態判斷失敗
   ```

2. **Theme**: 設定頁面主題切換可能失敗
3. **ProxyInfo**: 代理列表顯示異常

### 已知的變通方案

某些 enum 如果值是單個單詞（如 `Light`, `Dark`），camelCase 和 PascalCase 相同，可能暫時不會出錯。但為了一致性，仍建議修復。

## 📊 統計

- **總共發現**: 18 個結構缺少 `rename_all`
- **高優先級**: 8 個
- **中優先級**: 6 個
- **低優先級**: 4 個
- **已符合規範**: 4 個

## 🔄 後續行動

1. **立即**: 修復 P0 項目（DownloadStatus, Theme, SpeedLimitMode）
2. **本週**: 修復 P1 項目
3. **持續**:
   - 添加 pre-commit hook 檢查
   - 更新 CLAUDE.md 強調檢查 enum
   - 為所有修復的結構添加測試

## 📖 參考

- [CLAUDE.md - API 命名規範](./CLAUDE.md#api-命名規範與序列化-critical)
- [TESTING.md - 測試指南](./TESTING.md)
- [Serde 文檔](https://serde.rs/container-attrs.html#rename_all)
