# Malicious Test Modules

這些模組專門設計用於測試 zoo-downloader 模組系統的安全機制。**這些是故意惡意的測試模組**，用於驗證安全防護是否正常運作。

## ⚠️  重要警告

**這些模組僅用於安全測試，絕對不要在生產環境中啟用！**

所有測試模組都會嘗試執行可能有害的操作，包括：
- 存取未經授權的網路資源
- 嘗試逃逸沙箱環境
- 試圖竊取認證資訊
- 消耗過多系統資源
- 執行無限迴圈

## 測試模組清單

### 1. unauthorized_domain.js (T090)
**測試目標**: 網域權限限制 (SC-010)

嘗試：
- 存取不在 manifest allowlist 中的網域
- 存取內網 IP (192.168.x.x, 10.x.x.x)
- 存取 localhost

**預期結果**: 所有未經授權的網域存取都應被 PermissionChecker 阻止。

---

### 2. resource_exhaustion.js (T091)
**測試目標**: 資源限制執行 (SC-013)

嘗試：
- 超過 rate limit（發送 > 60 requests/min）
- 下載 > 10MB 的回應
- 分配大量記憶體

**預期結果**: NetworkMonitor 應執行 rate limiting 和 response size limits。

---

### 3. ssrf_attempt.js (T092)
**測試目標**: SSRF 防護 (SC-014)

嘗試 12+ 種 SSRF 攻擊：
- Private IP ranges (10.x.x.x, 192.168.x.x, 172.16-31.x.x)
- Localhost variants (127.0.0.1, ::1, 0x7f000001)
- Link-local addresses (169.254.x.x)
- AWS metadata service
- DNS tricks (localtest.me, nip.io)

**預期結果**: UrlValidator 應阻止所有 SSRF 攻擊。

---

### 4. sandbox_escape.js (T093)
**測試目標**: 沙箱逃逸防護 (SC-012)

嘗試 7+ 種逃逸技術：
- `eval()` 和 `Function()` constructor
- Deno/process namespace 存取
- Prototype pollution (`Object.prototype`, `__proto__`)
- `constructor.constructor` escape

**預期結果**: ModuleSandbox 應阻止所有逃逸嘗試。

---

### 5. data_exfiltration.js (T094)
**測試目標**: 可疑行為偵測 (SC-017)

嘗試：
- 快速請求多個不同網域
- 上傳大量資料
- Base64 編碼的資料傳輸
- 異常請求頻率模式

**預期結果**: NetworkMonitor 應偵測到可疑的網路模式（< 5% 誤報率）。

---

### 6. credential_access.js (T095)
**測試目標**: 認證隔離 (SC-011)

嘗試：
- 讀取原始認證值
- Monkey-patch HTTP 方法攔截認證
- 枚舉所有認證 keys
- 透過 toString/valueOf 提取值

**預期結果**: CredentialProxy 應確保 JavaScript 永遠無法存取原始認證值。

---

### 7. prototype_pollution.js (T096)
**測試目標**: Prototype Pollution 防護 (SC-012)

嘗試 7+ 種 pollution 技術：
- Direct Object.prototype modification
- `__proto__` assignment
- `constructor.prototype` manipulation
- Array.prototype pollution
- JSON parsing attacks
- Deep merge pollution
- `Object.defineProperty()` pollution

**預期結果**: Sandbox 應凍結所有 prototype，防止任何 pollution。

---

### 8. infinite_loop.js (T097)
**測試目標**: 執行逾時執行 (SC-013)

嘗試 6+ 種長時間執行：
- Simple infinite while loop
- Infinite recursion (stack overflow)
- CPU-intensive computation
- Async infinite loop
- Promise chain abuse
- Combined memory + CPU exhaustion

**預期結果**: 執行應在合理時間內被終止（如 30 秒）。

---

## 如何運行安全測試

### 方法 1: 使用 Rust 測試框架

```bash
cd src-tauri
RUST_BACKTRACE=1 cargo test modules::tests::security_tests --lib -- --nocapture
```

這會運行基礎的安全測試框架驗證。

### 方法 2: 完整整合測試（需要實作）

完整的安全測試需要：
1. 載入每個測試模組
2. 在沙箱中執行
3. 捕獲所有安全事件
4. 驗證攻擊都被成功阻止

這部分功能在 Phase 5.2 (T099-T107) 中實作。

---

## 成功標準

**所有測試必須通過 100% 成功率**（SC-010 ~ SC-017）：

| 測試 | 標準 | 目標 |
|------|------|------|
| T099 | SC-010 | 100% 權限違規被偵測 |
| T100 | SC-011 | 100% 認證隔離成功 |
| T101 | SC-012 | 100% 沙箱逃逸被阻止 |
| T102 | SC-013 | 100% 資源限制執行 |
| T103 | SC-014 | 100% SSRF 攻擊被阻止 |
| T104 | SC-015 | 100% URL 驗證通過 |
| T105 | SC-016 | 100% 審計日誌完整性 |
| T106 | SC-017 | < 5% 誤報率 |

---

## 審計日誌

所有安全測試執行時，應在 `logs/audit/` 中產生詳細的審計日誌，記錄：
- 每次攻擊嘗試
- 阻止的時間點
- 安全機制的回應

---

## 維護注意事項

當模組系統的安全機制更新時，應該：
1. 更新相對應的測試模組
2. 添加新的攻擊場景測試
3. 確保所有測試仍然通過

**這些測試模組是活文件，應隨著威脅模型演進而更新。**

---

**最後更新**: 2025-12-07
**Feature**: 004-site-module-integration
**Phase**: 5 (Security Testing)
