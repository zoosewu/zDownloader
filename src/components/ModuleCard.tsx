// Module Card Component - Display module information
import {
  Card,
  CardContent,
  CardHeader,
  Typography,
  Chip,
  Box,
  IconButton,
  Collapse,
  Stack,
  Divider,
  Tooltip,
} from '@mui/material';
import { useState } from 'react';
import { useTranslation } from 'react-i18next';
import ExpandMoreIcon from '@mui/icons-material/ExpandMore';
import RefreshIcon from '@mui/icons-material/Refresh';
import ErrorIcon from '@mui/icons-material/Error';
import CheckCircleIcon from '@mui/icons-material/CheckCircle';
import WarningIcon from '@mui/icons-material/Warning';
import type { ModuleMetadata } from '../types/module';

interface ModuleCardProps {
  module: ModuleMetadata;
  onReload?: (moduleId: string) => void;
  onManageCredentials?: (moduleId: string) => void;
}

export default function ModuleCard({ module, onReload, onManageCredentials }: ModuleCardProps) {
  const { t } = useTranslation();
  const [expanded, setExpanded] = useState(false);

  function getStatusColor(status: string): 'success' | 'error' | 'warning' | 'default' {
    switch (status) {
      case 'active':
        return 'success';
      case 'failed':
      case 'suspended':
        return 'error';
      case 'loading':
        return 'warning';
      default:
        return 'default';
    }
  }

  function getStatusIcon(status: string) {
    switch (status) {
      case 'active':
        return <CheckCircleIcon fontSize="small" />;
      case 'failed':
      case 'suspended':
        return <ErrorIcon fontSize="small" />;
      case 'loading':
        return <WarningIcon fontSize="small" />;
      default:
        return null;
    }
  }

  return (
    <Card>
      <CardHeader
        title={
          <Box display="flex" alignItems="center" gap={1}>
            <Typography variant="h6">{module.name}</Typography>
            <Chip
              icon={getStatusIcon(module.status)}
              label={t(`modules.management.status.${module.status}`)}
              color={getStatusColor(module.status)}
              size="small"
            />
          </Box>
        }
        subheader={
          <Stack direction="row" spacing={1} alignItems="center">
            <Typography variant="body2" color="text.secondary">
              v{module.version}
            </Typography>
            <Typography variant="body2" color="text.secondary">
              • {module.author}
            </Typography>
          </Stack>
        }
        action={
          <Box>
            {onReload && (
              <Tooltip title={t('modules.management.reloadModule')}>
                <IconButton onClick={() => onReload(module.id)} size="small">
                  <RefreshIcon />
                </IconButton>
              </Tooltip>
            )}
            <IconButton
              onClick={() => setExpanded(!expanded)}
              size="small"
              sx={{
                transform: expanded ? 'rotate(180deg)' : 'rotate(0deg)',
                transition: 'transform 0.3s',
              }}
            >
              <ExpandMoreIcon />
            </IconButton>
          </Box>
        }
      />

      <CardContent>
        <Typography variant="body2" color="text.secondary" paragraph>
          {module.description}
        </Typography>

        {module.errorMessage && (
          <Box sx={{ mt: 1, p: 1, bgcolor: 'error.light', borderRadius: 1 }}>
            <Typography variant="body2" color="error.contrastText">
              {module.errorMessage}
            </Typography>
          </Box>
        )}

        <Collapse in={expanded}>
          <Divider sx={{ my: 2 }} />

          {/* URL Patterns */}
          <Box mb={2}>
            <Typography variant="subtitle2" gutterBottom>
              {t('modules.management.urlPatterns')}:
            </Typography>
            <Stack direction="row" spacing={0.5} flexWrap="wrap" gap={0.5}>
              {module.urlPatterns.map((pattern, idx) => (
                <Chip key={idx} label={pattern} size="small" variant="outlined" />
              ))}
            </Stack>
          </Box>

          {/* Permissions */}
          <Box mb={2}>
            <Typography variant="subtitle2" gutterBottom>
              {t('modules.management.permissions')}:
            </Typography>

            {/* Network Permissions */}
            <Box ml={2} mb={1}>
              <Typography variant="body2" fontWeight="medium">
                {t('modules.management.allowedDomains')}:
              </Typography>
              <Stack direction="row" spacing={0.5} flexWrap="wrap" gap={0.5} mt={0.5}>
                {module.permissions.network.allowDomains.map((domain, idx) => (
                  <Chip key={idx} label={domain} size="small" color="primary" variant="outlined" />
                ))}
              </Stack>
            </Box>

            {/* Credential Permissions */}
            {module.permissions.credentials && (
              <Box ml={2}>
                <Typography variant="body2" fontWeight="medium">
                  {t('modules.management.requiredCredentials')}:
                </Typography>
                <Stack direction="row" spacing={0.5} flexWrap="wrap" gap={0.5} mt={0.5}>
                  {module.permissions.credentials.credentialKeys.map((key, idx) => (
                    <Chip key={idx} label={key} size="small" color="secondary" variant="outlined" />
                  ))}
                </Stack>
              </Box>
            )}
          </Box>

          {/* Metadata */}
          <Box>
            <Typography variant="caption" color="text.secondary">
              {t('modules.management.loadedAt')}: {new Date(module.loadTime).toLocaleString()}
            </Typography>
            {module.homepage && (
              <>
                {' • '}
                <Typography
                  variant="caption"
                  component="a"
                  href={module.homepage}
                  target="_blank"
                  rel="noopener noreferrer"
                  sx={{ color: 'primary.main', textDecoration: 'none' }}
                >
                  {t('modules.management.homepage')}
                </Typography>
              </>
            )}
          </Box>
        </Collapse>
      </CardContent>
    </Card>
  );
}
