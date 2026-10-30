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
  columnWidths?: Record<string, number>;
}

/**
 * Get status color for Chip component
 */
function getStatusColor(
  status: DownloadStatus
): 'default' | 'primary' | 'success' | 'error' | 'warning' {
  switch (status) {
    case DownloadStatus.Downloading:
    case DownloadStatus.Initializing:
      return 'primary';
    case DownloadStatus.Completed:
      return 'success';
    case DownloadStatus.Failed:
      return 'error';
    case DownloadStatus.Paused:
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
    case DownloadStatus.Completed:
      return <CompleteIcon fontSize="small" />;
    case DownloadStatus.Failed:
      return <ErrorIcon fontSize="small" />;
    case DownloadStatus.Downloading:
    case DownloadStatus.Initializing:
      return <DownloadingIcon fontSize="small" />;
    case DownloadStatus.Paused:
      return <PausedIcon fontSize="small" />;
    case DownloadStatus.Pending:
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
  ({ task, isSelected, onClick, columnWidths }) => {
    const { t } = useTranslation();

    // Format task data for display
    const displayData = formatTaskForTable(task, t);
    const createdAtFormatted = formatRelativeTime(task.createdAt);

    // Handle row click
    const handleClick = () => {
      onClick(task.id);
    };

    // Helper to get cell width styles
    const getCellWidthStyle = (columnKey: string) =>
      columnWidths
        ? {
            width: columnWidths[columnKey],
            minWidth: columnWidths[columnKey],
            maxWidth: columnWidths[columnKey],
          }
        : {};

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
        <TableCell sx={getCellWidthStyle('filename')}>
          <Tooltip title={task.filename} arrow placement="top-start">
            <Box
              sx={{
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
        <TableCell align="right" sx={getCellWidthStyle('fileSize')}>
          {displayData.filesizeFormatted}
        </TableCell>

        {/* Progress Column */}
        <TableCell sx={getCellWidthStyle('progress')}>
          <Box sx={{ display: 'flex', alignItems: 'center', gap: 1 }}>
            <Box sx={{ flex: 1 }}>
              <LinearProgress
                variant="determinate"
                value={displayData.progress}
                color={
                  task.status === DownloadStatus.Completed
                    ? 'success'
                    : task.status === DownloadStatus.Failed
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
        <TableCell align="right" sx={getCellWidthStyle('currentSpeed')}>
          {displayData.speedFormatted}
        </TableCell>

        {/* Status Column */}
        <TableCell sx={getCellWidthStyle('status')}>
          <Chip
            label={displayData.statusLabel}
            color={getStatusColor(task.status)}
            size="small"
            icon={getStatusIcon(task.status) || undefined}
          />
        </TableCell>

        {/* Priority Column */}
        <TableCell align="center" sx={getCellWidthStyle('priority')}>
          {formatSpeedPriority(task.speedPriority, t)}
        </TableCell>

        {/* Created At Column */}
        <TableCell sx={getCellWidthStyle('createdAt')}>
          <Tooltip title={task.createdAt} arrow>
            <span>{createdAtFormatted}</span>
          </Tooltip>
        </TableCell>

        {/* Remaining Time Column */}
        <TableCell align="right" sx={getCellWidthStyle('timeRemaining')}>
          {task.status === DownloadStatus.Downloading && displayData.remainingTimeFormatted !== '--' ? (
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
      prevProps.isSelected === nextProps.isSelected &&
      prevProps.columnWidths === nextProps.columnWidths
    );
  }
);

// Display name for debugging
DownloadTableRow.displayName = 'DownloadTableRow';

export default DownloadTableRow;
