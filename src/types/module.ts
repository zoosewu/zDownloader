// Module-related TypeScript type definitions (Feature 004)

/**
 * Module metadata DTO (matches backend ModuleMetadataDto)
 */
export interface ModuleMetadata {
  id: string;
  name: string;
  version: string;
  author: string;
  description: string;
  homepage?: string;
  urlPatterns: string[];
  status: ModuleStatus;
  errorMessage?: string;
  loadTime: string; // ISO 8601
  permissions: ModulePermissions;
}

/**
 * Module lifecycle status
 */
export type ModuleStatus =
  | 'pending'
  | 'loading'
  | 'active'
  | 'failed'
  | 'suspended'
  | 'unloaded';

/**
 * Module permissions structure
 */
export interface ModulePermissions {
  network: NetworkPermission;
  credentials?: CredentialPermission;
}

/**
 * Network access permissions
 */
export interface NetworkPermission {
  allowDomains: string[];
}

/**
 * Credential access permissions
 */
export interface CredentialPermission {
  required: boolean;
  credentialKeys: string[];
}

/**
 * Module detail DTO (extends ModuleMetadata with statistics)
 */
export interface ModuleDetail extends ModuleMetadata {
  statistics: ModuleStatistics;
  securityEvents: SecurityEventsSummary;
}

/**
 * Module usage statistics
 */
export interface ModuleStatistics {
  totalDownloads: number;
  successfulDownloads: number;
  failedDownloads: number;
  lastUsed?: string; // ISO 8601
}

/**
 * Security events summary
 */
export interface SecurityEventsSummary {
  totalViolations: number;
  lastViolation?: string; // ISO 8601
}

/**
 * URL detection result DTO
 */
export interface UrlDetectionResult {
  url: string;
  moduleId?: string;
  moduleName?: string;
  isSupported: boolean;
  requiresAuth: boolean;
}

/**
 * Credential status DTO
 */
export interface CredentialStatus {
  moduleId: string;
  required: boolean;
  configured: boolean;
  credentialKeys: string[];
  configuredKeys: string[];
  missingKeys: string[];
}

/**
 * Audit log entry DTO
 */
export interface AuditLogEntry {
  timestamp: string; // ISO 8601
  moduleId: string;
  eventType: string;
  severity: 'INFO' | 'WARN' | 'ERROR' | 'CRITICAL';
  details: any; // JSON object
}

/**
 * Audit logs response DTO
 */
export interface AuditLogsResponse {
  total: number;
  logs: AuditLogEntry[];
}

/**
 * Scan result DTO
 */
export interface ScanResult {
  totalFound: number;
  successfullyLoaded: number;
  failed: Array<{
    moduleId: string;
    error: string;
  }>;
}
