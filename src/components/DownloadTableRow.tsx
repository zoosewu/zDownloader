// Download table row component - single row in the download table
import React, { useMemo } from 'react';
import { useAtomValue } from 'jotai';
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
  DeleteOutlined as RemoveIcon,
} from '@mui/icons-material';
import { useTranslation } from 'react-i18next';
import type { TFunction } from 'i18next';
import { errorText } from '../utils/errors';
import type { Task, TaskId, TaskState } from '../backend';
import { taskAtom } from '../stores/downloadStore';
import { useDownloadActions } from '../hooks/useDownloadActions';
import { useSortable } from '@dnd-kit/sortable';
import { CSS } from '@dnd-kit/utilities';
import { useNow } from '../hooks/useNow';
import { deadline, formatTaskForTable } from '../utils/downloadCalculations';
import { formatAbsoluteTime, formatRelativeTime } from '../utils/formatters';
import { clickMode, type ClickMode } from '../utils/selection';
import { actionsFor, type Action } from '../utils/taskActions';

interface RowProps {
  isSelected: boolean;
  onClick: (taskId: string, mode: ClickMode) => void;
  onActionError?: (message: string) => void;
  /** Right click: the menu for the selection. */
  onContextMenu?: (taskId: string, x: number, y: number) => void;
  /** Drag to reorder (only where the whole queue is shown). */
  sortable?: boolean;
  columnWidths?: Record<string, number>;
}


