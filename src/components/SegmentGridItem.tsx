// Segment grid item component - single square representing a download segment
import React from 'react';
import { Box, Tooltip, Typography } from '@mui/material';
import { useTranslation } from 'react-i18next';
import { SegmentVisualizationData } from '../utils/downloadCalculations';

interface SegmentGridItemProps {
  data: SegmentVisualizationData;
}

/**
 * Segment grid item - displays a single colored square with tooltip
 * Colors: Grey (Pending), Blue (Downloading), Green (Completed), Red (Failed)
 */
const SegmentGridItem: React.FC<SegmentGridItemProps> = ({ data }) => {
  const { t } = useTranslation();

  // Tooltip content component
  const TooltipContent = () => (
    <Box sx={{ padding: 0.5 }}>
      <Typography variant="caption" fontWeight="bold" display="block" gutterBottom>
        {t('downloads.details.segmentId', { id: data.segmentId })}
      </Typography>
      <Typography variant="caption" display="block">
        {t('downloads.details.sizeRange')}: {data.tooltipData.sizeRange}
      </Typography>
      <Typography variant="caption" display="block">
        {t('downloads.details.downloaded')}: {data.tooltipData.downloaded}
      </Typography>
      <Typography variant="caption" display="block">
        {t('downloads.details.segmentProgress')}: {data.tooltipData.progress}
      </Typography>
      <Typography variant="caption" display="block">
        {t('downloads.status.title')}: {data.tooltipData.status}
      </Typography>
      <Typography variant="caption" display="block">
        {t('downloads.details.retryCount')}: {data.tooltipData.retryCount}
      </Typography>
      {data.tooltipData.proxy && (
        <Typography variant="caption" display="block">
          {t('downloads.details.proxy')}: {data.tooltipData.proxy}
        </Typography>
      )}
    </Box>
  );

  return (
    <Tooltip title={<TooltipContent />} arrow placement="top" enterDelay={200}>
      <Box
        sx={{
          width: 12,
          height: 12,
          backgroundColor: data.color,
          borderRadius: 0.5,
          cursor: 'pointer',
          transition: 'transform 0.15s ease-in-out',
          '&:hover': {
            transform: 'scale(1.3)',
            zIndex: 1,
          },
        }}
      />
    </Tooltip>
  );
};

export default React.memo(SegmentGridItem);
