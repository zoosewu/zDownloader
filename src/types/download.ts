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

// Values must match Rust's #[serde(rename_all = "camelCase")] output
export enum SegmentStatus {
  Pending = 'pending',
  Downloading = 'downloading',
  Completed = 'completed',
  Failed = 'failed',
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
  // Scheduling fields
  scheduledStart?: string;  // ISO 8601 - when to automatically start
  // Retry fields
  retryCount: number;  // Current retry attempt (0 = first attempt)
  maxRetries?: number;  // Override max retries from settings
  lastRetryAt?: string;  // ISO 8601 - last retry timestamp
  // Speed priority field
  speedPriority?: number;  // Speed priority (default: 10, 0 = lowest, higher = more priority)
}

// Values must match Rust's #[serde(rename_all = "camelCase")] output
export enum DownloadStatus {
  Pending = 'pending',
  Initializing = 'initializing',
  Downloading = 'downloading',
  Paused = 'paused',
  Cancelling = 'cancelling',
  Cancelled = 'cancelled',
  Completed = 'completed',
  Failed = 'failed',
  VerifyingIntegrity = 'verifyingIntegrity',
  MergingSegments = 'mergingSegments',
}
