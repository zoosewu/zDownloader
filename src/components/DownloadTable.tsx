// Download table component - table view for all download items with resizable columns
import React, { useState, useRef, useEffect, useCallback } from 'react';
import {
  Table,
  TableBody,
  TableCell,
  TableContainer,
  TableHead,
  TableRow,
  Paper,
  Box,
  Typography,
} from '@mui/material';
import { useTranslation } from 'react-i18next';
import { DndContext, PointerSensor, closestCenter, useSensor, useSensors, type DragEndEvent } from '@dnd-kit/core';
import { SortableContext, verticalListSortingStrategy } from '@dnd-kit/sortable';
import type { Task } from '../backend';
import DownloadTableRow from './DownloadTableRow';
import { useSelectedDownload } from '../hooks/useSelectedDownload';
import { useDownloadActions } from '../hooks/useDownloadActions';
import { droppedBefore } from '../utils/downloadCalculations';

interface DownloadTableProps {
  tasks: Task[];
  onActionError?: (message: string) => void;
}

// Column identifiers and default widths
type ColumnKey = 'filename' | 'fileSize' | 'progress' | 'currentSpeed' | 'status' | 'actions' | 'createdAt' | 'timeRemaining';
const DEFAULT_COLUMN_WIDTHS: Record<ColumnKey, number> = {
  filename: 200,
  fileSize: 100,
  progress: 180,
  currentSpeed: 100,
  status: 120,
  actions: 150,
  createdAt: 130,
  timeRemaining: 120,
};

/**
 * Download table component with sticky header and resizable columns
 * Displays all download tasks in a tabular format
 */
const DownloadTable: React.FC<DownloadTableProps> = ({ tasks, onActionError }) => {
  const { t } = useTranslation();
  const { selectedTaskId, selectTask } = useSelectedDownload();
  const [columnWidths, setColumnWidths] = useState<Record<ColumnKey, number>>(DEFAULT_COLUMN_WIDTHS);
  const [resizingColumn, setResizingColumn] = useState<ColumnKey | null>(null);
  const startXRef = useRef<number>(0);
  const startWidthRef = useRef<number>(0);

  // Toggle selection; stable across renders so memoized rows skip re-rendering.
  const selectedRef = useRef(selectedTaskId);
  selectedRef.current = selectedTaskId;
  const handleRowClick = useCallback(
    (taskId: string) => selectTask(selectedRef.current === taskId ? null : taskId),
    [selectTask]
  );

  // Drag a row to reorder; a short move first, so clicks still select.
  const sensors = useSensors(useSensor(PointerSensor, { activationConstraint: { distance: 5 } }));
  const { move } = useDownloadActions();
  const handleDragEnd = ({ active, over }: DragEndEvent) => {
    if (!over || active.id === over.id) return;
    const before = droppedBefore(tasks.map((t) => t.id), String(active.id), String(over.id));
    move(String(active.id), before).catch((error) => onActionError?.(error.message));
  };

  // Handle resize start
  const handleResizeStart = (e: React.MouseEvent, column: ColumnKey) => {
    e.preventDefault();
    e.stopPropagation();
    setResizingColumn(column);
    startXRef.current = e.clientX;
    startWidthRef.current = columnWidths[column];
  };

  // Handle mouse move during resize
  useEffect(() => {
    const handleMouseMove = (e: MouseEvent) => {
      if (!resizingColumn) return;

      const diff = e.clientX - startXRef.current;
      const newWidth = Math.max(50, startWidthRef.current + diff); // Minimum 50px

      setColumnWidths((prev) => ({
        ...prev,
        [resizingColumn]: newWidth,
      }));
    };

    const handleMouseUp = () => {
      setResizingColumn(null);
    };

    if (resizingColumn) {
      document.addEventListener('mousemove', handleMouseMove);
      document.addEventListener('mouseup', handleMouseUp);
    }

    return () => {
      document.removeEventListener('mousemove', handleMouseMove);
      document.removeEventListener('mouseup', handleMouseUp);
    };
  }, [resizingColumn]);

  // Empty state
  if (tasks.length === 0) {
    return (
      <Box
        sx={{
          display: 'flex',
          justifyContent: 'center',
          alignItems: 'center',
          minHeight: 200,
          padding: 4,
        }}
      >
        <Typography variant="body1" color="text.secondary">
          {t('downloads.noTasks')}
        </Typography>
      </Box>
    );
  }

  // Render resizable header cell
  const renderHeaderCell = (
    column: ColumnKey,
    label: string,
    align: 'left' | 'right' | 'center' = 'left'
  ) => (
    <TableCell
      key={column}
      align={align}
      sx={{
        fontWeight: 'bold',
        width: columnWidths[column],
        minWidth: columnWidths[column],
        maxWidth: columnWidths[column],
        position: 'relative',
        paddingRight: '24px',
        userSelect: 'none',
      }}
    >
      {label}
      {/* Resize handle */}
      <Box
        onMouseDown={(e) => handleResizeStart(e, column)}
        sx={{
          position: 'absolute',
          right: 0,
          top: 0,
          bottom: 0,
          width: '8px',
          cursor: 'col-resize',
          backgroundColor: resizingColumn === column ? 'primary.main' : 'transparent',
          '&:hover': {
            backgroundColor: 'primary.light',
          },
          transition: 'background-color 0.2s',
          zIndex: 1,
        }}
      />
    </TableCell>
  );

  return (
    <TableContainer
      component={Paper}
      sx={{
        maxHeight: 'calc(100vh - 300px)', // Allow vertical scrolling
        overflow: 'auto',
      }}
    >
      <Table stickyHeader aria-label="download table">
        <TableHead>
          <TableRow>
            {/* Column Headers with resize handles */}
            {renderHeaderCell('filename', t('downloads.details.filename', '檔案名稱'))}
            {renderHeaderCell('fileSize', t('downloads.details.fileSize'), 'right')}
            {renderHeaderCell('progress', t('common.progress', '進度'))}
            {renderHeaderCell('currentSpeed', t('downloads.statistics.currentSpeed'), 'right')}
            {renderHeaderCell('status', t('downloads.status.title', '狀態'))}
            {renderHeaderCell('actions', t('downloads.actions.title'), 'center')}
            {renderHeaderCell('createdAt', t('downloads.details.createdAt'))}
            {renderHeaderCell('timeRemaining', t('downloads.statistics.timeRemaining'), 'right')}
          </TableRow>
        </TableHead>
        <DndContext sensors={sensors} collisionDetection={closestCenter} onDragEnd={handleDragEnd}>
          <SortableContext items={tasks.map((t) => t.id)} strategy={verticalListSortingStrategy}>
            <TableBody>
              {tasks.map((task) => (
                <DownloadTableRow
                  key={task.id}
                  task={task}
                  isSelected={selectedTaskId === task.id}
                  onClick={handleRowClick}
                  onActionError={onActionError}
                  columnWidths={columnWidths}
                />
              ))}
            </TableBody>
          </SortableContext>
        </DndContext>
      </Table>
    </TableContainer>
  );
};

export default DownloadTable;
