// Mock data generator for development mode
import { DownloadTask, DownloadSegment, DownloadStatus, SegmentStatus } from '../types/download';

/**
 * Generate mock download segments for testing
 */
function generateMockSegments(taskId: string, count: number, status: SegmentStatus): DownloadSegment[] {
  const segments: DownloadSegment[] = [];
  const segmentSize = 1024 * 1024 * 10; // 10MB per segment

  for (let i = 0; i < count; i++) {
    const progress = status === SegmentStatus.Completed ? 100 :
                     status === SegmentStatus.Downloading ? Math.random() * 100 :
                     status === SegmentStatus.Failed ? Math.random() * 50 : 0;

    segments.push({
      id: `${taskId}-segment-${i}`,
      taskId,
      segmentId: i,
      startByte: i * segmentSize,
      endByte: (i + 1) * segmentSize,
      downloadedBytes: (progress / 100) * segmentSize,
      status,
      retryCount: status === SegmentStatus.Failed ? Math.floor(Math.random() * 3) : 0,
      proxyId: i % 3 === 0 ? `proxy${i % 5 + 1}` : undefined,
    });
  }

  return segments;
}

/**
 * Generate a single mock download task
 */
export function generateMockTask(
  id: string,
  status: DownloadStatus,
  options: {
    filename?: string;
    filesize?: number;
    progress?: number;
    speed?: number;
    segmentCount?: number;
    errorMessage?: string;
  } = {}
): DownloadTask {
  const now = new Date();
  const createdAt = new Date(now.getTime() - Math.random() * 86400000 * 7); // Random time in past week
  const filesize = options.filesize || Math.floor(Math.random() * 1024 * 1024 * 1024); // Random size up to 1GB
  const progress = options.progress ?? (status === DownloadStatus.Completed ? 100 : Math.random() * 100);
  const segmentCount = options.segmentCount || Math.floor(Math.random() * 10) + 1;

  // Determine segment status based on task status
  let segmentStatus: SegmentStatus = SegmentStatus.Pending;
  if (status === DownloadStatus.Downloading) segmentStatus = SegmentStatus.Downloading;
  if (status === DownloadStatus.Completed) segmentStatus = SegmentStatus.Completed;
  if (status === DownloadStatus.Failed) segmentStatus = SegmentStatus.Failed;

  const task: DownloadTask = {
    id,
    url: `https://example.com/files/${options.filename || `file-${id}.zip`}`,
    filename: options.filename || `mock-download-${id}.zip`,
    filepath: `/downloads/${options.filename || `mock-download-${id}.zip`}`,
    filesize,
    progress,
    currentSpeed: options.speed ?? (status === DownloadStatus.Downloading ? Math.random() * 10 * 1024 * 1024 : 0), // Random speed up to 10MB/s
    status,
    createdAt: createdAt.toISOString(),
    startedAt: status !== DownloadStatus.Pending ? new Date(createdAt.getTime() + 5000).toISOString() : undefined,
    completedAt: status === DownloadStatus.Completed ? new Date(now.getTime() - Math.random() * 3600000).toISOString() : undefined,
    segments: generateMockSegments(id, segmentCount, segmentStatus),
    errorMessage: options.errorMessage,
    retryCount: status === DownloadStatus.Failed ? Math.floor(Math.random() * 3) : 0,
    maxRetries: 3,
    lastRetryAt: status === DownloadStatus.Failed ? new Date(now.getTime() - Math.random() * 300000).toISOString() : undefined,
    scheduledStart: undefined,
  };

  return task;
}

/**
 * Generate a collection of mock download tasks for testing
 */
