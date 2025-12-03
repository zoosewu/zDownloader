// Error info tab component - displays error information for failed downloads
import React from 'react';
import { Box, Typography, Alert, Stack } from '@mui/material';
import { useTranslation } from 'react-i18next';
import { DownloadTask } from '../types/download';

interface ErrorInfoTabProps {
  task: DownloadTask;
}

/**
 * Error info tab - displays error message or "no error" message
 */
const ErrorInfoTab: React.FC<ErrorInfoTabProps> = ({ task }) => {
  const { t } = useTranslation();

  const hasError = task.status === 'Failed' && task.errorMessage;

  return (
    <Box sx={{ padding: 3 }}>
      {hasError ? (
        <Stack spacing={2}>
          {/* Error Status */}
          <Alert severity="error" variant="outlined">
            <Typography variant="subtitle2" gutterBottom>
              {t('downloads.status.failed')}
            </Typography>
            <Typography variant="body2" sx={{ whiteSpace: 'pre-wrap', wordBreak: 'break-word' }}>
              {task.errorMessage}
            </Typography>
          </Alert>

          {/* Additional Error Details */}
          {task.retryCount > 0 && (
            <Box>
              <Typography variant="subtitle2" color="text.secondary" gutterBottom>
                {t('downloads.details.retryCount')}
              </Typography>
              <Typography variant="body2">
                {task.retryCount} / {task.maxRetries || 3}
              </Typography>
            </Box>
          )}
        </Stack>
      ) : (
        <Box
          sx={{
            display: 'flex',
            justifyContent: 'center',
            alignItems: 'center',
            minHeight: 150,
          }}
        >
          <Alert severity="success" variant="outlined">
            <Typography variant="body2">{t('downloads.details.noError')}</Typography>
          </Alert>
        </Box>
      )}
    </Box>
  );
};

export default ErrorInfoTab;
