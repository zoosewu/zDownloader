// Download table component - table view for all download items with resizable columns
import React, { useState, useRef, useEffect, useCallback } from 'react';
import {
  Table,
  TableBody,
  TableCell,
  TableContainer,
  TableHead,
  TableRow,
  Box,
  Typography,
} from '@mui/material';
import { useTranslation } from 'react-i18next';
import { DndContext, PointerSensor, closestCenter, useSensor, useSensors, type DragEndEvent } from '@dnd-kit/core';
import { SortableContext, verticalListSortingStrategy } from '@dnd-kit/sortable';
import { useAtomValue } from 'jotai';
import DownloadTableRow from './DownloadTableRow';
import { visibleTaskIdsAtom, viewAtom, canReorder } from '../stores/viewStore';
import TaskCategoryMenu, { type MenuAnchor } from './TaskCategoryMenu';
import { useSelectedDownload } from '../hooks/useSelectedDownload';
import { useDownloadActions } from '../hooks/useDownloadActions';
import { droppedBefore } from '../utils/downloadCalculations';

interface DownloadTableProps {
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
const DownloadTable: React.FC<DownloadTableProps> = ({ onActionError }) => {
  const { t } = useTranslation();
  // Only the order of the shown tasks: progress re-renders the row of that task, not the table.
  const ids = useAtomValue(visibleTaskIdsAtom);
  const sortable = canReorder(useAtomValue(viewAtom));
  const { selectedTaskId, selectTask } = useSelectedDownload();
  const [menu, setMenu] = useState<MenuAnchor | null>(null);
  const handleContextMenu = useCallback((id: string, x: number, y: number) => setMenu({ id, x, y }), []);

  // A selected task that leaves the view (finished, moved to another category) is deselected.
  useEffect(() => {
    if (selectedTaskId && !ids.includes(selectedTaskId)) selectTask(null);
  }, [ids, selectedTaskId, selectTask]);
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
    if (!sortable || !over || active.id === over.id) return;
    const before = droppedBefore(ids, String(active.id), String(over.id));
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
  if (ids.length === 0) {
    return (
      <Box
        sx={{
          display: 'flex',
          justifyContent: 'center',
          alignItems: 'center',
          height: '100%',
          minHeight: 120,
        }}
      >
        <Typography variant="body2" color="text.secondary">
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
        width: columnWidths[column],
        minWidth: columnWidths[column],
        maxWidth: columnWidths[column],
        position: 'relative',
        whiteSpace: 'nowrap',
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
          width: 5,
          cursor: 'col-resize',
          borderRight: 1,
          borderColor: resizingColumn === column ? 'primary.main' : 'divider',
          '&:hover': { borderColor: 'primary.main' },
          zIndex: 1,
        }}
      />
    </TableCell>
  );

  return (
    <TableContainer sx={{ height: '100%', overflow: 'auto', backgroundColor: 'background.paper' }}>
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
          <SortableContext items={ids} strategy={verticalListSortingStrategy}>
            <TableBody>
              {ids.map((id) => (
                <DownloadTableRow
                  key={id}
                  id={id}
                  isSelected={selectedTaskId === id}
                  onClick={handleRowClick}
                  onActionError={onActionError}
                  onContextMenu={handleContextMenu}
                  sortable={sortable}
                  columnWidths={columnWidths}
                />
              ))}
            </TableBody>
          </SortableContext>
        </DndContext>
      </Table>
      {menu && <TaskCategoryMenu anchor={menu} onClose={() => setMenu(null)} onError={onActionError} />}
    </TableContainer>
  );
};

export default DownloadTable;
