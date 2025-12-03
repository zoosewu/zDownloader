// Dev mode banner component - shows when mock mode or mock tasks are enabled
import React from 'react';
import { Box, Alert, AlertTitle, Chip, Stack } from '@mui/material';
import { useAtomValue } from 'jotai';
import { useTranslation } from 'react-i18next';
import { devModeStatusAtom } from '../stores/devStore';

/**
 * DevModeBanner - displays development mode status
 * Shows a prominent banner when running in dev mode with mock features enabled
 */
const DevModeBanner: React.FC = () => {
  const { t } = useTranslation();
  const devStatus = useAtomValue(devModeStatusAtom);

  // Don't show banner if not in dev mode or no dev features enabled
  if (!devStatus.isActive) {
    return null;
  }

  // Build status message
  const statusItems: string[] = [];
  if (devStatus.useMockTasks) {
    statusItems.push(t('dev.banner.usingMockTasks'));
  }
  if (devStatus.mockMode) {
    statusItems.push(t('dev.banner.mockModeEnabled'));
  }
  if (devStatus.autoProgress) {
    statusItems.push(t('dev.banner.autoProgressEnabled'));
  }

  return (
    <Box sx={{ position: 'sticky', top: 0, zIndex: 1200, mb: 2 }}>
      <Alert severity="warning" variant="filled">
        <AlertTitle sx={{ fontWeight: 'bold' }}>
          {t('dev.banner.title')}
        </AlertTitle>
        <Stack direction="row" spacing={1} flexWrap="wrap" sx={{ mt: 1 }}>
          {devStatus.useMockTasks && (
            <Chip
              label={t('dev.banner.usingMockTasks')}
              size="small"
              sx={{
                backgroundColor: 'rgba(255, 255, 255, 0.2)',
                color: 'inherit',
              }}
            />
          )}
          {devStatus.mockMode && (
            <Chip
              label={t('dev.banner.mockModeEnabled')}
              size="small"
              sx={{
                backgroundColor: 'rgba(255, 255, 255, 0.2)',
                color: 'inherit',
              }}
            />
          )}
          {devStatus.autoProgress && (
            <Chip
              label={t('dev.banner.autoProgressEnabled')}
              size="small"
              sx={{
                backgroundColor: 'rgba(255, 255, 255, 0.2)',
                color: 'inherit',
              }}
            />
          )}
        </Stack>
        <Box sx={{ mt: 1, fontSize: '0.875rem', opacity: 0.9 }}>
          {t('dev.banner.description')}
        </Box>
      </Alert>
    </Box>
  );
};

export default DevModeBanner;
