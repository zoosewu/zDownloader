// Credential Manager Component - Manage module authentication credentials
import {
  Box,
  Button,
  Card,
  CardContent,
  CardHeader,
  TextField,
  Typography,
  Alert,
  Stack,
  Chip,
  Dialog,
  DialogTitle,
  DialogContent,
  DialogActions,
  CircularProgress,
  IconButton,
  Tooltip,
} from '@mui/material';
import { useState, useEffect } from 'react';
import { useTranslation } from 'react-i18next';
import DeleteIcon from '@mui/icons-material/Delete';
import SaveIcon from '@mui/icons-material/Save';
import CheckCircleIcon from '@mui/icons-material/CheckCircle';
import ErrorIcon from '@mui/icons-material/Error';
import {
  saveModuleCredentials,
  getModuleCredentialsStatus,
  deleteModuleCredentials,
} from '../api/modules';
import type { CredentialStatus } from '../types/module';

interface CredentialManagerProps {
  moduleId: string;
  moduleName: string;
  requiredKeys: string[];
  onCredentialsChanged?: () => void;
}

export default function CredentialManager({
  moduleId,
  moduleName,
  requiredKeys,
  onCredentialsChanged,
}: CredentialManagerProps) {
  const { t } = useTranslation();
  const [credentials, setCredentials] = useState<Record<string, string>>({});
  const [status, setStatus] = useState<CredentialStatus | null>(null);
  const [loading, setLoading] = useState(true);
  const [saving, setSaving] = useState(false);
  const [deleteDialogOpen, setDeleteDialogOpen] = useState(false);
  const [message, setMessage] = useState<{ type: 'success' | 'error'; text: string } | null>(null);

  useEffect(() => {
    loadCredentialStatus();
  }, [moduleId]);

  async function loadCredentialStatus() {
    try {
      setLoading(true);
      const credStatus = await getModuleCredentialsStatus(moduleId, requiredKeys);
      setStatus(credStatus);

      // Initialize empty credentials for missing keys
      const initialCreds: Record<string, string> = {};
      requiredKeys.forEach(key => {
        initialCreds[key] = '';
      });
      setCredentials(initialCreds);
    } catch (error) {
      console.error('Failed to load credential status:', error);
      setMessage({
        type: 'error',
        text: t('modules.credentials.errors.loadFailed', { error: String(error) }),
      });
    } finally {
      setLoading(false);
    }
  }

  async function handleSave() {
    // Validate all fields are filled
    const emptyFields = Object.entries(credentials)
      .filter(([_, value]) => !value.trim())
      .map(([key]) => key);

    if (emptyFields.length > 0) {
      setMessage({
        type: 'error',
        text: t('modules.credentials.errors.emptyField', { field: emptyFields.join(', ') }),
      });
      return;
    }

    try {
      setSaving(true);
      await saveModuleCredentials(moduleId, credentials);
      setMessage({
        type: 'success',
        text: t('modules.credentials.saveSuccess'),
      });

      // Reload status
      await loadCredentialStatus();

      // Notify parent component
      onCredentialsChanged?.();
    } catch (error) {
      console.error('Failed to save credentials:', error);
      setMessage({
        type: 'error',
        text: t('modules.credentials.errors.saveFailed', { error: String(error) }),
      });
    } finally {
      setSaving(false);
    }
  }

  async function handleDelete() {
    try {
      await deleteModuleCredentials(moduleId);
      setMessage({
        type: 'success',
        text: t('modules.credentials.deleteSuccess'),
      });
      setDeleteDialogOpen(false);

      // Clear form
      const clearedCreds: Record<string, string> = {};
      requiredKeys.forEach(key => {
        clearedCreds[key] = '';
      });
      setCredentials(clearedCreds);

      // Reload status
      await loadCredentialStatus();

      // Notify parent component
      onCredentialsChanged?.();
    } catch (error) {
      console.error('Failed to delete credentials:', error);
      setMessage({
        type: 'error',
        text: t('modules.credentials.errors.deleteFailed', { error: String(error) }),
      });
    }
  }

  function handleCredentialChange(key: string, value: string) {
    setCredentials(prev => ({
      ...prev,
      [key]: value,
    }));
  }

  if (loading) {
    return (
      <Box display="flex" justifyContent="center" p={3}>
        <CircularProgress />
      </Box>
    );
  }

  const isConfigured = status?.configured ?? false;
  const hasAllKeys = status?.missingKeys.length === 0;

  return (
    <Card>
      <CardHeader
        title={
          <Box display="flex" alignItems="center" gap={1}>
            <Typography variant="h6">
              {t('modules.credentials.title')} - {moduleName}
            </Typography>
            {isConfigured && hasAllKeys && (
              <Chip
                icon={<CheckCircleIcon />}
                label={t('modules.credentials.configured')}
                color="success"
                size="small"
              />
            )}
            {isConfigured && !hasAllKeys && (
              <Chip
                icon={<ErrorIcon />}
                label={t('modules.credentials.missingKeys')}
                color="warning"
                size="small"
              />
            )}
            {!isConfigured && (
              <Chip
                label={t('modules.credentials.notConfigured')}
                color="default"
                size="small"
              />
            )}
          </Box>
        }
        action={
          isConfigured && (
            <Tooltip title={t('modules.credentials.deleteCredentials')}>
              <IconButton onClick={() => setDeleteDialogOpen(true)} color="error">
                <DeleteIcon />
              </IconButton>
            </Tooltip>
          )
        }
      />
      <CardContent>
        <Stack spacing={2}>
          {message && (
            <Alert severity={message.type} onClose={() => setMessage(null)}>
              {message.text}
            </Alert>
          )}

          <Typography variant="body2" color="text.secondary">
            {t('modules.credentials.description')}
          </Typography>

          {/* Credential Status */}
          {status && status.missingKeys.length > 0 && (
            <Alert severity="warning">
              {t('modules.credentials.missingKeys')}: {status.missingKeys.join(', ')}
            </Alert>
          )}

          {/* Credential Input Fields */}
          <Stack spacing={2}>
            {requiredKeys.map(key => (
              <TextField
                key={key}
                label={key}
                type={key.toLowerCase().includes('password') ? 'password' : 'text'}
                value={credentials[key] || ''}
                onChange={(e) => handleCredentialChange(key, e.target.value)}
                fullWidth
                required
                placeholder={`Enter ${key}`}
                autoComplete="off"
              />
            ))}
          </Stack>

          {/* Actions */}
          <Box display="flex" gap={2} justifyContent="flex-end">
            <Button
              variant="contained"
              color="primary"
              startIcon={saving ? <CircularProgress size={20} /> : <SaveIcon />}
              onClick={handleSave}
              disabled={saving}
            >
              {saving ? t('common.saving') : t('modules.credentials.saveCredentials')}
            </Button>
          </Box>
        </Stack>
      </CardContent>

      {/* Delete Confirmation Dialog */}
      <Dialog open={deleteDialogOpen} onClose={() => setDeleteDialogOpen(false)}>
        <DialogTitle>{t('common.confirm')}</DialogTitle>
        <DialogContent>
          <Typography>
            {t('modules.credentials.confirmDelete', { moduleName })}
          </Typography>
        </DialogContent>
        <DialogActions>
          <Button onClick={() => setDeleteDialogOpen(false)}>
            {t('common.cancel')}
          </Button>
          <Button onClick={handleDelete} color="error" variant="contained">
            {t('common.delete')}
          </Button>
        </DialogActions>
      </Dialog>
    </Card>
  );
}