// Theme palette names only (see styles/theme.ts): running is info; completed, the normal end,
// is the brand but light (an outlined chip, a softer bar), so a finished list is not a wall of
// green and does not look disabled either.
const STATE_COLOR: Record<TaskState, 'default' | 'primary' | 'success' | 'error' | 'warning' | 'info'> = {
  scheduled: 'default',
  queued: 'default',
  running: 'info',
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

/** Waiting and cancelled bars stay neutral (`inherit` takes the grey set on the bar). */
const PROGRESS_COLOR: Record<TaskState, 'inherit' | 'success' | 'error' | 'warning' | 'info'> = {
  scheduled: 'inherit',
  queued: 'inherit',
  running: 'info',
  paused: 'warning',
  completed: 'success',
  failed: 'error',
  cancelled: 'inherit',
};


const ACTION_ICON: Record<Action, React.ReactElement> = {
  pause: <PausedIcon fontSize="small" />,
  resume: <ResumeIcon fontSize="small" />,
  cancel: <CancelIcon fontSize="small" />,
  retry: <RetryIcon fontSize="small" />,
  remove: <RemoveIcon fontSize="small" />,
};

// Cells that change only with the task's state, not its progress. Memoized on primitive
// props so a progress tick skips them (with their tooltips and buttons) entirely.

const widthSx = (width?: number) => (width ? { width, minWidth: width, maxWidth: width } : {});

const FilenameCell = React.memo(({ name, width }: { name: string; width?: number }) => (
  <TableCell sx={widthSx(width)}>
    <Tooltip title={name} arrow placement="top-start">
      <Box sx={{ overflow: 'hidden', textOverflow: 'ellipsis', whiteSpace: 'nowrap' }}>{name}</Box>
    </Tooltip>
  </TableCell>
));

const StatusCell = React.memo(
  ({ label, state, tooltip, width }: { label: string; state: TaskState; tooltip: string; width?: number }) => (
    <TableCell sx={widthSx(width)}>
      <Tooltip title={tooltip} arrow>
        <Chip
          label={label}
          color={STATE_COLOR[state]}
          variant={state === 'completed' ? 'outlined' : 'filled'}
          size="small"
          icon={STATE_ICON[state]}
        />
      </Tooltip>
    </TableCell>
  ),
);

const ActionsCell = React.memo(
  ({ id, state, onActionError, width }: { id: TaskId; state: TaskState; onActionError?: (message: string) => void; width?: number }) => {
    const { t } = useTranslation();
    const actions = useDownloadActions();
    const run = (action: Action) => (event: React.MouseEvent) => {
      event.stopPropagation(); // don't select the row
      actions[action](id).catch((error) => onActionError?.(error.message));
    };
    return (
      <TableCell align="center" sx={widthSx(width)}>
        {actionsFor(state).map((action) => (
          <Tooltip key={action} title={t(`downloads.actions.${action}`)} arrow>
            <IconButton onClick={run(action)} aria-label={t(`downloads.actions.${action}`)} sx={{ p: '2px' }}>
              {ACTION_ICON[action]}
            </IconButton>
          </Tooltip>
        ))}
      </TableCell>
    );
  },
);

const CreatedCell = React.memo(({ createdAt, width }: { createdAt: string; width?: number }) => {
  useTranslation(); // re-render in the new language
  return (
    <TableCell sx={widthSx(width)}>
      <Tooltip title={formatAbsoluteTime(createdAt)} arrow>
        <span>{formatRelativeTime(createdAt)}</span>
      </Tooltip>
    </TableCell>
  );
});

const TaskRow: React.FC<RowProps & { task: Task }> = React.memo(
  ({ task, isSelected, onClick, onActionError, onContextMenu, sortable = true, columnWidths }) => {
    const { t } = useTranslation();
    const { attributes, listeners, setNodeRef, transform, transition, isDragging } = useSortable({
      id: task.id,
      attributes: { role: 'row' },
      disabled: !sortable,
    });
    const until = deadline(task);
    const displayData = formatTaskForTable(task, t, useNow(until !== null));
    // An automatic retry is coming: looks like a scheduled task, not a failure.
    const shownState: TaskState = task.state === 'failed' && until ? 'scheduled' : task.state;

    const width = (columnKey: string) => columnWidths?.[columnKey];
    const getCellWidthStyle = (columnKey: string) => widthSx(width(columnKey));

    return (
      <TableRow
        ref={setNodeRef}
        {...attributes}
        {...listeners}
        hover
        selected={isSelected}
        onClick={(e) => onClick(task.id, clickMode(e))}
        onContextMenu={(e) => {
          e.preventDefault();
          onContextMenu?.(task.id, e.clientX, e.clientY);
        }}
        style={{ transform: CSS.Translate.toString(transform), transition }}
        sx={{
          cursor: 'pointer',
          // Shift-click selects rows, not text.
          userSelect: 'none',
          position: 'relative',
          zIndex: isDragging ? 1 : undefined,
          opacity: isDragging ? 0.8 : undefined,
          '&.Mui-selected': { backgroundColor: 'action.selected' },
          '&.Mui-selected:hover': { backgroundColor: 'action.selected' },
        }}
      >
        <FilenameCell name={displayData.filename} width={width('filename')} />

        <TableCell align="right" sx={getCellWidthStyle('fileSize')}>
          {displayData.filesizeFormatted}
        </TableCell>

        <TableCell sx={getCellWidthStyle('progress')}>
          <Box sx={{ display: 'flex', alignItems: 'center', gap: 1 }}>
            <Box sx={{ flex: 1 }}>
              <LinearProgress
                variant={task.total === null && task.state === 'running' ? 'indeterminate' : 'determinate'}
                value={displayData.progress}
                color={PROGRESS_COLOR[shownState]}
                sx={{ height: 4, color: 'text.disabled', '& .MuiLinearProgress-bar': { opacity: shownState === 'completed' ? 0.55 : 1 } }}
              />
            </Box>
            <Box sx={{ minWidth: 42, textAlign: 'right', fontVariantNumeric: 'tabular-nums' }}>{displayData.progressFormatted}</Box>
          </Box>
        </TableCell>

        <TableCell align="right" sx={getCellWidthStyle('currentSpeed')}>
          {displayData.speedFormatted}
        </TableCell>

        <StatusCell
          label={displayData.statusLabel}
          state={shownState}
          tooltip={statusTooltip(task, displayData.statusLabel, t)}
          width={width('status')}
        />

        <ActionsCell id={task.id} state={task.state} onActionError={onActionError} width={width('actions')} />

        <CreatedCell createdAt={task.createdAt} width={width('createdAt')} />

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
    prev.onContextMenu === next.onContextMenu &&
    prev.sortable === next.sortable &&
    prev.columnWidths === next.columnWidths
);

TaskRow.displayName = 'TaskRow';

interface DownloadTableRowProps extends RowProps {
  id: TaskId;
}

/**
 * One row, subscribed to its own task: a progress tick re-renders this row only. The table
 * passes ids, never task objects.
 */
const DownloadTableRow: React.FC<DownloadTableRowProps> = React.memo(({ id, ...props }) => {
  const task = useAtomValue(useMemo(() => taskAtom(id), [id]));
  // Removed: the table drops the row on its next render.
  return task ? <TaskRow task={task} {...props} /> : null;
});
DownloadTableRow.displayName = 'DownloadTableRow';

function statusTooltip(task: Task, label: string, t: TFunction): string {
  if (task.error) return `${label}: ${errorText(task.error, t)}`;
  if (task.state === 'running' && task.phase?.kind === 'waiting' && task.phase.reason) return task.phase.reason;
  return '';
}

export default DownloadTableRow;
