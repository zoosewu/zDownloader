// Download calculations utility - Time estimation and data formatting
import type { Task } from '../backend';

/** Percent complete (0-100), or null when the size is unknown. */
export function progressPercent(task: Task): number | null {
  if (task.state === 'completed') return 100;
  if (!task.total) return null;
  return Math.min(100, (task.downloaded / task.total) * 100);
}

/** Seconds left at the current speed, or null when it cannot be estimated. */
export function calculateRemainingTime(task: Task): number | null {
  if (task.state !== 'running' || task.speed <= 0 || task.total === null) return null;
  const seconds = (task.total - task.downloaded) / task.speed;
  return isFinite(seconds) && seconds >= 0 ? seconds : null;
}

/** e.g. "2分30秒", "1小時15分"; "--" when unknown. */
export function formatRemainingTime(seconds: number | null): string {
  if (seconds === null || !isFinite(seconds) || seconds < 0) {
    return '--';
  }
  const hours = Math.floor(seconds / 3600);
  const minutes = Math.floor((seconds % 3600) / 60);
  const secs = Math.floor(seconds % 60);
  if (hours > 0) {
    return `${hours}小時${minutes}分`;
  } else if (minutes > 0) {
    return `${minutes}分${secs}秒`;
  }
  return `${secs}秒`;
}

/** e.g. "1.50 GB", "230.00 MB". */
export function formatBytes(bytes: number): string {
  if (bytes <= 0) return '0 B';
  const k = 1024;
  const sizes = ['B', 'KB', 'MB', 'GB', 'TB'];
  const i = Math.min(sizes.length - 1, Math.floor(Math.log(bytes) / Math.log(k)));
  return `${(bytes / Math.pow(k, i)).toFixed(2)} ${sizes[i]}`;
}

export function formatSpeed(bytesPerSecond: number): string {
  return `${formatBytes(bytesPerSecond)}/s`;
}

export interface TableDisplayData {
  id: string;
  filename: string;
  filesizeFormatted: string;
  /** 0-100; 0 when the size is unknown. */
  progress: number;
  progressFormatted: string;
  speedFormatted: string;
  statusLabel: string;
  remainingTimeFormatted: string;
}

export function formatTaskForTable(task: Task, t: (key: string) => string): TableDisplayData {
  const percent = progressPercent(task);
  return {
    id: task.id,
    filename: task.name,
    filesizeFormatted: task.total !== null ? formatBytes(task.total) : '--',
    progress: percent ?? 0,
    progressFormatted: percent !== null ? `${percent.toFixed(1)}%` : formatBytes(task.downloaded),
    speedFormatted: task.state === 'running' && task.speed > 0 ? formatSpeed(task.speed) : '--',
    statusLabel: t(`downloads.state.${task.state}`),
    remainingTimeFormatted: formatRemainingTime(calculateRemainingTime(task)),
  };
}
