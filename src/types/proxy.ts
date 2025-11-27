// Proxy-related TypeScript types

export interface ProxyInfo {
  id: string;
  host: string;
  port: number;
  proxyType: ProxyType;
  status: ProxyStatus;
  avgLatencyMs: number;
  successfulRequests: number;
  failedRequests: number;
  lastVerified: string;  // ISO 8601
  lastUsed: string;  // ISO 8601
  healthScore: number;  // 0.0 - 1.0
}

export enum ProxyType {
  Http = 'Http',
}

export enum ProxyStatus {
  Unknown = 'Unknown',
  Verifying = 'Verifying',
  Active = 'Active',
  Failed = 'Failed',
  Blacklisted = 'Blacklisted',
}
