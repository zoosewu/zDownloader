// Advanced info tab component - displays retry information and scheduled time
import React from 'react';
import { Box, Typography, Stack, Chip } from '@mui/material';
import { useTranslation } from 'react-i18next';
import type { Task } from '../backend';
import { formatAbsoluteTime } from '../utils/formatters';

interface AdvancedInfoTabProps {
  task: Task;
}

const AdvancedInfoTab: React.FC<AdvancedInfoTabProps> = ({ task }) => {
  const { t } = useTranslation();

  return (
    <Box sx={{ padding: 3 }}>
      <Stack spacing={2.5}>
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
            {task.nextRetryAt && (
              <Chip
                label={`${t('downloads.details.nextRetryAt')}: ${formatAbsoluteTime(task.nextRetryAt)}`}
                size="small"
                variant="outlined"
              />
            )}
          </Stack>
        </Box>

        <Box>
          <Typography variant="subtitle2" color="text.secondary" gutterBottom>
            {t('downloads.details.scheduledStart')}
          </Typography>
          {task.scheduledAt ? (
            <Typography variant="body2">{formatAbsoluteTime(task.scheduledAt)}</Typography>
          ) : (
            <Typography variant="body2" color="text.secondary" fontStyle="italic">
              {t('downloads.details.notScheduled')}
            </Typography>
          )}
        </Box>

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
