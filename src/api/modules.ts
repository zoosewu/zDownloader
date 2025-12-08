// Module Management API Client (Phase 6 & 7)
// Tauri command wrappers for module system

import { invoke } from '@tauri-apps/api/core';
import type {
  ModuleMetadata,
  ModuleDetail,
  CredentialStatus,
  UrlDetectionResult,
  AuditLogsResponse,
  ScanResult,
} from '../types/module';

/**
 * T108: Save module credentials
 */
export async function saveModuleCredentials(
  moduleId: string,
  credentials: Record<string, string>
): Promise<void> {
  await invoke('save_module_credentials', {
    moduleId,
    credentials,
  });
}

/**
 * T109: Get module credentials status
 */
export async function getModuleCredentialsStatus(
  moduleId: string,
  requiredKeys: string[]
): Promise<CredentialStatus> {
  return await invoke('get_module_credentials_status', {
    moduleId,
    requiredKeys,
  });
}

/**
 * T110: Delete module credentials
 */
export async function deleteModuleCredentials(moduleId: string): Promise<void> {
  await invoke('delete_module_credentials', {
    moduleId,
  });
}

/**
 * T121: Get all modules
 */
export async function getAllModules(): Promise<ModuleMetadata[]> {
  return await invoke('get_all_modules');
}

/**
 * T122: Get module detail
 */
export async function getModuleDetail(moduleId: string): Promise<ModuleDetail> {
  return await invoke('get_module_detail', {
    moduleId,
  });
}

/**
 * T123: Reload module
 */
export async function reloadModule(moduleId: string): Promise<void> {
  await invoke('reload_module', {
    moduleId,
  });
}

/**
 * T124: Unload module
 */
export async function unloadModule(moduleId: string): Promise<void> {
  await invoke('unload_module', {
    moduleId,
  });
}

/**
 * T125: Scan modules
 */
export async function scanModules(): Promise<ScanResult> {
  return await invoke('scan_modules');
}

/**
 * T126: Get module audit logs
 */
export async function getModuleAuditLogs(
  moduleId: string,
  limit?: number,
  offset?: number
): Promise<AuditLogsResponse> {
  return await invoke('get_module_audit_logs', {
    moduleId,
    limit,
    offset,
  });
}

/**
 * T144: Detect URL module (Phase 8)
 */
export async function detectUrlModule(url: string): Promise<UrlDetectionResult> {
  return await invoke('detect_url_module', {
    url,
  });
}
