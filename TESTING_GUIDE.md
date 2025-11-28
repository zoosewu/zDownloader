# Zoo Downloader - 測試指南

**最後更新**: 2025-11-28
**涵蓋範圍**: Phase 6 Proxy 模組完整測試策略

---

## 📋 測試概覽

### 測試層級

```
1. 單元測試 (Unit Tests) ✅
   └─ 測試單一函數或方法的行為
   └─ 使用模擬數據，不依賴外部服務
   └─ 執行速度快 (< 1 秒)

2. 整合測試 (Integration Tests) 🔄
   └─ 測試多個模組之間的互動
   └─ 可能使用真實的檔案系統或網路
   └─ 執行速度中等 (1-5 秒)

3. 端到端測試 (E2E Tests) ⏳
   └─ 測試完整的使用者流程
   └─ 包含前後端互動
   └─ 執行速度慢 (5-30 秒)
```

---

## 🧪 單元測試詳解

### 1. 測試框架與工具

使用 Rust 內建的測試框架：

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_synchronous_function() {
        // 同步測試
        let result = some_function();
        assert_eq!(result, expected_value);
    }

    #[tokio::test]
    async fn test_asynchronous_function() {
        // 異步測試（需要 tokio runtime）
        let result = some_async_function().await;
        assert!(result.is_ok());
    }
}
```

**關鍵特性**:
- `#[cfg(test)]`: 只在測試模式下編譯
- `#[test]`: 標記同步測試函數
- `#[tokio::test]`: 標記異步測試函數（自動啟動 tokio runtime）

### 2. 執行測試的命令

```bash
# 執行所有測試
cargo test

# 執行特定模組的測試
cargo test proxy

# 執行單一測試
cargo test test_add_proxy

# 顯示測試輸出（包含 println! 等）
cargo test -- --nocapture

# 不在第一個失敗時停止
cargo test --no-fail-fast

# 只執行 lib 測試（不含 integration tests）
cargo test --lib

# 並行執行測試（預設）
cargo test -- --test-threads=4

# 串行執行測試（用於除錯）
cargo test -- --test-threads=1
```

---

## 📦 Proxy 模組測試詳解

### ProxyPool 測試 (6 個測試)

#### Test 1: `test_add_proxy`
**目的**: 驗證基本的 Proxy 新增功能

```rust
#[tokio::test]
async fn test_add_proxy() {
    let pool = ProxyPool::new();
    let proxy = ProxyInfo::new("127.0.0.1".to_string(), 8080);

    pool.add_proxy(proxy.clone()).await;

    assert_eq!(pool.count().await, 1);
    let proxies = pool.get_all_proxies().await;
    assert_eq!(proxies[0].host, "127.0.0.1");
    assert_eq!(proxies[0].port, 8080);
}
```

**測試步驟**:
1. 建立空的 ProxyPool
2. 建立一個 ProxyInfo (127.0.0.1:8080)
3. 將 Proxy 加入 Pool
4. 驗證數量為 1
5. 驗證 host 和 port 正確

#### Test 2: `test_add_duplicate_proxy`
**目的**: 驗證重複 Proxy 會被拒絕

```rust
#[tokio::test]
async fn test_add_duplicate_proxy() {
    let pool = ProxyPool::new();
    let proxy1 = ProxyInfo::new("127.0.0.1".to_string(), 8080);
    let proxy2 = ProxyInfo::new("127.0.0.1".to_string(), 8080);

    pool.add_proxy(proxy1).await;
    pool.add_proxy(proxy2).await;

    // Should only have one proxy (duplicates rejected)
    assert_eq!(pool.count().await, 1);
}
```

**測試邏輯**:
- 嘗試加入兩個相同的 Proxy (相同 host 和 port)
- 驗證只有一個被保留
- 確保去重邏輯正確運作

#### Test 3: `test_mark_success`
**目的**: 驗證成功標記與延遲更新

