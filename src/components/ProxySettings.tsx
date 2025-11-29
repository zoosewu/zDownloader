// Proxy Settings Component - Manage proxy list
import { useEffect } from 'react';
import { useAtom, useAtomValue, useSetAtom } from 'jotai';
import { open } from '@tauri-apps/plugin-dialog';
import {
  proxyListAtom,
  // activeProxiesAtom, // Currently unused
  proxyStatsAtom,
  proxyLoadingAtom,
  proxyErrorAtom,
  loadProxyListAtom,
  importProxyListAtom,
  removeProxyAtom,
  clearAllProxiesAtom,
  resetFailedProxiesAtom,
} from '../stores/proxyStore';
import { ProxyStatus } from '../types/proxy';
import './ProxySettings.css';

export default function ProxySettings() {
  const [proxies] = useAtom(proxyListAtom);
  // const activeProxies = useAtomValue(activeProxiesAtom);
  const stats = useAtomValue(proxyStatsAtom);
  const loading = useAtomValue(proxyLoadingAtom);
  const error = useAtomValue(proxyErrorAtom);

  const loadProxyList = useSetAtom(loadProxyListAtom);
  const importProxyList = useSetAtom(importProxyListAtom);
  const removeProxy = useSetAtom(removeProxyAtom);
  const clearAllProxies = useSetAtom(clearAllProxiesAtom);
  const resetFailedProxies = useSetAtom(resetFailedProxiesAtom);

  // Load proxy list on mount
  useEffect(() => {
    loadProxyList();
  }, [loadProxyList]);

  // Import proxy list from file
  const handleImport = async () => {
    try {
      const file = await open({
        filters: [
          {
            name: 'Text Files',
            extensions: ['txt'],
          },
        ],
      });

      if (file) {
        const filePath = typeof file === 'string' ? file : file;
        const count = await importProxyList(filePath);
        alert(`成功匯入 ${count} 個 Proxy`);
      }
    } catch (error) {
      console.error('Import failed:', error);
      alert(`匯入失敗: ${error}`);
    }
  };

  // Remove a proxy
  const handleRemove = async (proxyId: string) => {
    if (confirm('確定要移除此 Proxy？')) {
      try {
        await removeProxy(proxyId);
      } catch (error) {
        console.error('Remove failed:', error);
        alert(`移除失敗: ${error}`);
      }
    }
  };

  // Clear all proxies
  const handleClearAll = async () => {
    if (confirm(`確定要清空所有 ${proxies.length} 個 Proxy？此操作無法撤銷。`)) {
      try {
        await clearAllProxies();
      } catch (error) {
        console.error('Clear failed:', error);
        alert(`清空失敗: ${error}`);
      }
    }
  };

  // Reset failed proxies
  const handleReset = async () => {
    if (confirm('確定要重置所有失敗的 Proxy？')) {
      try {
        await resetFailedProxies();
      } catch (error) {
        console.error('Reset failed:', error);
        alert(`重置失敗: ${error}`);
      }
    }
  };

  // Get status badge class
  const getStatusClass = (status: ProxyStatus): string => {
    switch (status) {
      case ProxyStatus.Active:
        return 'status-badge status-active';
      case ProxyStatus.Failed:
        return 'status-badge status-failed';
      case ProxyStatus.Blacklisted:
        return 'status-badge status-blacklisted';
      default:
        return 'status-badge';
    }
  };

  // Get health score color
  const getHealthColor = (score: number): string => {
    if (score >= 80) return '#4caf50';
    if (score >= 60) return '#ff9800';
    return '#f44336';
  };

  // Format timestamp
  const formatTime = (timestamp: string | null): string => {
    if (!timestamp) return '-';
    const date = new Date(timestamp);
    return date.toLocaleString('zh-TW', {
      month: '2-digit',
      day: '2-digit',
      hour: '2-digit',
      minute: '2-digit',
    });
  };

  return (
    <div className="proxy-settings">
      <div className="proxy-header">
        <h2>Proxy 管理</h2>

        {/* Statistics */}
        <div className="proxy-stats">
          <div className="stat-item">
            <span className="stat-label">總數</span>
            <span className="stat-value">{stats.total}</span>
          </div>
          <div className="stat-item">
            <span className="stat-label">活躍</span>
            <span className="stat-value stat-active">{stats.active}</span>
          </div>
          <div className="stat-item">
            <span className="stat-label">失敗</span>
            <span className="stat-value stat-failed">{stats.failed}</span>
          </div>
          <div className="stat-item">
            <span className="stat-label">黑名單</span>
            <span className="stat-value stat-blacklisted">
              {stats.blacklisted}
            </span>
          </div>
          <div className="stat-item">
            <span className="stat-label">平均健康度</span>
            <span className="stat-value">{stats.avgHealth.toFixed(1)}</span>
          </div>
        </div>

        {/* Toolbar */}
        <div className="proxy-toolbar">
          <button onClick={handleImport} disabled={loading} className="btn-primary">
            📥 匯入 Proxy
          </button>
          <button onClick={() => loadProxyList()} disabled={loading}>
            🔄 重新載入
          </button>
          <button onClick={handleReset} disabled={loading || stats.failed === 0}>
            ♻️ 重置失敗
          </button>
          <button
            onClick={handleClearAll}
            disabled={loading || proxies.length === 0}
            className="btn-danger"
          >
            🗑️ 清空全部
          </button>
        </div>
      </div>

      {/* Error Message */}
      {error && <div className="error-message">❌ {error}</div>}

      {/* Loading State */}
      {loading && <div className="loading-message">載入中...</div>}

      {/* Proxy Table */}
      {!loading && proxies.length === 0 ? (
        <div className="empty-state">
          <p>📝 尚無 Proxy</p>
          <p className="empty-hint">點擊「匯入 Proxy」從檔案載入</p>
        </div>
      ) : (
        <div className="proxy-table-container">
          <table className="proxy-table">
            <thead>
              <tr>
                <th>地址</th>
                <th>狀態</th>
                <th>健康度</th>
                <th>成功/失敗</th>
                <th>平均延遲</th>
                <th>最後使用</th>
                <th>操作</th>
              </tr>
            </thead>
            <tbody>
              {proxies.map((proxy) => (
                <tr key={proxy.id}>
                  <td className="proxy-address">
                    {proxy.host}:{proxy.port}
                  </td>
                  <td>
                    <span className={getStatusClass(proxy.status)}>
                      {proxy.status}
                    </span>
                  </td>
                  <td>
                    <div className="health-cell">
                      <div
                        className="health-bar"
                        style={{
                          width: `${proxy.health_score}%`,
                          backgroundColor: getHealthColor(proxy.health_score),
                        }}
                      />
                      <span className="health-value">
                        {proxy.health_score.toFixed(0)}
                      </span>
                    </div>
                  </td>
                  <td className="requests-cell">
                    <span className="success-count">
                      {proxy.successful_requests}
                    </span>
                    {' / '}
                    <span className="failed-count">{proxy.failed_requests}</span>
                  </td>
                  <td>{proxy.avg_latency_ms.toFixed(0)} ms</td>
                  <td className="time-cell">{formatTime(proxy.last_used)}</td>
                  <td>
                    <button
                      onClick={() => handleRemove(proxy.id)}
                      disabled={loading}
                      className="btn-remove"
                    >
                      移除
                    </button>
                  </td>
                </tr>
              ))}
            </tbody>
          </table>
        </div>
      )}
    </div>
  );
}
