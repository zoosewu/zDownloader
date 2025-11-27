// Download-related TypeScript types

export interface DownloadTask {
  id: string;
  url: string;
  filename: string;
  filepath: string;
  filesize: number;
  status: DownloadStatus;
  progress: number;  // 0.0 - 100.0
  currentSpeed: number;  // bytes/sec
  createdAt: string;  // ISO 8601
  completedAt?: string;  // ISO 8601
}

export enum DownloadStatus {
  Pending = 'Pending',
  Initializing = 'Initializing',
  Downloading = 'Downloading',
  Paused = 'Paused',
  Cancelling = 'Cancelling',
  Cancelled = 'Cancelled',
  Completed = 'Completed',
  Failed = 'Failed',
  VerifyingIntegrity = 'VerifyingIntegrity',
  MergingSegments = 'MergingSegments',
}

export interface DownloadProgress {
  progress: number;
  speed: number;
  etaSeconds: number;
  downloadedBytes: number;
  totalBytes: number;
}

export interface ProgressUpdateEvent {
  taskId: string;
  progress: number;
  speed: number;
  etaSeconds: number;
}

export interface StatusChangeEvent {
  taskId: string;
  oldStatus: DownloadStatus;
  newStatus: DownloadStatus;
}

export interface ClipboardDetectedEvent {
  url: string;
  timestamp: number;
}
