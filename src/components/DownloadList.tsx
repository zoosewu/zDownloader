// Download list component - displays all download tasks in table view with resizable detail panel
import React, { useState, useRef, useEffect } from 'react';
import { Box, Divider, Button } from '@mui/material';
import AddIcon from '@mui/icons-material/Add';
import DownloadTable from './DownloadTable';
import DownloadDetailPanel from './DownloadDetailPanel';
import QueueControls from './QueueControls';
import BatchImportButton from './BatchImportButton';
import { useSelectedDownload, useSelectedTask } from '../hooks/useSelectedDownload';
import type { BatchResult } from '../backend';

/** The panel is the only part of the list that follows the selected task's progress. */
const SelectedTaskPanel: React.FC = () => <DownloadDetailPanel task={useSelectedTask()} />;

interface DownloadListProps {
  className?: string;
  onControlError: (error: string) => void;
  onAddDownload: () => void;
  onBatchImportSuccess: (result: BatchResult) => void;
  onBatchImportError: (error: string) => void;
}

const MIN_PANEL_HEIGHT = 200; // Minimum height for detail panel
const DEFAULT_PANEL_HEIGHT = 300; // Default height for detail panel

const DownloadList: React.FC<DownloadListProps> = ({
  className,
  onControlError,
  onAddDownload,
  onBatchImportSuccess,
  onBatchImportError,
}) => {
  const { selectedTaskId } = useSelectedDownload();
  const [panelHeight, setPanelHeight] = useState(DEFAULT_PANEL_HEIGHT);
  const [isDragging, setIsDragging] = useState(false);
  const containerRef = useRef<HTMLDivElement>(null);

  // Handle resize drag
  const handleMouseDown = (e: React.MouseEvent) => {
    e.preventDefault();
    setIsDragging(true);
  };

  // Handle mouse move during drag
  useEffect(() => {
    const handleMouseMove = (e: MouseEvent) => {
      if (!isDragging || !containerRef.current) return;

      const containerRect = containerRef.current.getBoundingClientRect();
      const newHeight = containerRect.bottom - e.clientY;

      // Constrain panel height
      const maxHeight = containerRect.height - 200; // Leave at least 200px for table
      const constrainedHeight = Math.max(MIN_PANEL_HEIGHT, Math.min(newHeight, maxHeight));

      setPanelHeight(constrainedHeight);
    };

    const handleMouseUp = () => {
      setIsDragging(false);
    };

    if (isDragging) {
      document.addEventListener('mousemove', handleMouseMove);
      document.addEventListener('mouseup', handleMouseUp);
    }

    return () => {
      document.removeEventListener('mousemove', handleMouseMove);
      document.removeEventListener('mouseup', handleMouseUp);
    };
  }, [isDragging]);

  return (
    <Box
      ref={containerRef}
      className={className}
      sx={{
        width: '100%',
        height: '100%',
        display: 'flex',
        flexDirection: 'column',
        position: 'relative',
      }}
    >
      {/* Queue Controls and Action Buttons Header - fixed at top */}
      <Box
        sx={{
          display: 'flex',
          justifyContent: 'space-between',
          alignItems: 'center',
          px: 2,
          py: 1.5,
          borderBottom: 1,
          borderColor: 'divider',
          backgroundColor: 'background.paper',
        }}
      >
        {/* Queue controls on the left */}
        <QueueControls onControlError={onControlError} />

        {/* Action buttons on the right */}
        <Box sx={{ display: 'flex', gap: 2 }}>
          <Button
            variant="contained"
            startIcon={<AddIcon />}
            onClick={onAddDownload}
            size="small"
          >
            新增下載
          </Button>
          <BatchImportButton
            onImportSuccess={onBatchImportSuccess}
            onImportError={onBatchImportError}
          />
        </Box>
      </Box>

      {/* Scrollable Content Area - DownloadTable */}
      <Box
        sx={{
          flex: 1,
          overflow: 'auto',
          minHeight: 200,
        }}
      >

        {/* Download Table */}
        <DownloadTable onActionError={onControlError} />
      </Box>

      {/* Resize Handle and Detail Panel - fixed at bottom */}
      {selectedTaskId && (
        <>
          <Divider
            onMouseDown={handleMouseDown}
            sx={{
              height: '8px',
              cursor: 'ns-resize',
              backgroundColor: isDragging ? 'primary.main' : 'divider',
              '&:hover': {
                backgroundColor: 'primary.light',
              },
              transition: 'background-color 0.2s',
              position: 'relative',
              '&::after': {
                content: '""',
                position: 'absolute',
                top: '50%',
                left: '50%',
                transform: 'translate(-50%, -50%)',
                width: '40px',
                height: '4px',
                borderRadius: '2px',
                backgroundColor: 'text.secondary',
              },
            }}
          />

          {/* Download Detail Panel - fixed height */}
          <Box
            sx={{
              height: `${panelHeight}px`,
              overflow: 'hidden',
              backgroundColor: 'background.paper',
            }}
          >
            <SelectedTaskPanel />
          </Box>
        </>
      )}
    </Box>
  );
};

export default DownloadList;