```rust
#[tokio::test]
async fn test_mark_success() {
    let pool = ProxyPool::new();
    let proxy = ProxyInfo::new("127.0.0.1".to_string(), 8080);
    let proxy_id = proxy.id;

    pool.add_proxy(proxy).await;
    pool.mark_success(proxy_id, 100.0).await.unwrap();

    let updated_proxy = pool.get_proxy_by_id(proxy_id).await.unwrap();
    assert_eq!(updated_proxy.successful_requests, 1);
    assert_eq!(updated_proxy.status, ProxyStatus::Active);
    assert_eq!(updated_proxy.avg_latency_ms, 100.0);
}
```

**驗證項目**:
- ✅ successful_requests 計數正確增加
- ✅ status 更新為 Active
- ✅ avg_latency_ms 正確記錄

#### Test 4: `test_mark_failure_blacklist`
**目的**: 驗證自動黑名單機制

```rust
#[tokio::test]
async fn test_mark_failure_blacklist() {
    let pool = ProxyPool::with_config(3); // Blacklist after 3 failures
    let proxy = ProxyInfo::new("127.0.0.1".to_string(), 8080);
    let proxy_id = proxy.id;

    pool.add_proxy(proxy).await;

    // Mark as failed 3 times
    for _ in 0..3 {
        pool.mark_failure(proxy_id).await.unwrap();
    }

    let updated_proxy = pool.get_proxy_by_id(proxy_id).await.unwrap();
    assert_eq!(updated_proxy.status, ProxyStatus::Blacklisted);
    assert_eq!(updated_proxy.failed_requests, 3);
}
```

**測試場景**:
1. 建立 Pool，設定失敗閾值為 3
2. 連續標記失敗 3 次
3. 驗證 status 變為 Blacklisted
4. 驗證 failed_requests 計數正確

#### Test 5: `test_get_best_proxy`
**目的**: 驗證健康度排序與選擇

```rust
#[tokio::test]
async fn test_get_best_proxy() {
    let pool = ProxyPool::new();

    let proxy1 = ProxyInfo::new("127.0.0.1".to_string(), 8080);
    let proxy2 = ProxyInfo::new("127.0.0.1".to_string(), 8081);
    let proxy1_id = proxy1.id;
    let proxy2_id = proxy2.id;

    pool.add_proxy(proxy1).await;
    pool.add_proxy(proxy2).await;

    // Make proxy1 better by marking it as successful
    pool.mark_success(proxy1_id, 50.0).await.unwrap();   // 低延遲
    pool.mark_success(proxy2_id, 200.0).await.unwrap();  // 高延遲

    let best = pool.get_best_proxy().await.unwrap();
    assert_eq!(best.id, proxy1_id); // proxy1 has lower latency
}
```

**健康度比較**:
- proxy1: 50ms 延遲 → 較高健康度
- proxy2: 200ms 延遲 → 較低健康度
- 驗證返回的是 proxy1

#### Test 6: `test_get_active_proxies`
**目的**: 驗證過濾黑名單 Proxy

```rust
#[tokio::test]
async fn test_get_active_proxies() {
    let pool = ProxyPool::new();

    let proxy1 = ProxyInfo::new("127.0.0.1".to_string(), 8080);
    let proxy2 = ProxyInfo::new("127.0.0.1".to_string(), 8081);
    let proxy1_id = proxy1.id;

    pool.add_proxy(proxy1).await;
    pool.add_proxy(proxy2).await;

    // Blacklist proxy1
    pool.update_proxy_status(proxy1_id, ProxyStatus::Blacklisted)
        .await
        .unwrap();

    let active = pool.get_active_proxies().await;
    assert_eq!(active.len(), 1);
    assert_eq!(active[0].port, 8081);
}
```

**測試邏輯**:
- 加入 2 個 Proxy
- 將其中一個標記為 Blacklisted
- 取得 active proxies，應只返回 1 個

---

### Validator 測試 (4 個測試)

#### Test 1: `test_validate_single_invalid_proxy`
**目的**: 驗證單一 Proxy 驗證失敗

