// Proxy-related TypeScript types

export interface ProxyInfo {
  id: string;
  host: string;
  port: number;
  status: ProxyStatus;
  health_score: number;          // 0-100
  successful_requests: number;
  failed_requests: number;
  avg_latency_ms: number;
  last_used: string | null;      // ISO 8601
  last_failure: string | null;   // ISO 8601
}

export enum ProxyStatus {
  Active = 'Active',
  Failed = 'Failed',
  Blacklisted = 'Blacklisted',
}
