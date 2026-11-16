// Queue controls component - pause all, resume all, clear completed
import React from 'react';
import { Button } from '@mui/material';
import {
  PauseCircle as PauseIcon,
  PlayCircle as PlayIcon,
  DeleteSweep as ClearIcon,
} from '@mui/icons-material';
import { useTranslation } from 'react-i18next';
import { useAtomValue } from 'jotai';
import { useDownloadActions } from '../hooks/useDownloadActions';
import { viewCountsAtom } from '../stores/viewStore';

interface QueueControlsProps {
  onControlError?: (error: string) => void;
}

const QueueControls: React.FC<QueueControlsProps> = ({ onControlError }) => {
  const { t } = useTranslation();
  // Counts without the speed: a progress tick must not re-render the toolbar.
  const counts = useAtomValue(viewCountsAtom);
  const activeCount = counts.running + counts.waiting;
  const pausedCount = counts.paused + counts.failed;
  const completedCount = counts.completed + counts.cancelled;
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

  // Counts live in the sidebar and the status bar; the toolbar only enables what can run.
  return (
    <>
      <Button
        color="inherit"
        startIcon={<PlayIcon fontSize="small" />}
        onClick={handleResumeAll}
        disabled={pausedCount === 0 || loading !== null}
      >
        {t('downloads.actions.resumeAll')}
      </Button>
        <Button
          color="inherit"
          startIcon={<PauseIcon fontSize="small" />}
          onClick={handlePauseAll}
          disabled={activeCount === 0 || loading !== null}
        >
          {t('downloads.actions.pauseAll')}
        </Button>
        <Button
          color="inherit"
          startIcon={<ClearIcon fontSize="small" />}
          onClick={handleClearCompleted}
          disabled={completedCount === 0 || loading !== null}
        >
          {t('downloads.actions.clearCompleted')}
        </Button>
    </>
  );
};

export default QueueControls;
