// URL Detection Indicator - Shows which module handles a URL (T146)
import { Box, Chip, Tooltip, Typography } from '@mui/material';
import CheckCircleIcon from '@mui/icons-material/CheckCircle';
import ErrorIcon from '@mui/icons-material/Error';
import InfoIcon from '@mui/icons-material/Info';
import { useTranslation } from 'react-i18next';

export interface UrlDetectionStatus {
  isDetecting: boolean;
  isSupported: boolean;
  moduleId?: string;
  moduleName?: string;
  requiresAuth: boolean;
  error?: string;
}

interface UrlDetectionIndicatorProps {
  status: UrlDetectionStatus | null;
}

export default function UrlDetectionIndicator({ status }: UrlDetectionIndicatorProps) {
  const { t } = useTranslation();

  if (!status) {
    return null;
  }

  // Show loading state
  if (status.isDetecting) {
    return (
      <Box sx={{ mt: 1 }}>
        <Chip
          size="small"
          icon={<InfoIcon />}
          label={t('urlDetection.detecting', { defaultValue: '檢測中...' })}
          color="default"
        />
      </Box>
    );
  }

  // Show error state
  if (status.error) {
    return (
      <Box sx={{ mt: 1 }}>
        <Tooltip title={status.error}>
          <Chip
            size="small"
            icon={<ErrorIcon />}
            label={t('urlDetection.failed', { defaultValue: '檢測失敗' })}
            color="error"
            variant="outlined"
          />
        </Tooltip>
      </Box>
    );
  }

  // Show module detection result (only when module is detected)
  if (status.isSupported && status.moduleId) {
    return (
      <Box sx={{ mt: 1, display: 'flex', gap: 1, alignItems: 'center', flexWrap: 'wrap' }}>
        <Chip
          size="small"
          icon={<CheckCircleIcon />}
          label={
            <Box component="span">
              <Typography component="span" variant="caption" fontWeight="medium">
                {t('urlDetection.supportedBy', { defaultValue: '支援模組' })}:{' '}
              </Typography>
              <Typography component="span" variant="caption">
                {status.moduleName || status.moduleId}
              </Typography>
            </Box>
          }
          color="success"
          variant="outlined"
        />
        {status.requiresAuth && (
          <Chip
            size="small"
            label={t('urlDetection.requiresAuth', { defaultValue: '需要認證' })}
            color="warning"
            variant="outlined"
          />
        )}
      </Box>
    );
  }

  // Don't show anything for unsupported URLs or basic file extension detection
  return null;
}
