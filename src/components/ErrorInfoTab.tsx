// Error info tab component - displays error information for failed downloads
import React from 'react';
import { Box, Typography, Alert, Stack } from '@mui/material';
import { useTranslation } from 'react-i18next';
import type { Task } from '../backend';
import { errorText } from './DownloadTableRow';
import { formatAbsoluteTime } from '../utils/formatters';

interface ErrorInfoTabProps {
  task: Task;
}

const ErrorInfoTab: React.FC<ErrorInfoTabProps> = ({ task }) => {
  const { t } = useTranslation();

  if (!task.error) {
    return (
      <Box sx={{ padding: 3, display: 'flex', justifyContent: 'center', alignItems: 'center', minHeight: 150 }}>
        <Alert severity="success" variant="outlined">
          <Typography variant="body2">{t('downloads.details.noError')}</Typography>
        </Alert>
      </Box>
    );
  }

  return (
    <Box sx={{ padding: 3 }}>
      <Stack spacing={2}>
        <Alert severity="error" variant="outlined">
          <Typography variant="subtitle2" gutterBottom>
            {t(`downloads.state.${task.state}`)}
          </Typography>
          <Typography variant="body2" sx={{ whiteSpace: 'pre-wrap', wordBreak: 'break-word' }}>
            {errorText(task)}
          </Typography>
        </Alert>

        {task.retryCount > 0 && (
          <Box>
            <Typography variant="subtitle2" color="text.secondary" gutterBottom>
              {t('downloads.details.retryCount')}
            </Typography>
            <Typography variant="body2">{task.retryCount}</Typography>
          </Box>
        )}

        {task.nextRetryAt && (
          <Box>
            <Typography variant="subtitle2" color="text.secondary" gutterBottom>
              {t('downloads.details.nextRetryAt')}
            </Typography>
            <Typography variant="body2">{formatAbsoluteTime(task.nextRetryAt)}</Typography>
          </Box>
        )}
      </Stack>
    </Box>
  );
};

export default ErrorInfoTab;
