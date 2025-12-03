// Basic info tab component - displays basic download information
import React from 'react';
import { Box, Typography, Stack, Link } from '@mui/material';
import { useTranslation } from 'react-i18next';
import { DownloadTask } from '../types/download';
import { formatBytes } from '../utils/downloadCalculations';
import { formatAbsoluteTime } from '../utils/formatters';

interface BasicInfoTabProps {
  task: DownloadTask;
}

/**
 * Basic info tab - displays URL, file path, file size, and timestamps
 */
const BasicInfoTab: React.FC<BasicInfoTabProps> = ({ task }) => {
  const { t } = useTranslation();

  return (
    <Box sx={{ padding: 3 }}>
      <Stack spacing={2.5}>
        {/* URL */}
        <Box>
          <Typography variant="subtitle2" color="text.secondary" gutterBottom>
            {t('downloads.details.url')}
          </Typography>
          <Link
            href={task.url}
            target="_blank"
            rel="noopener noreferrer"
            sx={{
              wordBreak: 'break-all',
              fontSize: '0.875rem',
            }}
          >
            {task.url}
          </Link>
        </Box>

        {/* File Path */}
        <Box>
          <Typography variant="subtitle2" color="text.secondary" gutterBottom>
            {t('downloads.details.filePath')}
          </Typography>
          <Typography
            variant="body2"
            sx={{
              fontFamily: 'monospace',
              wordBreak: 'break-all',
              backgroundColor: 'action.hover',
              padding: 1,
              borderRadius: 1,
            }}
          >
            {task.filepath || task.filename}
          </Typography>
        </Box>

        {/* File Size */}
        <Box>
          <Typography variant="subtitle2" color="text.secondary" gutterBottom>
            {t('downloads.details.fileSize')}
          </Typography>
          <Typography variant="body2">
            {formatBytes(task.filesize)} ({task.filesize.toLocaleString()} bytes)
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
        {task.completedAt && (
          <Box>
            <Typography variant="subtitle2" color="text.secondary" gutterBottom>
              {t('downloads.details.completedAt')}
            </Typography>
            <Typography variant="body2">
              {formatAbsoluteTime(task.completedAt)}
            </Typography>
          </Box>
        )}
      </Stack>
    </Box>
  );
};

export default BasicInfoTab;
