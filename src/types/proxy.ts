// Proxy-related TypeScript types

// Mirrors src-tauri/src/models/proxy_info.rs (camelCase via serde)
export interface ProxyInfo {
  id: string;
  host: string;
  port: number;
  proxyType: 'http';
  status: ProxyStatus;
  healthScore: number;           // 0.0-1.0
  successfulRequests: number;
  failedRequests: number;
  avgLatencyMs: number;
  lastVerified: string;          // ISO 8601
  lastUsed: string;              // ISO 8601
}

export enum ProxyStatus {
  Unknown = 'unknown',
  Verifying = 'verifying',
  Active = 'active',
  Failed = 'failed',
  Blacklisted = 'blacklisted',
}
