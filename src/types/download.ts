// Download-related TypeScript types

export interface DownloadSegment {
  id: string;
  taskId: string;
  segmentId: number;
  startByte: number;
  endByte: number;
  downloadedBytes: number;
  status: SegmentStatus;
  threadId?: number;
  proxyId?: string;
  retryCount: number;
}

export enum SegmentStatus {
  Pending = 'Pending',
  Downloading = 'Downloading',
  Completed = 'Completed',
  Failed = 'Failed',
}

export interface DownloadTask {
  id: string;
  url: string;
  filename: string;
  filepath: string;
  filesize: number;
  status: DownloadStatus;
  progress: number;  // 0.0 - 100.0
  currentSpeed: number;  // bytes/sec
  segments: DownloadSegment[];  // Download segments for multi-threaded downloads
  createdAt: string;  // ISO 8601
  startedAt?: string;  // ISO 8601
  completedAt?: string;  // ISO 8601
  errorMessage?: string;
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

export interface UnfinishedDownloadsDetectedEvent {
  tasks: DownloadTask[];
}
