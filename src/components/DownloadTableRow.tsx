// Download table row component - single row in the download table
import React from 'react';
import { TableRow, TableCell, Chip, Box, LinearProgress, Tooltip } from '@mui/material';
import {
  CheckCircle as CompleteIcon,
  Error as ErrorIcon,
  Download as DownloadingIcon,
  Pause as PausedIcon,
  Schedule as PendingIcon,
} from '@mui/icons-material';
import { useTranslation } from 'react-i18next';
import { DownloadTask, DownloadStatus } from '../types/download';
import { formatTaskForTable } from '../utils/downloadCalculations';
import { formatRelativeTime, formatSpeedPriority } from '../utils/formatters';

interface DownloadTableRowProps {
  task: DownloadTask;
  isSelected: boolean;
  onClick: (taskId: string) => void;
}

/**
 * Get status color for Chip component
 */
function getStatusColor(
  status: DownloadStatus
): 'default' | 'primary' | 'success' | 'error' | 'warning' {
  switch (status) {
    case 'Downloading':
    case 'Initializing':
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
}

/**
 * Get status icon for Chip component
 */
function getStatusIcon(status: DownloadStatus) {
  switch (status) {
    case 'Completed':
      return <CompleteIcon fontSize="small" />;
    case 'Failed':
      return <ErrorIcon fontSize="small" />;
    case 'Downloading':
    case 'Initializing':
      return <DownloadingIcon fontSize="small" />;
    case 'Paused':
      return <PausedIcon fontSize="small" />;
    case 'Pending':
      return <PendingIcon fontSize="small" />;
    default:
      return null;
  }
}

/**
 * Download table row component with React.memo optimization
 * Only re-renders when task data actually changes
 */
const DownloadTableRow: React.FC<DownloadTableRowProps> = React.memo(
  ({ task, isSelected, onClick }) => {
    const { t } = useTranslation();

    // Format task data for display
    const displayData = formatTaskForTable(task, t);
    const createdAtFormatted = formatRelativeTime(task.createdAt);

    // Handle row click
    const handleClick = () => {
      onClick(task.id);
    };

    return (
      <TableRow
        hover
        selected={isSelected}
        onClick={handleClick}
        sx={{
          cursor: 'pointer',
          '&.Mui-selected': {
            backgroundColor: 'action.selected',
          },
          '&.Mui-selected:hover': {
            backgroundColor: 'action.selected',
          },
        }}
      >
        {/* Filename Column */}
        <TableCell>
          <Tooltip title={task.filename} arrow placement="top-start">
            <Box
              sx={{
                maxWidth: 300,
                overflow: 'hidden',
                textOverflow: 'ellipsis',
                whiteSpace: 'nowrap',
              }}
            >
              {displayData.filename}
            </Box>
          </Tooltip>
        </TableCell>

        {/* File Size Column */}
        <TableCell align="right">{displayData.filesizeFormatted}</TableCell>

        {/* Progress Column */}
        <TableCell>
          <Box sx={{ display: 'flex', alignItems: 'center', gap: 1, minWidth: 120 }}>
            <Box sx={{ flex: 1 }}>
              <LinearProgress
                variant="determinate"
                value={displayData.progress}
                color={
                  task.status === 'Completed'
                    ? 'success'
                    : task.status === 'Failed'
                    ? 'error'
                    : 'primary'
                }
                sx={{ height: 6, borderRadius: 3 }}
              />
            </Box>
            <Box sx={{ minWidth: 45, textAlign: 'right' }}>
              {displayData.progressFormatted}
            </Box>
          </Box>
        </TableCell>

        {/* Speed Column */}
        <TableCell align="right">{displayData.speedFormatted}</TableCell>

        {/* Status Column */}
        <TableCell>
          <Chip
            label={displayData.statusLabel}
            color={getStatusColor(task.status)}
            size="small"
            icon={getStatusIcon(task.status) || undefined}
          />
        </TableCell>

        {/* Priority Column */}
        <TableCell align="center">
          {formatSpeedPriority(task.speedPriority, t)}
        </TableCell>

        {/* Created At Column */}
        <TableCell>
          <Tooltip title={task.createdAt} arrow>
            <span>{createdAtFormatted}</span>
          </Tooltip>
        </TableCell>

        {/* Remaining Time Column */}
        <TableCell align="right">
          {task.status === 'Downloading' && displayData.remainingTimeFormatted !== '--' ? (
            <Tooltip title={t('downloads.statistics.timeRemaining')} arrow>
              <span>{displayData.remainingTimeFormatted}</span>
            </Tooltip>
          ) : (
            <span>--</span>
          )}
        </TableCell>
      </TableRow>
    );
  },
  (prevProps, nextProps) => {
    // Custom comparison function for React.memo
    // Only re-render if these specific properties change
    return (
      prevProps.task.id === nextProps.task.id &&
      prevProps.task.filename === nextProps.task.filename &&
      prevProps.task.progress === nextProps.task.progress &&
      prevProps.task.currentSpeed === nextProps.task.currentSpeed &&
      prevProps.task.status === nextProps.task.status &&
      prevProps.task.filesize === nextProps.task.filesize &&
      prevProps.task.createdAt === nextProps.task.createdAt &&
      prevProps.task.speedPriority === nextProps.task.speedPriority &&
      prevProps.isSelected === nextProps.isSelected
    );
  }
);

// Display name for debugging
DownloadTableRow.displayName = 'DownloadTableRow';

export default DownloadTableRow;