export function generateMockTasks(): DownloadTask[] {
  const tasks: DownloadTask[] = [];

  // Active downloads with various speeds
  tasks.push(generateMockTask('mock-1', DownloadStatus.Downloading, {
    filename: 'ubuntu-24.04-desktop-amd64.iso',
    filesize: 5_500_000_000, // 5.5 GB
    progress: 45.5,
    speed: 8.5 * 1024 * 1024, // 8.5 MB/s
    segmentCount: 8,
  }));

  tasks.push(generateMockTask('mock-2', DownloadStatus.Downloading, {
    filename: 'nodejs-v22.0.0-win-x64.zip',
    filesize: 45_000_000, // 45 MB
    progress: 78.2,
    speed: 2.3 * 1024 * 1024, // 2.3 MB/s
    segmentCount: 4,
  }));

  // Paused downloads
  tasks.push(generateMockTask('mock-3', DownloadStatus.Paused, {
    filename: 'python-3.12.1-amd64.exe',
    filesize: 28_000_000, // 28 MB
    progress: 33.8,
    speed: 0,
    segmentCount: 3,
  }));

  // Completed downloads
  tasks.push(generateMockTask('mock-4', DownloadStatus.Completed, {
    filename: 'visual-studio-code-1.95.0.dmg',
    filesize: 156_000_000, // 156 MB
    progress: 100,
    speed: 0,
    segmentCount: 6,
  }));

  tasks.push(generateMockTask('mock-5', DownloadStatus.Completed, {
    filename: 'docker-desktop-4.28.0-x86_64.pkg',
    filesize: 812_000_000, // 812 MB
    progress: 100,
    speed: 0,
    segmentCount: 10,
  }));

  // Failed download
  tasks.push(generateMockTask('mock-6', DownloadStatus.Failed, {
    filename: 'large-dataset-v2.tar.gz',
    filesize: 2_400_000_000, // 2.4 GB
    progress: 67.3,
    speed: 0,
    segmentCount: 12,
    errorMessage: 'Network error: Connection timeout after 30 seconds',
  }));

  // Pending downloads
  tasks.push(generateMockTask('mock-7', DownloadStatus.Pending, {
    filename: 'minecraft-server-1.20.4.jar',
    filesize: 52_000_000, // 52 MB
    progress: 0,
    speed: 0,
    segmentCount: 5,
  }));

  // Download with very long URL and filename for testing CollapsibleField
  tasks.push(generateMockTask('mock-8', DownloadStatus.Downloading, {
    filename: 'very-long-filename-that-will-test-the-collapsible-field-component-functionality-in-the-basic-info-tab-v1.2.3-final-release.zip',
    filesize: 123_456_789,
    progress: 12.5,
    speed: 1.8 * 1024 * 1024,
    segmentCount: 6,
  }));

  // Download with many segments for testing segment visualization
  tasks.push(generateMockTask('mock-9', DownloadStatus.Downloading, {
    filename: 'large-video-file-4k-hdr.mkv',
    filesize: 15_000_000_000, // 15 GB
    progress: 23.4,
    speed: 12.5 * 1024 * 1024, // 12.5 MB/s
    segmentCount: 150, // Many segments to test performance
  }));

  // Cancelled download
  tasks.push(generateMockTask('mock-10', DownloadStatus.Cancelled, {
    filename: 'update-package-cancelled.deb',
    filesize: 85_000_000,
    progress: 15.7,
    speed: 0,
    segmentCount: 4,
  }));

  return tasks;
}

/**
 * Simulate progress updates for mock downloads
 * Returns updated task with incremented progress
 */
export function updateMockTaskProgress(task: DownloadTask): DownloadTask {
  if (task.status !== DownloadStatus.Downloading) {
    return task;
  }

  // Increment progress by a small random amount
  const progressIncrement = Math.random() * 2; // 0-2% per update
  const newProgress = Math.min(100, task.progress + progressIncrement);

  // Vary speed slightly
  const speedVariation = (Math.random() - 0.5) * 0.5 * 1024 * 1024; // ±0.5 MB/s
  const newSpeed = Math.max(0, task.currentSpeed + speedVariation);

  // Update segments
  const updatedSegments = task.segments.map(seg => {
    if (seg.status === SegmentStatus.Downloading) {
      const segmentSize = seg.endByte - seg.startByte;
      const newDownloaded = Math.min(segmentSize, seg.downloadedBytes + (segmentSize * progressIncrement / 100));
      return {
        ...seg,
        downloadedBytes: newDownloaded,
        status: newDownloaded >= segmentSize ? SegmentStatus.Completed : SegmentStatus.Downloading,
      };
    }
    return seg;
  });

  return {
    ...task,
    progress: newProgress,
    currentSpeed: newSpeed,
    segments: updatedSegments,
    status: newProgress >= 100 ? DownloadStatus.Completed : DownloadStatus.Downloading,
    completedAt: newProgress >= 100 ? new Date().toISOString() : undefined,
  };
}
