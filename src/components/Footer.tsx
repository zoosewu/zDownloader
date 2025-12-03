// Footer component - displays app info and mode controls
import React from 'react';
import { Box, Typography, Chip } from '@mui/material';
import { useAtomValue } from 'jotai';
import { useTranslation } from 'react-i18next';
import {
  activeDownloadsAtom,
  downloadTasksAtom,
  completedDownloadsAtom,
} from '../stores/downloadStore';
import { devModeStatusAtom } from '../stores/devStore';

/**
 * Footer - displays at the bottom of the app
 * Shows download statistics and development mode status
 */
const Footer: React.FC = () => {
  const { t } = useTranslation();
  const tasks = useAtomValue(downloadTasksAtom);
  const activeDownloads = useAtomValue(activeDownloadsAtom);
  const completedDownloads = useAtomValue(completedDownloadsAtom);
  const devStatus = useAtomValue(devModeStatusAtom);

  // Calculate total download speed
  const totalSpeed = activeDownloads.reduce((sum, task) => sum + task.currentSpeed, 0);
  const speedMBps = (totalSpeed / 1024 / 1024).toFixed(2);

  return (
    <Box
      sx={{
        height: '32px',
        display: 'flex',
        alignItems: 'center',
        justifyContent: 'space-between',
        px: 2,
        borderTop: 1,
        borderColor: 'divider',
        backgroundColor: 'background.paper',
        fontSize: '0.75rem',
      }}
    >
      {/* Left side - Download statistics */}
      <Box sx={{ display: 'flex', gap: 3, alignItems: 'center' }}>
        <Typography variant="caption">
          {t('downloads.statistics.totalDownloaded')}: <strong>{tasks.length}</strong>
        </Typography>
        <Typography variant="caption">
          {t('downloads.statistics.activeDownloads')}: <strong>{activeDownloads.length}</strong>
        </Typography>
        <Typography variant="caption">
          {t('downloads.statistics.completedDownloads')}: <strong>{completedDownloads.length}</strong>
        </Typography>
        {activeDownloads.length > 0 && (
          <Typography variant="caption">
            {t('downloads.statistics.currentSpeed')}: <strong>{speedMBps} MB/s</strong>
          </Typography>
        )}
      </Box>

      {/* Right side - Dev mode indicator and app info */}
      <Box sx={{ display: 'flex', gap: 2, alignItems: 'center' }}>
        {devStatus.isActive && (
          <Chip
            label={t('dev.banner.title')}
            size="small"
            color="warning"
            sx={{ height: '20px', fontSize: '0.7rem' }}
          />
        )}
        <Typography variant="caption" color="text.secondary">
          Zoo Downloader v0.1.0
        </Typography>
      </Box>
    </Box>
  );
};

export default Footer;