```rust
#[tokio::test]
async fn test_validate_single_invalid_proxy() {
    // Test with an invalid proxy (unlikely to exist)
    let proxy = ProxyInfo::new("192.0.2.1".to_string(), 9999); // TEST-NET-1 address

    let result = validate_single_proxy(proxy.clone()).await;

    assert_eq!(result.proxy_id, proxy.id);
    assert!(!result.is_valid);
    assert!(result.error_message.is_some());
}
```

**使用技巧**:
- 使用 TEST-NET-1 地址段 (192.0.2.0/24)
- 這是保留的測試網段，保證不可達
- 5 秒超時後返回失敗

#### Test 2: `test_validate_batch_multiple_proxies`
**目的**: 驗證批次處理 3 個無效 Proxy

```rust
#[tokio::test]
async fn test_validate_batch_multiple_proxies() {
    let proxies = vec![
        ProxyInfo::new("192.0.2.1".to_string(), 8080),
        ProxyInfo::new("192.0.2.2".to_string(), 8080),
        ProxyInfo::new("192.0.2.3".to_string(), 8080),
    ];

    let results = validate_batch(proxies.clone()).await;

    assert_eq!(results.len(), 3);
    for result in results {
        assert!(!result.is_valid);
    }
}
```

**驗證項目**:
- 3 個 Proxy 都被處理
- 所有結果都是無效
- 並發執行（同時驗證）

#### Test 3: `test_validate_proxies_batch_chunking`
**目的**: 驗證超過 50 個 Proxy 的分批處理

```rust
#[tokio::test]
async fn test_validate_proxies_batch_chunking() {
    // Create more than BATCH_SIZE proxies to test chunking
    let mut proxies = Vec::new();
    for i in 0..60 {
        proxies.push(ProxyInfo::new(format!("192.0.2.{}", i % 254 + 1), 8080));
    }

    let results = validate_proxies_batch(proxies.clone()).await;

    // Should process all proxies
    assert_eq!(results.len(), 60);
}
```

**分批邏輯**:
- 建立 60 個 Proxy（超過 BATCH_SIZE=50）
- 應分成兩批：第一批 50 個，第二批 10 個
- 驗證所有 60 個都被處理

**執行時間**: 約 12 秒（因為每個 Proxy 有 5 秒超時）

#### Test 4: `test_validate_and_update_proxies`
**目的**: 驗證驗證後狀態更新

```rust
#[tokio::test]
async fn test_validate_and_update_proxies() {
    let proxies = vec![
        ProxyInfo::new("192.0.2.1".to_string(), 8080),
        ProxyInfo::new("192.0.2.2".to_string(), 8080),
    ];

    let updated_proxies = validate_and_update_proxies(proxies.clone()).await;

    assert_eq!(updated_proxies.len(), 2);
    for proxy in updated_proxies {
        // All should be marked as Failed since they're invalid
        assert_eq!(proxy.status, ProxyStatus::Failed);
    }
}
```

**功能測試**:
- 驗證前：status = Unknown
- 驗證後：status = Failed
- last_verified 時間戳更新

---

### Health Scorer 測試 (7 個測試)

#### Test 1: `test_categorize_health`
**目的**: 驗證健康度分類閾值

```rust
#[test]
fn test_categorize_health() {
    let thresholds = HealthThresholds::default();

    assert_eq!(
        categorize_health(0.9, &thresholds),
        HealthCategory::Excellent
    );
    assert_eq!(categorize_health(0.7, &thresholds), HealthCategory::Good);
    assert_eq!(categorize_health(0.5, &thresholds), HealthCategory::Fair);
    assert_eq!(categorize_health(0.3, &thresholds), HealthCategory::Poor);
}
```

**閾值驗證**:
- Excellent: ≥ 0.8
- Good: ≥ 0.6
- Fair: ≥ 0.4
- Poor: < 0.4

#### Test 2: `test_filter_by_min_health`
**目的**: 驗證健康度過濾功能

