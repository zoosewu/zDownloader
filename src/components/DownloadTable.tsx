// Download table component - table view for all download items
import React from 'react';
import {
  Table,
  TableBody,
  TableCell,
  TableContainer,
  TableHead,
  TableRow,
  Paper,
  Box,
  Typography,
} from '@mui/material';
import { useTranslation } from 'react-i18next';
import { DownloadTask } from '../types/download';
import DownloadTableRow from './DownloadTableRow';
import { useSelectedDownload } from '../hooks/useSelectedDownload';

interface DownloadTableProps {
  tasks: DownloadTask[];
}

/**
 * Download table component with sticky header
 * Displays all download tasks in a tabular format
 */
const DownloadTable: React.FC<DownloadTableProps> = ({ tasks }) => {
  const { t } = useTranslation();
  const { selectedTaskId, toggleSelection } = useSelectedDownload();

  // Handle row click - toggle selection
  const handleRowClick = (taskId: string) => {
    toggleSelection(taskId);
  };

  // Empty state
  if (tasks.length === 0) {
    return (
      <Box
        sx={{
          display: 'flex',
          justifyContent: 'center',
          alignItems: 'center',
          minHeight: 200,
          padding: 4,
        }}
      >
        <Typography variant="body1" color="text.secondary">
          {t('downloads.noTasks')}
        </Typography>
      </Box>
    );
  }

  return (
    <TableContainer
      component={Paper}
      sx={{
        maxHeight: 'calc(100vh - 300px)', // Allow vertical scrolling
        overflow: 'auto',
      }}
    >
      <Table stickyHeader aria-label="download table">
        <TableHead>
          <TableRow>
            {/* Column Headers */}
            <TableCell sx={{ fontWeight: 'bold', minWidth: 200 }}>
              {t('downloads.details.filename', '檔案名稱')}
            </TableCell>
            <TableCell align="right" sx={{ fontWeight: 'bold', minWidth: 100 }}>
              {t('downloads.details.fileSize')}
            </TableCell>
            <TableCell sx={{ fontWeight: 'bold', minWidth: 180 }}>
              {t('common.progress', '進度')}
            </TableCell>
            <TableCell align="right" sx={{ fontWeight: 'bold', minWidth: 100 }}>
              {t('downloads.statistics.currentSpeed')}
            </TableCell>
            <TableCell sx={{ fontWeight: 'bold', minWidth: 120 }}>
              {t('downloads.status.title', '狀態')}
            </TableCell>
            <TableCell align="center" sx={{ fontWeight: 'bold', minWidth: 80 }}>
              {t('downloads.priority.title', '優先度')}
            </TableCell>
            <TableCell sx={{ fontWeight: 'bold', minWidth: 130 }}>
              {t('downloads.details.createdAt')}
            </TableCell>
            <TableCell align="right" sx={{ fontWeight: 'bold', minWidth: 120 }}>
              {t('downloads.statistics.timeRemaining')}
            </TableCell>
          </TableRow>
        </TableHead>
        <TableBody>
          {tasks.map((task) => (
            <DownloadTableRow
              key={task.id}
              task={task}
              isSelected={selectedTaskId === task.id}
              onClick={handleRowClick}
            />
          ))}
        </TableBody>
      </Table>
    </TableContainer>
  );
};

export default DownloadTable;
