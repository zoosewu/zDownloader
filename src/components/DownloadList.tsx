// Download list component - displays all download tasks in table view with resizable detail panel
import React, { useState, useRef, useEffect } from 'react';
import { Box, Divider } from '@mui/material';
import { useAtom } from 'jotai';
import { downloadTasksAtom } from '../stores/downloadStore';
import DownloadTable from './DownloadTable';
import DownloadDetailPanel from './DownloadDetailPanel';
import DevModeBanner from './DevModeBanner';
import { useSelectedDownload } from '../hooks/useSelectedDownload';

interface DownloadListProps {
  className?: string;
}

const MIN_PANEL_HEIGHT = 200; // Minimum height for detail panel
const DEFAULT_PANEL_HEIGHT = 300; // Default height for detail panel

const DownloadList: React.FC<DownloadListProps> = ({ className }) => {
  const [tasks] = useAtom(downloadTasksAtom);
  const { selectedTask } = useSelectedDownload();
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
      {/* Dev Mode Banner - only shows in dev mode with mock features enabled */}
      <DevModeBanner />

      {/* Download Table - scrollable */}
      <Box
        sx={{
          flex: 1,
          overflow: 'auto',
          minHeight: 200,
        }}
      >
        <DownloadTable tasks={tasks} />
      </Box>

      {/* Resize Handle */}
      {selectedTask && (
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

          {/* Download Detail Panel - fixed height, scrollable content */}
          <Box
            sx={{
              height: `${panelHeight}px`,
              overflow: 'hidden',
              borderTop: 1,
              borderColor: 'divider',
              backgroundColor: 'background.paper',
            }}
          >
            <DownloadDetailPanel task={selectedTask} />
          </Box>
        </>
      )}
    </Box>
  );
};

export default DownloadList;
