// Main page - integrates UrlInput, BatchImport, QueueControls and DownloadList
import React, { useEffect } from 'react';
import { Box, Snackbar, Alert } from '@mui/material';
import { useTranslation } from 'react-i18next';
import { useAtomValue } from 'jotai';
import DownloadList from '../components/DownloadList';
import useTauriEvents from '../hooks/useTauriEvents';
import useDownloads from '../hooks/useDownloads';
import { useNavigation } from '../hooks/useNavigation';
import {
  activeDownloadsAtom,
  pausedDownloadsAtom,
  completedDownloadsAtom,
} from '../stores/downloadStore';

const MainPage: React.FC = () => {
  const { t } = useTranslation();
  const { refreshDownloadList } = useDownloads();
  const { navigateTo } = useNavigation();

  // Get download counts for queue controls
  const activeDownloads = useAtomValue(activeDownloadsAtom);
  const pausedDownloads = useAtomValue(pausedDownloadsAtom);
  const completedDownloads = useAtomValue(completedDownloadsAtom);

  // Snackbar state for notifications
  const [snackbar, setSnackbar] = React.useState<{
    open: boolean;
    message: string;
    severity: 'success' | 'error' | 'info';
  }>({
    open: false,
    message: '',
    severity: 'info',
  });

  // Setup Tauri event listeners
  useTauriEvents();

  // Load initial download list
  useEffect(() => {
    // Add a small delay to ensure Tauri API is ready
    const timer = setTimeout(() => {
      refreshDownloadList();
    }, 100);

    return () => clearTimeout(timer);
  }, [refreshDownloadList]);

  const handleBatchImportSuccess = (taskIds: string[]) => {
    setSnackbar({
      open: true,
      message: t('downloads.batch.importSuccess', { count: taskIds.length }),
      severity: 'success',
    });
    refreshDownloadList();
  };

  const handleBatchImportError = (error: string) => {
    setSnackbar({
      open: true,
      message: t('downloads.batch.importError', { error }),
      severity: 'error',
    });
  };

  const handleQueueControlSuccess = () => {
    refreshDownloadList();
  };

  const handleQueueControlError = (error: string) => {
    setSnackbar({
      open: true,
      message: t('downloads.errors.queueControlFailed', { error }),
      severity: 'error',
    });
  };

  const handleCloseSnackbar = () => {
    setSnackbar({ ...snackbar, open: false });
  };

  return (
    <Box sx={{ width: '100%', height: '100%', display: 'flex', flexDirection: 'column' }}>
      {/* Download list - fills all space */}
      <Box sx={{ flex: 1, overflow: 'hidden' }}>
        <DownloadList
          activeCount={activeDownloads.length}
          pausedCount={pausedDownloads.length}
          completedCount={completedDownloads.length}
          onControlSuccess={handleQueueControlSuccess}
          onControlError={handleQueueControlError}
          onAddDownload={() => navigateTo('add-download')}
          onBatchImportSuccess={handleBatchImportSuccess}
          onBatchImportError={handleBatchImportError}
        />
      </Box>

      {/* Snackbar for notifications */}
      <Snackbar
        open={snackbar.open}
        autoHideDuration={6000}
        onClose={handleCloseSnackbar}
        anchorOrigin={{ vertical: 'bottom', horizontal: 'right' }}
      >
        <Alert
          onClose={handleCloseSnackbar}
          severity={snackbar.severity}
          sx={{ width: '100%' }}
        >
          {snackbar.message}
        </Alert>
      </Snackbar>
    </Box>
  );
};

export default MainPage;
