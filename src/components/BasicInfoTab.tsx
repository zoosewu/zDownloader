// Basic info tab component - displays basic download information
import React from 'react';
import { Box, Typography, Stack } from '@mui/material';
import { useTranslation } from 'react-i18next';
import type { Task } from '../backend';
import { formatBytes } from '../utils/downloadCalculations';
import { formatAbsoluteTime } from '../utils/formatters';
import CollapsibleField from './CollapsibleField';

interface BasicInfoTabProps {
  task: Task;
}

/**
 * Basic info tab - displays URL, file path, file size, and timestamps
 */
const BasicInfoTab: React.FC<BasicInfoTabProps> = ({ task }) => {
  const { t } = useTranslation();

  return (
    <Box sx={{ padding: 3 }}>
      <Stack spacing={0.5}>
        {/* URL - Collapsible */}
        <CollapsibleField
          label={t('downloads.details.url')}
          content={task.url}
          contentStyle="link"
        />

        {/* File Path - Collapsible */}
        <CollapsibleField
          label={t('downloads.details.filePath')}
          content={task.finalPath ?? `${task.dir}/${task.name}`}
          contentStyle="monospace"
        />

        {/* File Size */}
        <Box>
          <Typography variant="subtitle2" color="text.secondary" gutterBottom>
            {t('downloads.details.fileSize')}
          </Typography>
          <Typography variant="body2">
            {task.total !== null ? `${formatBytes(task.total)} (${task.total.toLocaleString()} bytes)` : '--'}
          </Typography>
        </Box>

        {/* Created At */}
        <Box>
          <Typography variant="subtitle2" color="text.secondary" gutterBottom>
            {t('downloads.details.createdAt')}
          </Typography>
          <Typography variant="body2">
            {formatAbsoluteTime(task.createdAt)}
          </Typography>
        </Box>

        {/* Started At */}
        {task.startedAt && (
          <Box>
            <Typography variant="subtitle2" color="text.secondary" gutterBottom>
              {t('downloads.details.startedAt')}
            </Typography>
            <Typography variant="body2">
              {formatAbsoluteTime(task.startedAt)}
            </Typography>
          </Box>
        )}

        {/* Completed At */}
        {task.state === 'completed' && task.finishedAt && (
          <Box>
            <Typography variant="subtitle2" color="text.secondary" gutterBottom>
              {t('downloads.details.completedAt')}
            </Typography>
            <Typography variant="body2">
              {formatAbsoluteTime(task.finishedAt)}
            </Typography>
          </Box>
        )}
      </Stack>
    </Box>
  );
};

export default BasicInfoTab;
