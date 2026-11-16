// Basic info tab: URL, file path, size and timestamps.
import React, { useState } from 'react';
import { Box, Button, Chip, Link } from '@mui/material';
import { Edit as EditIcon } from '@mui/icons-material';
import { useTranslation } from 'react-i18next';
import type { Task } from '../backend';
import { formatBytes } from '../utils/downloadCalculations';
import { formatAbsoluteTime } from '../utils/formatters';
import PropertyList from './PropertyList';
import EditTaskDialog, { editable } from './EditTaskDialog';
import { useCategories } from '../hooks/useCategories';

const BasicInfoTab: React.FC<{ task: Task }> = ({ task }) => {
  const { t } = useTranslation();
  const [editing, setEditing] = useState(false);
  const { categories } = useCategories();
  const inCategories = categories.filter((c) => task.tags.includes(c.id));

  return (
    <Box sx={{ p: 1.5, display: 'flex', gap: 1, alignItems: 'flex-start' }}>
      <Box sx={{ flex: 1, minWidth: 0 }}>
        <PropertyList
          items={[
            { label: t('downloads.details.url'), value: <Link href={task.url} target="_blank" rel="noreferrer" underline="hover">{task.url}</Link> },
            { label: t('downloads.details.filePath'), value: task.finalPath ?? `${task.dir}\\${task.name}`, mono: true },
            {
              label: t('downloads.details.fileSize'),
              value: task.total !== null ? `${formatBytes(task.total)} (${task.total.toLocaleString()} bytes)` : '--',
            },
            { label: t('downloads.details.createdAt'), value: formatAbsoluteTime(task.createdAt) },
            task.startedAt && { label: t('downloads.details.startedAt'), value: formatAbsoluteTime(task.startedAt) },
            task.state === 'completed' &&
              task.finishedAt && { label: t('downloads.details.completedAt'), value: formatAbsoluteTime(task.finishedAt) },
            inCategories.length > 0 && {
              label: t('sidebar.categories'),
              value: (
                <Box sx={{ display: 'flex', gap: 0.5, flexWrap: 'wrap' }}>
                  {inCategories.map((c) => (
                    <Chip key={c.id} label={c.name} variant="outlined" />
                  ))}
                </Box>
              ),
            },
          ]}
        />
      </Box>
      {editable(task) && (
        <Button color="inherit" startIcon={<EditIcon fontSize="small" />} onClick={() => setEditing(true)}>
          {t('downloads.edit.title')}
        </Button>
      )}
      <EditTaskDialog task={editing ? task : null} onClose={() => setEditing(false)} />
    </Box>
  );
};

export default BasicInfoTab;
