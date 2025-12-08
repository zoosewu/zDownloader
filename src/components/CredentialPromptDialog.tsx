// Credential Prompt Dialog - Prompt user for missing credentials (T118)
import React, { useState, useEffect } from 'react';
import {
  Dialog,
  DialogTitle,
  DialogContent,
  DialogActions,
  Button,
  TextField,
  Alert,
  Stack,
  Typography,
  Box,
  CircularProgress,
} from '@mui/material';
import { useTranslation } from 'react-i18next';
import LockIcon from '@mui/icons-material/Lock';
import { saveModuleCredentials } from '../api/modules';

export interface CredentialPromptProps {
  open: boolean;
  moduleId: string;
  moduleName: string;
  requiredKeys: string[];
  onClose: () => void;
  onSuccess: () => void;
}

export default function CredentialPromptDialog({
  open,
  moduleId,
  moduleName,
  requiredKeys,
  onClose,
  onSuccess,
}: CredentialPromptProps) {
  const { t } = useTranslation();
  const [credentials, setCredentials] = useState<Record<string, string>>({});
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  // Reset form when dialog opens
  useEffect(() => {
    if (open) {
      // Initialize with empty values for all required keys
      const initial: Record<string, string> = {};
      requiredKeys.forEach((key) => {
        initial[key] = '';
      });
      setCredentials(initial);
      setError(null);
    }
  }, [open, requiredKeys]);

  const handleInputChange = (key: string, value: string) => {
    setCredentials((prev) => ({
      ...prev,
      [key]: value,
    }));
    // Clear error when user starts typing
    if (error) {
      setError(null);
    }
  };

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();

    // Validate all required fields are filled
    const emptyFields = requiredKeys.filter((key) => !credentials[key]?.trim());
    if (emptyFields.length > 0) {
      setError(
        t('modules.credentials.errors.emptyField', {
          field: emptyFields.join(', '),
        })
      );
      return;
    }

    setLoading(true);
    setError(null);

    try {
      await saveModuleCredentials(moduleId, credentials);
      console.log('[CredentialPromptDialog] Credentials saved successfully');
      onSuccess();
      onClose();
    } catch (err: any) {
      console.error('[CredentialPromptDialog] Failed to save credentials:', err);
      setError(
        t('modules.credentials.errors.saveFailed', {
          error: err.message || String(err),
        })
      );
    } finally {
      setLoading(false);
    }
  };

  const handleCancel = () => {
    if (!loading) {
      onClose();
    }
  };

  // Get user-friendly label for credential keys
  const getFieldLabel = (key: string): string => {
    const labels: Record<string, string> = {
      username: t('modules.credentials.username', { defaultValue: '用戶名' }),
      password: t('modules.credentials.password', { defaultValue: '密碼' }),
      apiKey: t('modules.credentials.apiKey', { defaultValue: 'API 金鑰' }),
      token: t('modules.credentials.token', { defaultValue: '令牌' }),
      email: t('modules.credentials.email', { defaultValue: '電子郵件' }),
    };
    return labels[key] || key;
  };

  // Determine if field should be password type
  const isPasswordField = (key: string): boolean => {
    return ['password', 'apiKey', 'token'].includes(key);
  };

  return (
    <Dialog
      open={open}
      onClose={handleCancel}
      maxWidth="sm"
      fullWidth
      disableEscapeKeyDown={loading}
    >
      <DialogTitle>
        <Box display="flex" alignItems="center" gap={1}>
          <LockIcon color="primary" />
          <Typography variant="h6">
            {t('modules.credentials.promptTitle', {
              defaultValue: '需要認證',
            })}
          </Typography>
        </Box>
      </DialogTitle>

      <DialogContent>
        <Stack spacing={2} sx={{ mt: 1 }}>
          <Alert severity="info">
            {t('modules.credentials.promptDescription', {
              moduleName,
              defaultValue: `模組「${moduleName}」需要認證才能下載。請輸入您的登入資訊。`,
            })}
          </Alert>

          {error && (
            <Alert severity="error" onClose={() => setError(null)}>
              {error}
            </Alert>
          )}

          <form onSubmit={handleSubmit}>
            <Stack spacing={2}>
              {requiredKeys.map((key) => (
                <TextField
                  key={key}
                  label={getFieldLabel(key)}
                  type={isPasswordField(key) ? 'password' : 'text'}
                  value={credentials[key] || ''}
                  onChange={(e) => handleInputChange(key, e.target.value)}
                  required
                  fullWidth
                  disabled={loading}
                  autoComplete="off"
                  placeholder={t('modules.credentials.enterCredential', {
                    field: getFieldLabel(key),
                    defaultValue: `請輸入${getFieldLabel(key)}`,
                  })}
                />
              ))}
            </Stack>
          </form>

          <Typography variant="caption" color="text.secondary">
            {t('modules.credentials.securityNotice', {
              defaultValue: '您的認證資訊將安全存儲在系統密鑰鏈中。',
            })}
          </Typography>
        </Stack>
      </DialogContent>

      <DialogActions>
        <Button onClick={handleCancel} disabled={loading}>
          {t('common.cancel', { defaultValue: '取消' })}
        </Button>
        <Button
          onClick={handleSubmit}
          variant="contained"
          disabled={loading}
          startIcon={loading && <CircularProgress size={16} />}
        >
          {loading
            ? t('common.saving', { defaultValue: '儲存中...' })
            : t('common.save', { defaultValue: '儲存' })}
        </Button>
      </DialogActions>
    </Dialog>
  );
}
