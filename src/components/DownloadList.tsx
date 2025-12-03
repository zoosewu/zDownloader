// Download list component - displays all download tasks in table view
import React from 'react';
import { Box, Typography, Paper } from '@mui/material';
import { useAtom } from 'jotai';
import { downloadTasksAtom } from '../stores/downloadStore';
import DownloadTable from './DownloadTable';
import { useTranslation } from 'react-i18next';

interface DownloadListProps {
  className?: string;
}

const DownloadList: React.FC<DownloadListProps> = ({ className }) => {
  const { t } = useTranslation();
  const [tasks] = useAtom(downloadTasksAtom);

  return (
    <Box className={className} sx={{ width: '100%', display: 'flex', flexDirection: 'column', gap: 2 }}>
      {/* Header */}
      <Paper
        elevation={0}
        sx={{
          backgroundColor: 'background.default',
          padding: 2,
        }}
      >
        <Typography variant="h6">
          {t('downloads.title')} ({tasks.length})
        </Typography>
      </Paper>

      {/* Download Table */}
      <DownloadTable tasks={tasks} />
    </Box>
  );
};

export default DownloadList;
