// Download table row component - single row in the download table
import React from 'react';
import { TableRow, TableCell, Chip, Box, LinearProgress, Tooltip, IconButton } from '@mui/material';
import {
  CheckCircle as CompleteIcon,
  Error as ErrorIcon,
  Download as DownloadingIcon,
  Pause as PausedIcon,
  Schedule as PendingIcon,
  Cancel as CancelledIcon,
  PlayArrow as ResumeIcon,
  Stop as CancelIcon,
  Replay as RetryIcon,
  DeleteOutline as RemoveIcon,
} from '@mui/icons-material';
import { useTranslation } from 'react-i18next';
import type { TFunction } from 'i18next';
import { errorText } from '../utils/errors';
import type { Task, TaskState } from '../backend';
import { useDownloadActions } from '../hooks/useDownloadActions';
import { formatTaskForTable } from '../utils/downloadCalculations';
import { formatRelativeTime } from '../utils/formatters';

interface DownloadTableRowProps {
  task: Task;
  isSelected: boolean;
  onClick: (taskId: string) => void;
  onActionError?: (message: string) => void;
  columnWidths?: Record<string, number>;
}

const STATE_COLOR: Record<TaskState, 'default' | 'primary' | 'success' | 'error' | 'warning' | 'info'> = {
  scheduled: 'info',
  queued: 'default',
  running: 'primary',
  paused: 'warning',
  completed: 'success',
  failed: 'error',
  cancelled: 'default',
};

const STATE_ICON: Record<TaskState, React.ReactElement> = {
  scheduled: <PendingIcon fontSize="small" />,
  queued: <PendingIcon fontSize="small" />,
  running: <DownloadingIcon fontSize="small" />,
  paused: <PausedIcon fontSize="small" />,
  completed: <CompleteIcon fontSize="small" />,
  failed: <ErrorIcon fontSize="small" />,
  cancelled: <CancelledIcon fontSize="small" />,
};

type Action = 'pause' | 'resume' | 'cancel' | 'retry' | 'remove';

/** Which buttons make sense in each state (mirrors the transitions zoo-core allows). */
function actionsFor(state: TaskState): Action[] {
  switch (state) {
    case 'running':
    case 'queued':
    case 'scheduled':
      return ['pause', 'cancel', 'remove'];
    case 'paused':
      return ['resume', 'cancel', 'remove'];
    case 'failed':
      return ['retry', 'cancel', 'remove'];
    case 'cancelled':
      return ['retry', 'remove'];
    case 'completed':
      return ['remove'];
  }
}

const ACTION_ICON: Record<Action, React.ReactElement> = {
  pause: <PausedIcon fontSize="small" />,
  resume: <ResumeIcon fontSize="small" />,
  cancel: <CancelIcon fontSize="small" />,
  retry: <RetryIcon fontSize="small" />,
  remove: <RemoveIcon fontSize="small" />,
};

const DownloadTableRow: React.FC<DownloadTableRowProps> = React.memo(
  ({ task, isSelected, onClick, onActionError, columnWidths }) => {
    const { t } = useTranslation();
    const actions = useDownloadActions();
    const displayData = formatTaskForTable(task, t);

    const run = (action: Action) => (event: React.MouseEvent) => {
      event.stopPropagation(); // don't select the row
      actions[action](task.id).catch((error) => onActionError?.(error.message));
    };

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
        onClick={() => onClick(task.id)}
        sx={{
          cursor: 'pointer',
          '&.Mui-selected': { backgroundColor: 'action.selected' },
          '&.Mui-selected:hover': { backgroundColor: 'action.selected' },
        }}
      >
        <TableCell sx={getCellWidthStyle('filename')}>
          <Tooltip title={task.name} arrow placement="top-start">
            <Box sx={{ overflow: 'hidden', textOverflow: 'ellipsis', whiteSpace: 'nowrap' }}>
              {displayData.filename}
            </Box>
          </Tooltip>
        </TableCell>

        <TableCell align="right" sx={getCellWidthStyle('fileSize')}>
          {displayData.filesizeFormatted}
        </TableCell>

        <TableCell sx={getCellWidthStyle('progress')}>
          <Box sx={{ display: 'flex', alignItems: 'center', gap: 1 }}>
            <Box sx={{ flex: 1 }}>
              <LinearProgress
                variant={task.total === null && task.state === 'running' ? 'indeterminate' : 'determinate'}
                value={displayData.progress}
                color={task.state === 'completed' ? 'success' : task.state === 'failed' ? 'error' : 'primary'}
                sx={{ height: 6, borderRadius: 3 }}
              />
            </Box>
            <Box sx={{ minWidth: 45, textAlign: 'right' }}>{displayData.progressFormatted}</Box>
          </Box>
        </TableCell>

        <TableCell align="right" sx={getCellWidthStyle('currentSpeed')}>
          {displayData.speedFormatted}
        </TableCell>

        <TableCell sx={getCellWidthStyle('status')}>
          <Tooltip title={statusTooltip(task, displayData.statusLabel, t)} arrow>
            <Chip
              label={displayData.statusLabel}
              color={STATE_COLOR[task.state]}
              size="small"
              icon={STATE_ICON[task.state]}
            />
          </Tooltip>
        </TableCell>

        <TableCell align="center" sx={getCellWidthStyle('actions')}>
          {actionsFor(task.state).map((action) => (
            <Tooltip key={action} title={t(`downloads.actions.${action}`)} arrow>
              <IconButton size="small" onClick={run(action)} aria-label={t(`downloads.actions.${action}`)}>
                {ACTION_ICON[action]}
              </IconButton>
            </Tooltip>
          ))}
        </TableCell>

        <TableCell sx={getCellWidthStyle('createdAt')}>
          <Tooltip title={task.createdAt} arrow>
            <span>{formatRelativeTime(task.createdAt)}</span>
          </Tooltip>
        </TableCell>

        <TableCell align="right" sx={getCellWidthStyle('timeRemaining')}>
          {displayData.remainingTimeFormatted}
        </TableCell>
      </TableRow>
    );
  },
  (prev, next) =>
    // `rev` covers every state change; progress ticks change downloaded/total/speed.
    prev.task.rev === next.task.rev &&
    prev.task.downloaded === next.task.downloaded &&
    prev.task.total === next.task.total &&
    prev.task.speed === next.task.speed &&
    prev.isSelected === next.isSelected &&
    prev.onClick === next.onClick &&
    prev.onActionError === next.onActionError &&
    prev.columnWidths === next.columnWidths
);

DownloadTableRow.displayName = 'DownloadTableRow';

function statusTooltip(task: Task, label: string, t: TFunction): string {
  if (task.error) return `${label}: ${errorText(task.error, t)}`;
  if (task.state === 'running' && task.phase?.kind === 'waiting' && task.phase.reason) return task.phase.reason;
  return '';
}

export default DownloadTableRow;
