// Download item component - single download task UI with progress, speed, and control buttons
import React, { useState } from 'react';
import {
  Box,
  Card,
  CardContent,
  Typography,
  IconButton,
  Chip,
  Stack,
  Collapse,
  LinearProgress,
  Tooltip,
} from '@mui/material';
import {
  PlayArrow as PlayIcon,
  Pause as PauseIcon,
  Cancel as CancelIcon,
  CheckCircle as CompleteIcon,
  Error as ErrorIcon,
  ExpandMore as ExpandMoreIcon,
  ViewModule as SegmentsIcon,
} from '@mui/icons-material';
import { useTranslation } from 'react-i18next';
import { DownloadTask, DownloadStatus, SegmentStatus } from '../types/download';
import ProgressBar from './ProgressBar';
import useDownloads from '../hooks/useDownloads';

interface DownloadItemProps {
  task: DownloadTask;
}

const DownloadItem: React.FC<DownloadItemProps> = ({ task }) => {
  const { t } = useTranslation();
  const { startDownload, pauseDownload, cancelDownload } = useDownloads();
  const [showSegments, setShowSegments] = useState(false);

  const hasSegments = task.segments && task.segments.length > 0;

  const getStatusColor = (status: DownloadStatus): 'default' | 'primary' | 'success' | 'error' | 'warning' => {
    switch (status) {
      case 'Downloading':
        return 'primary';
      case 'Completed':
        return 'success';
      case 'Failed':
        return 'error';
      case 'Paused':
        return 'warning';
      default:
        return 'default';
    }
  };

  const getStatusIcon = (status: DownloadStatus) => {
    switch (status) {
      case 'Completed':
        return <CompleteIcon fontSize="small" />;
      case 'Failed':
        return <ErrorIcon fontSize="small" />;
      default:
        return null;
    }
  };

  const formatBytes = (bytes: number): string => {
    if (bytes === 0) return '0 B';
    const k = 1024;
    const sizes = ['B', 'KB', 'MB', 'GB'];
    const i = Math.floor(Math.log(bytes) / Math.log(k));
    return `${(bytes / Math.pow(k, i)).toFixed(2)} ${sizes[i]}`;
  };

  const formatSpeed = (bytesPerSecond: number): string => {
    return `${formatBytes(bytesPerSecond)}/s`;
  };

  return (
    <Card elevation={1}>
      <CardContent>
        <Box sx={{ display: 'flex', flexDirection: 'column', gap: 1 }}>
          {/* Header: filename and status */}
          <Box sx={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center' }}>
            <Typography variant="subtitle1" noWrap sx={{ maxWidth: '60%' }}>
              {task.filename}
            </Typography>
            <Chip
              label={t(`downloads.status.${task.status.toLowerCase()}`)}
              color={getStatusColor(task.status)}
              size="small"
              icon={getStatusIcon(task.status) || undefined}
            />
          </Box>

          {/* Progress bar */}
          {task.status === 'Downloading' || task.status === 'Paused' ? (
            <ProgressBar progress={task.progress} />
          ) : null}

          {/* Info: size, speed, progress percentage */}
          <Stack direction="row" spacing={2} sx={{ alignItems: 'center' }}>
            <Typography variant="caption" color="text.secondary">
              {task.filesize > 0 ? formatBytes(task.filesize) : t('downloads.unknown')}
            </Typography>
            {task.status === 'Downloading' && task.currentSpeed > 0 && (
              <Typography variant="caption" color="text.secondary">
                {formatSpeed(task.currentSpeed)}
              </Typography>
            )}
            {(task.status === 'Downloading' || task.status === 'Paused') && (
              <Typography variant="caption" color="text.secondary">
                {task.progress.toFixed(1)}%
              </Typography>
            )}
          </Stack>

          {/* Control buttons */}
          <Box sx={{ display: 'flex', gap: 1, marginTop: 1 }}>
            {task.status === 'Pending' || task.status === 'Paused' ? (
              <IconButton
                size="small"
                color="primary"
                aria-label={t('downloads.actions.start')}
                onClick={() => startDownload(task.id)}
              >
                <PlayIcon />
              </IconButton>
            ) : null}

            {task.status === 'Downloading' ? (
              <IconButton
                size="small"
                color="warning"
                aria-label={t('downloads.actions.pause')}
                onClick={() => pauseDownload(task.id)}
              >
                <PauseIcon />
              </IconButton>
            ) : null}

            {task.status !== 'Completed' && task.status !== 'Failed' ? (
              <IconButton
                size="small"
                color="error"
                aria-label={t('downloads.actions.cancel')}
                onClick={() => cancelDownload(task.id)}
              >
                <CancelIcon />
              </IconButton>
            ) : null}
          </Box>

          {/* URL (truncated) */}
          <Typography variant="caption" color="text.secondary" noWrap>
            {task.url}
          </Typography>

          {/* Segments toggle button */}
          {hasSegments && (
            <Box sx={{ display: 'flex', alignItems: 'center', gap: 1, marginTop: 1 }}>
              <IconButton
                size="small"
                onClick={() => setShowSegments(!showSegments)}
                sx={{
                  transform: showSegments ? 'rotate(180deg)' : 'rotate(0deg)',
                  transition: 'transform 0.3s',
                }}
              >
                <ExpandMoreIcon />
              </IconButton>
              <Chip
                icon={<SegmentsIcon />}
                label={t('downloads.segments', '{{count}} 個分段', {
                  count: task.segments.length,
                })}
                size="small"
                variant="outlined"
              />
            </Box>
          )}

          {/* Segments details (collapsible) */}
          {hasSegments && (
            <Collapse in={showSegments}>
              <Box sx={{ marginTop: 2, paddingLeft: 2, borderLeft: '2px solid', borderColor: 'divider' }}>
                <Typography variant="caption" fontWeight="bold" color="text.secondary" gutterBottom>
                  {t('downloads.segmentDetails', '分段詳細資訊')}
                </Typography>
                <Stack spacing={1} sx={{ marginTop: 1 }}>
                  {task.segments.map((segment) => {
                    const segmentProgress =
                      segment.endByte - segment.startByte > 0
                        ? (segment.downloadedBytes / (segment.endByte - segment.startByte + 1)) * 100
                        : 0;

                    const getSegmentChipColor = (status: SegmentStatus): 'default' | 'primary' | 'success' | 'error' => {
                      switch (status) {
                        case 'Downloading':
                          return 'primary';
                        case 'Completed':
                          return 'success';
                        case 'Failed':
                          return 'error';
                        default:
                          return 'default';
                      }
                    };

                    const getSegmentProgressColor = (status: SegmentStatus): 'inherit' | 'primary' | 'secondary' | 'success' | 'error' => {
                      switch (status) {
                        case 'Downloading':
                          return 'primary';
                        case 'Completed':
                          return 'success';
                        case 'Failed':
                          return 'error';
                        default:
                          return 'secondary';
                      }
                    };

                    return (
                      <Box key={segment.id} sx={{ padding: 1, bgcolor: 'action.hover', borderRadius: 1 }}>
                        <Box sx={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center', mb: 0.5 }}>
                          <Typography variant="caption" fontWeight="medium">
                            {t('downloads.segment', '分段 {{id}}', { id: segment.segmentId + 1 })}
                          </Typography>
                          <Chip
                            label={segment.status}
                            size="small"
                            color={getSegmentChipColor(segment.status)}
                            sx={{ height: 18, fontSize: '0.65rem' }}
                          />
                        </Box>
                        <Tooltip
                          title={`${formatBytes(segment.downloadedBytes)} / ${formatBytes(
                            segment.endByte - segment.startByte + 1
                          )}`}
                        >
                          <LinearProgress
                            variant="determinate"
                            value={segmentProgress}
                            color={getSegmentProgressColor(segment.status)}
                            sx={{ height: 4, borderRadius: 2 }}
                          />
                        </Tooltip>
                        <Typography variant="caption" color="text.secondary" sx={{ fontSize: '0.65rem' }}>
                          {segmentProgress.toFixed(1)}% • {formatBytes(segment.startByte)} - {formatBytes(segment.endByte)}
                        </Typography>
                      </Box>
                    );
                  })}
                </Stack>
              </Box>
            </Collapse>
          )}
        </Box>
      </CardContent>
    </Card>
  );
};

export default DownloadItem;
