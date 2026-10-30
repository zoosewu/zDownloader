// Queue controls component - pause all, resume all, clear completed
import React from 'react';
import { Box, Button, ButtonGroup, Chip } from '@mui/material';
import {
  PauseCircle as PauseIcon,
  PlayCircle as PlayIcon,
  DeleteSweep as ClearIcon,
} from '@mui/icons-material';
import { useTranslation } from 'react-i18next';
import { useDownloadActions } from '../hooks/useDownloadActions';

interface QueueControlsProps {
  activeCount: number;
  pausedCount: number;
  completedCount: number;
  onControlError?: (error: string) => void;
}

const QueueControls: React.FC<QueueControlsProps> = ({
  activeCount,
  pausedCount,
  completedCount,
  onControlError,
}) => {
  const { t } = useTranslation();
  const actions = useDownloadActions();
  const [loading, setLoading] = React.useState<string | null>(null);

  const run = (name: string, action: () => Promise<void>) => async () => {
    setLoading(name);
    try {
      await action();
    } catch (error) {
      onControlError?.(error instanceof Error ? error.message : String(error));
    } finally {
      setLoading(null);
    }
  };

  const handlePauseAll = run('pause', actions.pauseAll);
  const handleResumeAll = run('resume', actions.resumeAll);
  const handleClearCompleted = run('clear', actions.clearFinished);

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
