import React, { useState } from 'react';
import {
  Dialog,
  DialogTitle,
  DialogContent,
  DialogActions,
  Button,
  List,
  ListItem,
  ListItemButton,
  ListItemText,
  ListItemIcon,
  Checkbox,
  Typography,
  Box,
  Divider,
  Chip,
} from '@mui/material';
import {
  CloudDownload as DownloadIcon,
  Info as InfoIcon,
} from '@mui/icons-material';
import { DownloadTask } from '../types/download';
import { useTranslation } from 'react-i18next';

interface ResumeDialogProps {
  open: boolean;
  tasks: DownloadTask[];
  onResumeAll: () => void;
  onResumeSelected: (taskIds: string[]) => void;
  onDiscardAll: () => void;
  onClose: () => void;
}

const ResumeDialog: React.FC<ResumeDialogProps> = ({
  open,
  tasks,
  onResumeAll,
  onResumeSelected,
  onDiscardAll,
  onClose,
}) => {
  const { t } = useTranslation();
  const [selectedTasks, setSelectedTasks] = useState<Set<string>>(
    new Set(tasks.map(task => task.id))
  );

  const handleToggleTask = (taskId: string) => {
    setSelectedTasks(prev => {
      const newSet = new Set(prev);
      if (newSet.has(taskId)) {
        newSet.delete(taskId);
      } else {
        newSet.add(taskId);
      }
      return newSet;
    });
  };

  const handleToggleAll = () => {
    if (selectedTasks.size === tasks.length) {
      setSelectedTasks(new Set());
    } else {
      setSelectedTasks(new Set(tasks.map(task => task.id)));
    }
  };

  const handleResumeSelected = () => {
    onResumeSelected(Array.from(selectedTasks));
    onClose();
  };

  const handleResumeAll = () => {
    onResumeAll();
    onClose();
  };

  const handleDiscardAll = () => {
    onDiscardAll();
    onClose();
  };

  const formatFileSize = (bytes: number): string => {
    if (bytes === 0) return '0 B';
    const k = 1024;
    const sizes = ['B', 'KB', 'MB', 'GB', 'TB'];
    const i = Math.floor(Math.log(bytes) / Math.log(k));
    return `${(bytes / Math.pow(k, i)).toFixed(2)} ${sizes[i]}`;
  };

  const calculateProgress = (task: DownloadTask): number => {
    if (task.segments.length === 0) {
      return task.progress;
    }
    const totalDownloaded = task.segments.reduce(
      (sum, segment) => sum + segment.downloadedBytes,
      0
    );
    return task.filesize > 0 ? (totalDownloaded / task.filesize) * 100 : 0;
  };

  return (
    <Dialog
      open={open}
      onClose={onClose}
      maxWidth="md"
      fullWidth
    >
      <DialogTitle>
        <Box display="flex" alignItems="center" gap={1}>
          <InfoIcon color="primary" />
          <Typography variant="h6">
            {t('resumeDialog.title', '發現未完成的下載')}
          </Typography>
        </Box>
      </DialogTitle>

      <DialogContent>
        <Typography variant="body2" color="text.secondary" paragraph>
          {t(
            'resumeDialog.description',
            '應用程式偵測到 {{count}} 個未完成的下載任務。您想要恢復這些下載嗎？',
            { count: tasks.length }
          )}
        </Typography>

        <Box mb={2}>
          <Button
            size="small"
            onClick={handleToggleAll}
            startIcon={<Checkbox checked={selectedTasks.size === tasks.length} />}
          >
            {selectedTasks.size === tasks.length
              ? t('resumeDialog.deselectAll', '取消全選')
              : t('resumeDialog.selectAll', '全選')}
          </Button>
          <Typography variant="caption" color="text.secondary" ml={2}>
            {t('resumeDialog.selected', '已選擇 {{count}} 個任務', {
              count: selectedTasks.size,
            })}
          </Typography>
        </Box>

        <Divider />

        <List sx={{ maxHeight: 400, overflow: 'auto' }}>
          {tasks.map((task) => {
            const progress = calculateProgress(task);
            const isSelected = selectedTasks.has(task.id);

            return (
              <ListItem
                key={task.id}
                disablePadding
                sx={{
                  borderLeft: isSelected ? 3 : 0,
                  borderColor: 'primary.main',
                }}
              >
                <ListItemButton
                  dense
                  onClick={() => handleToggleTask(task.id)}
                  sx={{
                    bgcolor: isSelected ? 'action.hover' : 'transparent',
                  }}
                >
                  <ListItemIcon>
                    <Checkbox
                      edge="start"
                      checked={isSelected}
                      tabIndex={-1}
                      disableRipple
                    />
                  </ListItemIcon>
                  <ListItemIcon>
                    <DownloadIcon color={isSelected ? 'primary' : 'disabled'} />
                  </ListItemIcon>
                  <ListItemText
                    primary={
                      <Box display="flex" alignItems="center" gap={1}>
                        <Typography variant="body2" noWrap>
                          {task.filename}
                        </Typography>
                        {task.segments.length > 0 && (
                          <Chip
                            label={t('resumeDialog.segmented', '分段: {{count}}', {
                              count: task.segments.length,
                            })}
                            size="small"
                            color="primary"
                            variant="outlined"
                          />
                        )}
                      </Box>
                    }
                    secondary={
                      <Box>
                        <Typography variant="caption" component="div">
                          {formatFileSize(task.filesize)} • {progress.toFixed(1)}%
                        </Typography>
                        {task.errorMessage && (
                          <Typography variant="caption" color="error">
                            {task.errorMessage}
                          </Typography>
                        )}
                      </Box>
                    }
                  />
                </ListItemButton>
              </ListItem>
            );
          })}
        </List>
      </DialogContent>

      <DialogActions>
        <Button onClick={handleDiscardAll} color="error">
          {t('resumeDialog.discardAll', '捨棄全部')}
        </Button>
        <Box flex={1} />
        <Button onClick={onClose}>
          {t('resumeDialog.later', '稍後處理')}
        </Button>
        <Button
          onClick={handleResumeSelected}
          variant="outlined"
          disabled={selectedTasks.size === 0}
        >
          {t('resumeDialog.resumeSelected', '恢復選擇 ({{count}})', {
            count: selectedTasks.size,
          })}
        </Button>
        <Button onClick={handleResumeAll} variant="contained" color="primary">
          {t('resumeDialog.resumeAll', '恢復全部')}
        </Button>
      </DialogActions>
    </Dialog>
  );
};

export default ResumeDialog;
