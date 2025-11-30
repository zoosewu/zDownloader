// Scheduler API - Tauri command wrappers for download scheduling and retry
import { invoke } from '@tauri-apps/api/core';
import { DownloadTask } from '../types/download';

/**
 * Schedule a download to start at a specific time
 * @param taskId - ID of the download task
 * @param scheduledTime - ISO 8601 datetime string when to start
 */
export async function scheduleDownload(taskId: string, scheduledTime: string): Promise<void> {
  await invoke('schedule_download', { taskId, scheduledTime });
}

/**
 * Cancel a scheduled download
 * @param taskId - ID of the scheduled download task
 * @returns true if cancelled, false if not found
 */
export async function cancelScheduledDownload(taskId: string): Promise<boolean> {
  return await invoke<boolean>('cancel_scheduled_download', { taskId });
}

/**
 * Get all scheduled downloads
 * @returns List of all scheduled download tasks
 */
export async function getScheduledDownloads(): Promise<DownloadTask[]> {
  return await invoke<DownloadTask[]>('get_scheduled_downloads');
}

/**
 * Retry a failed download
 * @param taskId - ID of the failed download task
 */
export async function retryDownload(taskId: string): Promise<void> {
  await invoke('retry_download', { taskId });
}