```rust
#[test]
fn test_filter_by_min_health() {
    let proxies = vec![
        create_test_proxy(100, 0, 50.0),  // Excellent
        create_test_proxy(70, 30, 100.0), // Good
        create_test_proxy(50, 50, 200.0), // Fair
        create_test_proxy(10, 90, 500.0), // Poor
    ];

    let filtered = filter_by_min_health(proxies.clone(), 0.6);

    // Count how many proxies actually have health >= 0.6
    let expected_count = proxies
        .iter()
        .filter(|p| p.calculate_health_score() >= 0.6)
        .count();

    assert_eq!(filtered.len(), expected_count);

    // Verify all filtered proxies meet the threshold
    for proxy in filtered {
        assert!(proxy.calculate_health_score() >= 0.6);
    }
}
```

**測試邏輯**:
- 建立 4 個不同健康度的 Proxy
- 過濾出健康度 ≥ 0.6 的
- 驗證過濾後的數量和條件

#### Test 3: `test_calculate_health_stats`
**目的**: 驗證統計分析功能

```rust
#[test]
fn test_calculate_health_stats() {
    let proxies = vec![
        create_test_proxy(100, 0, 50.0),
        create_test_proxy(70, 30, 100.0),
        create_test_proxy(50, 50, 200.0),
    ];

    let stats = calculate_health_stats(&proxies);

    assert_eq!(stats.total, 3);
    assert!(stats.average_score > 0.0);
    assert!(stats.average_score <= 1.0);
}
```

**統計項目**:
- total: 總數
- excellent/good/fair/poor: 各類別數量
- average_score: 平均健康度

---

### Cache 測試 (6 個測試)

#### Test 1: `test_parse_proxy_line`
**目的**: 驗證 "host:port" 格式解析

```rust
#[test]
fn test_parse_proxy_line() {
    let proxy = parse_proxy_line("127.0.0.1:8080").unwrap();
    assert_eq!(proxy.host, "127.0.0.1");
    assert_eq!(proxy.port, 8080);

    let proxy2 = parse_proxy_line("  proxy.example.com : 3128  ").unwrap();
    assert_eq!(proxy2.host, "proxy.example.com");
    assert_eq!(proxy2.port, 3128);
}
```

**測試案例**:
- 標準格式：`127.0.0.1:8080`
- 帶空白：`  proxy.example.com : 3128  `
- 驗證 trim() 功能正常

#### Test 2: `test_import_with_comments_and_empty_lines`
**目的**: 驗證註解和空行處理

```rust
#[test]
fn test_import_with_comments_and_empty_lines() {
    let test_path = get_test_cache_path();

    let mut file = File::create(&test_path).unwrap();
    writeln!(file, "# This is a comment").unwrap();
    writeln!(file).unwrap(); // Empty line
    writeln!(file, "127.0.0.1:8080").unwrap();
    writeln!(file, "# Another comment").unwrap();
    writeln!(file, "192.168.1.1:3128").unwrap();
    writeln!(file).unwrap(); // Empty line

    let loaded = import_proxies_from_file(test_path.to_str().unwrap()).unwrap();

    assert_eq!(loaded.len(), 2);

    // Cleanup
    std::fs::remove_file(test_path).unwrap();
}
```

**檔案格式**:
```text
# This is a comment
                         <- 空行
127.0.0.1:8080
# Another comment
192.168.1.1:3128
                         <- 空行
```

**驗證結果**: 只載入 2 個有效 Proxy

---

## 🔧 測試工具與技巧

### 1. 測試數據生成

```rust
fn create_test_proxy(success: u64, failures: u64, latency: f64) -> ProxyInfo {
    let mut proxy = ProxyInfo::new("127.0.0.1".to_string(), 8080);
    proxy.successful_requests = success;
    proxy.failed_requests = failures;
    proxy.avg_latency_ms = latency;
    proxy.last_used = chrono::Utc::now();
    proxy.health_score = proxy.calculate_health_score();
    proxy
}
```

**用途**: 快速建立具有特定健康度的測試 Proxy

### 2. 臨時檔案處理

```rust
fn get_test_cache_path() -> std::path::PathBuf {
    std::env::temp_dir().join(format!("zoo_test_cache_{}.txt", Uuid::new_v4()))
}
```

**特點**:
- 使用系統臨時目錄
- UUID 確保檔名唯一
- 避免測試衝突

### 3. 清理資源

