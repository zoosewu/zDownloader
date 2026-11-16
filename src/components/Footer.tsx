// Footer component - displays app info and mode controls
import React from 'react';
import { Box, Typography } from '@mui/material';
import { useAtomValue } from 'jotai';
import { useTranslation } from 'react-i18next';
import { taskCountsAtom } from '../stores/downloadStore';

/**
 * Footer - displays at the bottom of the app
 * Shows download statistics
 */
const Footer: React.FC = () => {
  const { t } = useTranslation();
  const counts = useAtomValue(taskCountsAtom);
  const speedMBps = (counts.speed / 1024 / 1024).toFixed(2);

  return (
    <Box
      sx={{
        height: 24,
        display: 'flex',
        alignItems: 'center',
        justifyContent: 'space-between',
        px: 1.5,
        borderTop: 1,
        borderColor: 'divider',
        backgroundColor: 'background.paper',
        fontSize: '0.75rem',
      }}
    >
      {/* Left side - Download statistics */}
      <Box sx={{ display: 'flex', gap: 3, alignItems: 'center' }}>
        <Typography variant="caption">
          {t('downloads.statistics.totalTasks')}: <strong>{counts.total}</strong>
        </Typography>
        <Typography variant="caption">
          {t('downloads.statistics.activeDownloads')}: <strong>{counts.running}</strong>
        </Typography>
        <Typography variant="caption">
          {t('downloads.statistics.completedDownloads')}: <strong>{counts.completed}</strong>
        </Typography>
        {counts.running > 0 && (
          <Typography variant="caption">
            {t('downloads.statistics.currentSpeed')}: <strong>{speedMBps} MB/s</strong>
          </Typography>
        )}
      </Box>

      {/* Right side - app info */}
      <Box sx={{ display: 'flex', gap: 2, alignItems: 'center' }}>
        <Typography variant="caption" color="text.secondary">
          Zoo Downloader v0.1.0
        </Typography>
      </Box>
    </Box>
  );
};

export default Footer;
