// Advanced info tab component - displays retry information and scheduled time
import React from 'react';
import { Box, Typography, Stack, Chip } from '@mui/material';
import { useTranslation } from 'react-i18next';
import { DownloadTask } from '../types/download';
import { formatAbsoluteTime } from '../utils/formatters';

interface AdvancedInfoTabProps {
  task: DownloadTask;
}

/**
 * Advanced info tab - displays retry info, scheduled start time, and other advanced settings
 */
const AdvancedInfoTab: React.FC<AdvancedInfoTabProps> = ({ task }) => {
  const { t } = useTranslation();

  return (
    <Box sx={{ padding: 3 }}>
      <Stack spacing={2.5}>
        {/* Retry Information */}
        <Box>
          <Typography variant="subtitle2" color="text.secondary" gutterBottom>
            {t('downloads.details.retryInfo')}
          </Typography>
          <Stack direction="row" spacing={1} alignItems="center" sx={{ marginTop: 1 }}>
            <Chip
              label={`${t('downloads.details.retryCount')}: ${task.retryCount}`}
              size="small"
              variant="outlined"
              color={task.retryCount > 0 ? 'warning' : 'default'}
            />
            <Chip
              label={`${t('downloads.details.maxRetries')}: ${task.maxRetries || 3}`}
              size="small"
              variant="outlined"
            />
          </Stack>
        </Box>

        {/* Last Retry Time */}
        {task.lastRetryAt && (
          <Box>
            <Typography variant="subtitle2" color="text.secondary" gutterBottom>
              {t('downloads.details.lastRetryAt')}
            </Typography>
            <Typography variant="body2">
              {formatAbsoluteTime(task.lastRetryAt)}
            </Typography>
          </Box>
        )}

        {/* Scheduled Start Time */}
        <Box>
          <Typography variant="subtitle2" color="text.secondary" gutterBottom>
            {t('downloads.details.scheduledStart')}
          </Typography>
          {task.scheduledStart ? (
            <Typography variant="body2">
              {formatAbsoluteTime(task.scheduledStart)}
            </Typography>
          ) : (
            <Typography variant="body2" color="text.secondary" fontStyle="italic">
              {t('downloads.details.notScheduled')}
            </Typography>
          )}
        </Box>

        {/* Segments Count */}
        {task.segments && task.segments.length > 0 && (
          <Box>
            <Typography variant="subtitle2" color="text.secondary" gutterBottom>
              {t('downloads.details.segments')}
            </Typography>
            <Chip
              label={t('downloads.details.segmentCount', { count: task.segments.length })}
              size="small"
              color="primary"
              variant="outlined"
            />
          </Box>
        )}

        {/* Task ID (for debugging) */}
        <Box>
          <Typography variant="subtitle2" color="text.secondary" gutterBottom>
            Task ID
          </Typography>
          <Typography
            variant="body2"
            sx={{
              fontFamily: 'monospace',
              fontSize: '0.75rem',
              backgroundColor: 'action.hover',
              padding: 1,
              borderRadius: 1,
              wordBreak: 'break-all',
            }}
          >
            {task.id}
          </Typography>
        </Box>
      </Stack>
    </Box>
  );
};

export default AdvancedInfoTab;