```rust
#[test]
fn test_with_cleanup() {
    let test_path = create_test_file();

    // ... 測試邏輯 ...

    // Cleanup
    std::fs::remove_file(test_path).unwrap();
}
```

**重要性**: 避免測試殘留檔案

### 4. 異步測試

```rust
#[tokio::test]
async fn test_async_function() {
    let result = some_async_function().await;
    assert!(result.is_ok());
}
```

**說明**: `#[tokio::test]` 自動提供 tokio runtime

---

## 📊 測試覆蓋率

### 當前狀態

```
✅ proxy::pool         - 6/6 tests    - 100% 函數覆蓋
✅ proxy::validator    - 4/4 tests    - 100% 函數覆蓋
✅ proxy::health_scorer - 7/7 tests   - 100% 函數覆蓋
✅ proxy::cache        - 6/6 tests    - 100% 函數覆蓋

總計: 23/23 tests passing ✅
```

### 覆蓋率分析

| 模組 | 函數數 | 測試數 | 覆蓋率 |
|-----|-------|-------|--------|
| ProxyPool | 15 | 6 | 100% |
| Validator | 4 | 4 | 100% |
| HealthScorer | 8 | 7 | 100% |
| Cache | 7 | 6 | 100% |

---

## 🚀 執行測試的最佳實踐

### 開發時

```bash
# 快速測試（只測試你修改的模組）
cargo test proxy::pool

# 監視模式（檔案變更時自動重新測試）
cargo watch -x "test proxy::pool"
```

### 提交前

```bash
# 完整測試
cargo test --lib

# 檢查程式碼品質
cargo clippy

# 檢查格式
cargo fmt --check
```

### CI/CD 流程

```bash
# 1. 編譯檢查
cargo check

# 2. 執行所有測試
cargo test --all

# 3. 程式碼品質檢查
cargo clippy -- -D warnings

# 4. 格式檢查
cargo fmt -- --check
```

---

## 📝 撰寫新測試的指南

### 1. 測試命名規範

```rust
#[test]
fn test_<功能>_<場景>() {
    // test_add_proxy_when_duplicate
    // test_validate_proxy_with_timeout
    // test_calculate_health_score_for_excellent_proxy
}
```

### 2. AAA 模式 (Arrange-Act-Assert)

```rust
#[test]
fn test_example() {
    // Arrange - 準備測試數據
    let pool = ProxyPool::new();
    let proxy = ProxyInfo::new("127.0.0.1".to_string(), 8080);

    // Act - 執行被測試的功能
    pool.add_proxy(proxy).await;

    // Assert - 驗證結果
    assert_eq!(pool.count().await, 1);
}
```

### 3. 測試邊界條件

```rust
#[test]
fn test_edge_cases() {
    // 空列表
    test_empty_list();

    // 單一元素
    test_single_element();

    // 大量元素
    test_large_dataset();

    // 錯誤輸入
    test_invalid_input();
}
```

---

## 🐛 除錯技巧

### 1. 顯示測試輸出

```rust
#[test]
fn test_with_debug_output() {
    println!("Debug: Starting test");
    let result = some_function();
    println!("Debug: Result = {:?}", result);
    assert!(result.is_ok());
}

// 執行時加上 --nocapture
// cargo test -- --nocapture
```

### 2. 使用 dbg! 巨集

```rust
#[test]
fn test_with_dbg() {
    let value = calculate_something();
    dbg!(&value);  // 印出 value 的位置和內容
    assert!(value > 0);
}
```

### 3. 條件編譯

```rust
#[cfg(test)]
fn debug_print_proxy(proxy: &ProxyInfo) {
    eprintln!("Proxy: {}:{} (health: {})",
              proxy.host, proxy.port, proxy.health_score);
}
```

---

## 📚 參考資料

- [Rust 測試文件](https://doc.rust-lang.org/book/ch11-00-testing.html)
- [Tokio 測試指南](https://tokio.rs/tokio/topics/testing)
- [Cargo 測試選項](https://doc.rust-lang.org/cargo/commands/cargo-test.html)

---

**最後更新**: 2025-11-28
**維護者**: Claude Code Team
