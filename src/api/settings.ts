// Settings API - Tauri command wrappers for settings management
import { invoke } from '@tauri-apps/api/core';
import { AppSettings } from '../types/settings';

/**
 * Get current application settings
 */
export async function getSettings(): Promise<AppSettings> {
  return await invoke<AppSettings>('get_settings');
}

/**
 * Update application settings
 * @param settings - New settings to apply
 */
export async function updateSettings(settings: AppSettings): Promise<AppSettings> {
  return await invoke<AppSettings>('update_settings', { newSettings: settings });
}

/**
 * Export settings to a file
 * @param exportPath - Full path to export file
 */
export async function exportSettings(exportPath: string): Promise<void> {
  await invoke('export_settings', { exportPath });
}

/**
 * Import settings from a file
 * @param importPath - Full path to import file
 */
export async function importSettings(importPath: string): Promise<AppSettings> {
  return await invoke<AppSettings>('import_settings', { importPath });
}

/**
 * Reset settings to default values
 */
export async function resetSettings(): Promise<AppSettings> {
  return await invoke<AppSettings>('reset_settings');
}
