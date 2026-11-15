// Main page - download list with queue controls, batch import and detail panel
import React from 'react';
import { Box, Snackbar, Alert } from '@mui/material';
import { useTranslation } from 'react-i18next';
import DownloadList from '../components/DownloadList';
import { useNavigation } from '../hooks/useNavigation';
import type { BatchResult } from '../backend';

type Severity = 'success' | 'error' | 'info' | 'warning';

const MainPage: React.FC = () => {
  const { t } = useTranslation();
  const { navigateTo } = useNavigation();

  const [snackbar, setSnackbar] = React.useState<{ open: boolean; message: string; severity: Severity }>({
    open: false,
    message: '',
    severity: 'info',
  });
  const show = (message: string, severity: Severity) => setSnackbar({ open: true, message, severity });

  const handleBatchImportSuccess = (result: BatchResult) => {
    const message = t('downloads.batch.importSuccess', { count: result.added.length });
    if (result.failed.length === 0) {
      show(message, 'success');
    } else {
      show(`${message}; ${t('downloads.batch.importSkipped', { count: result.failed.length })}`, 'warning');
    }
  };

  // Stable: it reaches every table row, and a new function each render would defeat their memo.
  const handleError = React.useCallback(
    (error: string) => setSnackbar({ open: true, message: t('downloads.errors.queueControlFailed', { error }), severity: 'error' }),
    [t],
  );

  return (
    <Box sx={{ width: '100%', height: '100%', display: 'flex', flexDirection: 'column' }}>
      <Box sx={{ flex: 1, overflow: 'hidden' }}>
        <DownloadList
          onControlError={handleError}
          onAddDownload={() => navigateTo('add-download')}
          onBatchImportSuccess={handleBatchImportSuccess}
          onBatchImportError={(error) => show(t('downloads.batch.importError', { error }), 'error')}
        />
      </Box>

      <Snackbar
        open={snackbar.open}
        autoHideDuration={6000}
        onClose={() => setSnackbar((s) => ({ ...s, open: false }))}
        anchorOrigin={{ vertical: 'bottom', horizontal: 'right' }}
      >
        <Alert
          onClose={() => setSnackbar((s) => ({ ...s, open: false }))}
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
