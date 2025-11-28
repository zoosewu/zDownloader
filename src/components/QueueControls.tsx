// Queue controls component - pause all, resume all, clear completed
import React from 'react';
import { Box, Button, ButtonGroup, Chip } from '@mui/material';
import {
  PauseCircle as PauseIcon,
  PlayCircle as PlayIcon,
  DeleteSweep as ClearIcon,
} from '@mui/icons-material';
import { useTranslation } from 'react-i18next';
import { invoke } from '@tauri-apps/api/core';

interface QueueControlsProps {
  activeCount: number;
  pausedCount: number;
  completedCount: number;
  onControlSuccess?: () => void;
  onControlError?: (error: string) => void;
}

const QueueControls: React.FC<QueueControlsProps> = ({
  activeCount,
  pausedCount,
  completedCount,
  onControlSuccess,
  onControlError,
}) => {
  const { t } = useTranslation();
  const [loading, setLoading] = React.useState<string | null>(null);

  const handlePauseAll = async () => {
    try {
      setLoading('pause');
      await invoke('pause_all_downloads');
      onControlSuccess?.();
    } catch (error) {
      console.error('Failed to pause all downloads:', error);
      onControlError?.(
        error instanceof Error ? error.message : 'Unknown error'
      );
    } finally {
      setLoading(null);
    }
  };

  const handleResumeAll = async () => {
    try {
      setLoading('resume');
      await invoke('resume_all_downloads');
      onControlSuccess?.();
    } catch (error) {
      console.error('Failed to resume all downloads:', error);
      onControlError?.(
        error instanceof Error ? error.message : 'Unknown error'
      );
    } finally {
      setLoading(null);
    }
  };

  const handleClearCompleted = async () => {
    try {
      setLoading('clear');
      const removedCount = await invoke<number>('clear_completed_downloads');
      console.log(`Cleared ${removedCount} completed downloads`);
      onControlSuccess?.();
    } catch (error) {
      console.error('Failed to clear completed downloads:', error);
      onControlError?.(
        error instanceof Error ? error.message : 'Unknown error'
      );
    } finally {
      setLoading(null);
    }
  };

  return (
    <Box
      sx={{
        display: 'flex',
        alignItems: 'center',
        gap: 2,
        flexWrap: 'wrap',
      }}
    >
      {/* Status chips */}
      <Box sx={{ display: 'flex', gap: 1 }}>
        <Chip
          label={`${t('downloads.status.active')}: ${activeCount}`}
          color="primary"
          size="small"
        />
        <Chip
          label={`${t('downloads.status.paused')}: ${pausedCount}`}
          color="warning"
          size="small"
        />
        <Chip
          label={`${t('downloads.status.completed')}: ${completedCount}`}
          color="success"
          size="small"
        />
      </Box>

      {/* Control buttons */}
      <ButtonGroup size="small" variant="outlined">
        <Button
          startIcon={<PauseIcon />}
          onClick={handlePauseAll}
          disabled={activeCount === 0 || loading !== null}
        >
          {t('downloads.actions.pauseAll')}
        </Button>
        <Button
          startIcon={<PlayIcon />}
          onClick={handleResumeAll}
          disabled={pausedCount === 0 || loading !== null}
        >
          {t('downloads.actions.resumeAll')}
        </Button>
        <Button
          startIcon={<ClearIcon />}
          onClick={handleClearCompleted}
          disabled={completedCount === 0 || loading !== null}
        >
          {t('downloads.actions.clearCompleted')}
        </Button>
      </ButtonGroup>
    </Box>
  );
};

export default QueueControls;
