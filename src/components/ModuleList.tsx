// Module List Component - Display list of all modules
import {
  Box,
  Button,
  Stack,
  Typography,
  Alert,
  CircularProgress,
  Paper,
} from '@mui/material';
import { useState, useEffect } from 'react';
import { useTranslation } from 'react-i18next';
import RefreshIcon from '@mui/icons-material/Refresh';
import {
  getAllModules,
  reloadModule,
  scanModules,
} from '../api/modules';
import type { ModuleMetadata } from '../types/module';
import ModuleCard from './ModuleCard';

interface ModuleListProps {
  onModuleSelect?: (moduleId: string) => void;
}

export default function ModuleList({ onModuleSelect }: ModuleListProps) {
  const { t } = useTranslation();
  const [modules, setModules] = useState<ModuleMetadata[]>([]);
  const [loading, setLoading] = useState(true);
  const [scanning, setScanning] = useState(false);
  const [message, setMessage] = useState<{ type: 'success' | 'error' | 'info'; text: string } | null>(null);

  useEffect(() => {
    loadModules();
  }, []);

  async function loadModules() {
    try {
      setLoading(true);
      const loadedModules = await getAllModules();
      setModules(loadedModules);

      if (loadedModules.length === 0) {
        setMessage({
          type: 'info',
          text: t('modules.management.noModules'),
        });
      }
    } catch (error) {
      console.error('Failed to load modules:', error);
      setMessage({
        type: 'error',
        text: t('modules.management.errors.loadFailed', { error: String(error) }),
      });
    } finally {
      setLoading(false);
    }
  }

  async function handleScanModules() {
    try {
      setScanning(true);
      const result = await scanModules();

      setMessage({
        type: 'success',
        text: t('modules.management.scanComplete', {
          found: result.totalFound,
          loaded: result.successfullyLoaded,
        }),
      });

      // Reload modules after scan
      await loadModules();
    } catch (error) {
      console.error('Failed to scan modules:', error);
      setMessage({
        type: 'error',
        text: t('modules.management.errors.scanFailed', { error: String(error) }),
      });
    } finally {
      setScanning(false);
    }
  }

  async function handleReloadModule(moduleId: string) {
    try {
      await reloadModule(moduleId);
      setMessage({
        type: 'success',
        text: t('modules.management.reloadSuccess', { moduleId }),
      });

      // Reload all modules to get updated status
      await loadModules();
    } catch (error) {
      console.error('Failed to reload module:', error);
      setMessage({
        type: 'error',
        text: t('modules.management.errors.reloadFailed', { error: String(error) }),
      });
    }
  }

  if (loading) {
    return (
      <Box display="flex" justifyContent="center" p={4}>
        <CircularProgress />
      </Box>
    );
  }

  return (
    <Box>
      {/* Header */}
      <Box display="flex" justifyContent="space-between" alignItems="center" mb={3}>
        <Typography variant="h5">{t('modules.management.title')}</Typography>
        <Stack direction="row" spacing={2}>
          <Button
            variant="outlined"
            startIcon={scanning ? <CircularProgress size={20} /> : <RefreshIcon />}
            onClick={handleScanModules}
            disabled={scanning}
          >
            {t('modules.management.scanModules')}
          </Button>
        </Stack>
      </Box>

      {/* Messages */}
      {message && (
        <Alert severity={message.type} onClose={() => setMessage(null)} sx={{ mb: 2 }}>
          {message.text}
        </Alert>
      )}

      {/* Module List */}
      {modules.length === 0 ? (
        <Paper sx={{ p: 4, textAlign: 'center' }}>
          <Typography variant="body1" color="text.secondary">
            {t('modules.management.noModules')}
          </Typography>
          <Typography variant="body2" color="text.secondary" sx={{ mt: 1 }}>
            {t('modules.management.scanToFind')}
          </Typography>
        </Paper>
      ) : (
        <Stack spacing={2}>
          {modules.map(module => (
            <ModuleCard
              key={module.id}
              module={module}
              onReload={handleReloadModule}
              onManageCredentials={onModuleSelect}
            />
          ))}
        </Stack>
      )}
    </Box>
  );
}
