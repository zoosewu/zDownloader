// Segment visualization component - grid display of all download segments
import React from 'react';
import { Box, Typography, Alert } from '@mui/material';
import { useTranslation } from 'react-i18next';
import { DownloadTask } from '../types/download';
import { formatSegmentForVisualization } from '../utils/downloadCalculations';
import SegmentGridItem from './SegmentGridItem';

interface SegmentVisualizationProps {
  task: DownloadTask;
}

/**
 * Segment visualization - displays all segments as colored squares in a grid
 * Colors: Grey (Pending), Blue (Downloading), Green (Completed), Red (Failed)
 */
const SegmentVisualization: React.FC<SegmentVisualizationProps> = ({ task }) => {
  const { t } = useTranslation();

  // No segments available
  if (!task.segments || task.segments.length === 0) {
    return (
      <Box
        sx={{
          display: 'flex',
          justifyContent: 'center',
          alignItems: 'center',
          minHeight: 150,
          padding: 3,
        }}
      >
        <Alert severity="info" variant="outlined">
          <Typography variant="body2">{t('downloads.details.noSegments')}</Typography>
        </Alert>
      </Box>
    );
  }

  // Format segments for visualization
  const visualizationData = task.segments.map((segment) =>
    formatSegmentForVisualization(segment, t)
  );

  return (
    <Box sx={{ padding: 3 }}>
      {/* Header */}
      <Typography variant="subtitle2" color="text.secondary" gutterBottom>
        {t('downloads.details.segmentCount', { count: task.segments.length })}
      </Typography>

      {/* Segment Grid */}
      <Box
        sx={{
          display: 'grid',
          gridTemplateColumns: 'repeat(auto-fill, 12px)',
          gap: '2px',
          marginTop: 2,
          maxHeight: 400,
          overflowY: 'auto',
          padding: 1,
          backgroundColor: 'background.default',
          borderRadius: 1,
        }}
      >
        {visualizationData.map((data) => (
          <SegmentGridItem key={data.segmentId} data={data} />
        ))}
      </Box>

      {/* Legend */}
      <Box sx={{ marginTop: 3, display: 'flex', gap: 3, flexWrap: 'wrap' }}>
        <Box sx={{ display: 'flex', alignItems: 'center', gap: 1 }}>
          <Box
            sx={{
              width: 12,
              height: 12,
              backgroundColor: '#9e9e9e',
              borderRadius: 0.5,
            }}
          />
          <Typography variant="caption">{t('downloads.status.pending')}</Typography>
        </Box>
        <Box sx={{ display: 'flex', alignItems: 'center', gap: 1 }}>
          <Box
            sx={{
              width: 12,
              height: 12,
              backgroundColor: '#2196f3',
              borderRadius: 0.5,
            }}
          />
          <Typography variant="caption">{t('downloads.status.downloading')}</Typography>
        </Box>
        <Box sx={{ display: 'flex', alignItems: 'center', gap: 1 }}>
          <Box
            sx={{
              width: 12,
              height: 12,
              backgroundColor: '#4caf50',
              borderRadius: 0.5,
            }}
          />
          <Typography variant="caption">{t('downloads.status.completed')}</Typography>
        </Box>
        <Box sx={{ display: 'flex', alignItems: 'center', gap: 1 }}>
          <Box
            sx={{
              width: 12,
              height: 12,
              backgroundColor: '#f44336',
              borderRadius: 0.5,
            }}
          />
          <Typography variant="caption">{t('downloads.status.failed')}</Typography>
        </Box>
      </Box>
    </Box>
  );
};

export default SegmentVisualization;
