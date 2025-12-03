// Download calculations utility - Time estimation and data formatting
import { DownloadTask } from '../types/download';

/**
 * Calculate remaining time for a download task (in seconds)
 * @param task Download task
 * @returns Remaining seconds, or null if calculation not possible
 */
export function calculateRemainingTime(task: DownloadTask): number | null {
  // If speed is 0 or task is not downloading, return null
  if (task.currentSpeed === 0 || task.status !== 'Downloading') {
    return null;
  }

  const downloadedBytes = Math.floor((task.progress / 100) * task.filesize);
  const remainingBytes = task.filesize - downloadedBytes;

  // Calculate remaining time using current speed
  const remainingSeconds = remainingBytes / task.currentSpeed;

  // Return null for invalid calculations
  if (!isFinite(remainingSeconds) || remainingSeconds < 0) {
    return null;
  }

  return remainingSeconds;
}

/**
 * Format remaining time to human-readable format
 * @param seconds Remaining seconds
 * @returns Formatted string (e.g., "2分30秒", "1小時15分")
 */
export function formatRemainingTime(seconds: number | null): string {
  if (seconds === null || !isFinite(seconds)) {
    return '--';
  }

  if (seconds < 0) {
    return '計算中';
  }

  const hours = Math.floor(seconds / 3600);
  const minutes = Math.floor((seconds % 3600) / 60);
  const secs = Math.floor(seconds % 60);

  if (hours > 0) {
    return `${hours}小時${minutes}分`;
  } else if (minutes > 0) {
    return `${minutes}分${secs}秒`;
  } else {
    return `${secs}秒`;
  }
}

/**
 * Format bytes to human-readable format
 * @param bytes Number of bytes
 * @returns Formatted string (e.g., "1.5 GB", "230 MB")
 */
export function formatBytes(bytes: number): string {
  if (bytes === 0) return '0 B';
  const k = 1024;
  const sizes = ['B', 'KB', 'MB', 'GB', 'TB'];
  const i = Math.floor(Math.log(bytes) / Math.log(k));
  return `${(bytes / Math.pow(k, i)).toFixed(2)} ${sizes[i]}`;
}

/**
 * Format speed to human-readable format
 * @param bytesPerSecond Speed in bytes per second
 * @returns Formatted string (e.g., "2.3 MB/s")
 */
export function formatSpeed(bytesPerSecond: number): string {
  return `${formatBytes(bytesPerSecond)}/s`;
}

/**
 * Table display data interface
 */
export interface TableDisplayData {
  id: string;
  filename: string;
  filesizeFormatted: string;
  progress: number;
  progressFormatted: string;
  speed: number;
  speedFormatted: string;
  status: string;
  statusLabel: string;
  createdAt: string;
  createdAtFormatted: string;
  remainingTime: number | null;
  remainingTimeFormatted: string;
}

/**
 * Format download task for table display
 * @param task Download task
 * @param t Translation function
 * @returns Formatted display data
 */
export function formatTaskForTable(
  task: DownloadTask,
  t: (key: string) => string
): TableDisplayData {
  const remainingTime = calculateRemainingTime(task);

  return {
    id: task.id,
    filename: task.filename,
    filesizeFormatted: formatBytes(task.filesize),
    progress: task.progress,
    progressFormatted: `${task.progress.toFixed(1)}%`,
    speed: task.currentSpeed,
    speedFormatted:
      task.status === 'Downloading' && task.currentSpeed > 0
        ? formatSpeed(task.currentSpeed)
        : '--',
    status: task.status,
    statusLabel: t(`downloads.status.${task.status.toLowerCase()}`),
    createdAt: task.createdAt,
    createdAtFormatted: '', // Will be filled by formatRelativeTime
    remainingTime,
    remainingTimeFormatted: formatRemainingTime(remainingTime),
  };
}

/**
 * Segment visualization data interface
 */
export interface SegmentVisualizationData {
  segmentId: number;
  status: string;
  color: string;
  tooltipData: {
    segmentId: number;
    sizeRange: string;
    downloaded: string;
    progress: string;
    status: string;
    retryCount: number;
    proxy: string | null;
  };
}

/**
 * Get color for segment status
 * @param status Segment status
 * @returns Color hex code
 */
export function getSegmentColor(status: string): string {
  switch (status) {
    case 'Pending':
      return '#9e9e9e'; // Grey
    case 'Downloading':
      return '#2196f3'; // Blue
    case 'Completed':
      return '#4caf50'; // Green
    case 'Failed':
      return '#f44336'; // Red
    default:
      return '#9e9e9e';
  }
}

/**
 * Format segment for visualization
 * @param segment Download segment
 * @param t Translation function
 * @returns Formatted visualization data
 */
export function formatSegmentForVisualization(
  segment: any,
  t: (key: string) => string
): SegmentVisualizationData {
  const segmentSize = segment.endByte - segment.startByte + 1;
  const progress = (segment.downloadedBytes / segmentSize) * 100;

  return {
    segmentId: segment.segmentId,
    status: segment.status,
    color: getSegmentColor(segment.status),
    tooltipData: {
      segmentId: segment.segmentId,
      sizeRange: `${formatBytes(segment.startByte)} - ${formatBytes(segment.endByte)}`,
      downloaded: formatBytes(segment.downloadedBytes),
      progress: `${progress.toFixed(1)}%`,
      status: t(`downloads.status.${segment.status.toLowerCase()}`),
      retryCount: segment.retryCount,
      proxy: segment.proxy,
    },
  };
}
