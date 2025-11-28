// Download list component - displays all download tasks
import React from 'react';
import { Box, Typography, Paper } from '@mui/material';
import { useAtom } from 'jotai';
import { downloadTasksAtom } from '../stores/downloadStore';
import DownloadItem from './DownloadItem';
import { useTranslation } from 'react-i18next';

interface DownloadListProps {
  className?: string;
}

const DownloadList: React.FC<DownloadListProps> = ({ className }) => {
  const { t } = useTranslation();
  const [tasks] = useAtom(downloadTasksAtom);

  if (tasks.length === 0) {
    return (
      <Box
        className={className}
        sx={{
          display: 'flex',
          justifyContent: 'center',
          alignItems: 'center',
          minHeight: '200px',
          padding: 3,
        }}
      >
        <Typography variant="body1" color="text.secondary">
          {t('downloads.noTasks')}
        </Typography>
      </Box>
    );
  }

  return (
    <Box className={className} sx={{ width: '100%' }}>
      <Paper
        elevation={0}
        sx={{
          backgroundColor: 'background.default',
          padding: 2,
        }}
      >
        <Typography variant="h6" sx={{ marginBottom: 2 }}>
          {t('downloads.title')} ({tasks.length})
        </Typography>
        <Box sx={{ display: 'flex', flexDirection: 'column', gap: 2 }}>
          {tasks.map((task) => (
            <DownloadItem key={task.id} task={task} />
          ))}
        </Box>
      </Paper>
    </Box>
  );
};

export default DownloadList;
