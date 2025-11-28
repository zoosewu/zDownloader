// Progress bar component - Material UI LinearProgress wrapper
import React from 'react';
import { LinearProgress, Box, Typography } from '@mui/material';

interface ProgressBarProps {
  progress: number; // 0-100
  showLabel?: boolean;
  height?: number;
}

const ProgressBar: React.FC<ProgressBarProps> = ({
  progress,
  showLabel = false,
  height = 8,
}) => {
  const clampedProgress = Math.min(100, Math.max(0, progress));

  return (
    <Box sx={{ display: 'flex', alignItems: 'center', width: '100%' }}>
      <Box sx={{ width: '100%', marginRight: showLabel ? 1 : 0 }}>
        <LinearProgress
          variant="determinate"
          value={clampedProgress}
          sx={{
            height,
            borderRadius: height / 2,
            backgroundColor: 'rgba(0, 0, 0, 0.1)',
            '& .MuiLinearProgress-bar': {
              borderRadius: height / 2,
            },
          }}
        />
      </Box>
      {showLabel && (
        <Box sx={{ minWidth: 50 }}>
          <Typography variant="body2" color="text.secondary">
            {`${clampedProgress.toFixed(1)}%`}
          </Typography>
        </Box>
      )}
    </Box>
  );
};

export default ProgressBar;
