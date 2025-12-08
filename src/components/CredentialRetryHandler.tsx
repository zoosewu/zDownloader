// Credential Retry Handler - Handle authentication errors and retry (T119)
import { useState } from 'react';
import { Alert, Button, Box } from '@mui/material';
import { useTranslation } from 'react-i18next';
import RefreshIcon from '@mui/icons-material/Refresh';
import { invoke } from '../utils/tauri';
import { isCredentialError, extractModuleIdFromUrl } from '../utils/credentialError';
import CredentialPromptDialog from './CredentialPromptDialog';
import { getModuleCredentialsStatus } from '../api/modules';
import type { DownloadTask } from '../types/download';
import { DownloadStatus } from '../types/download';

interface CredentialRetryHandlerProps {
  task: DownloadTask;
  onRetrySuccess?: () => void;
}

/**
 * T119: Component to detect credential errors and provide retry with re-authentication
 *
 * This component:
 * 1. Detects if a download failed due to credential issues
 * 2. Provides a "Retry with Credentials" button
 * 3. Shows credential prompt dialog when clicked
 * 4. Retries the download after credentials are updated
 */
export default function CredentialRetryHandler({
  task,
  onRetrySuccess,
}: CredentialRetryHandlerProps) {
  const { t } = useTranslation();
  const [showCredentialPrompt, setShowCredentialPrompt] = useState(false);
  const [credentialPromptData, setCredentialPromptData] = useState<{
    moduleId: string;
    moduleName: string;
    requiredKeys: string[];
  } | null>(null);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  // Check if this is a credential error
  const isCredError = task.status === DownloadStatus.Failed && isCredentialError(task.errorMessage);

  if (!isCredError) {
    return null; // Don't show anything if not a credential error
  }

  const handleRetryClick = async () => {
    console.log('[CredentialRetryHandler] Retry with credentials clicked');

    // Try to extract module ID from URL
    const moduleId = extractModuleIdFromUrl(task.url);

    if (!moduleId) {
      console.warn('[CredentialRetryHandler] Could not determine module ID from URL');
      setError(t('modules.credentials.errors.unknownModule', {
        defaultValue: '無法識別模組，請手動設定認證',
      }));
      return;
    }

    try {
      setLoading(true);
      setError(null);

      // Get credential status to determine required keys
      const credStatus = await getModuleCredentialsStatus(moduleId, [
        'username',
        'password',
      ]);

      console.log('[CredentialRetryHandler] Credential status:', credStatus);

      // Show credential prompt
      setCredentialPromptData({
        moduleId,
        moduleName: moduleId.toUpperCase(), // TODO: Get actual module name
        requiredKeys: credStatus.credentialKeys,
      });
      setShowCredentialPrompt(true);
    } catch (err: any) {
      console.error('[CredentialRetryHandler] Failed to check credentials:', err);
      setError(
        t('modules.credentials.errors.checkFailed', {
          error: err.message || String(err),
          defaultValue: `檢查認證狀態失敗: ${err.message || String(err)}`,
        })
      );
    } finally {
      setLoading(false);
    }
  };

  const handleCredentialSuccess = async () => {
    console.log('[CredentialRetryHandler] Credentials updated, retrying download...');
    setShowCredentialPrompt(false);
    setLoading(true);
    setError(null);

    try {
      // Retry the download
      console.log('[CredentialRetryHandler] Calling retry_download_direct...');
      await invoke('retry_download_direct', { taskId: task.id });
      console.log('[CredentialRetryHandler] ✓ Download retry started');

      // Notify parent
      if (onRetrySuccess) {
        onRetrySuccess();
      }
    } catch (err: any) {
      console.error('[CredentialRetryHandler] Failed to retry download:', err);
      setError(
        t('downloads.errors.retryFailed', {
          error: err.message || String(err),
          defaultValue: `重試失敗: ${err.message || String(err)}`,
        })
      );
    } finally {
      setLoading(false);
    }
  };

  const handleCredentialCancel = () => {
    console.log('[CredentialRetryHandler] User cancelled credential input');
    setShowCredentialPrompt(false);
    setCredentialPromptData(null);
  };

  return (
    <Box>
      <Alert
        severity="warning"
        action={
          <Button
            color="inherit"
            size="small"
            onClick={handleRetryClick}
            disabled={loading}
            startIcon={<RefreshIcon />}
          >
            {t('modules.credentials.retryWithAuth', {
              defaultValue: '重新認證並重試',
            })}
          </Button>
        }
      >
        {t('modules.credentials.authErrorDetected', {
          defaultValue: '檢測到認證錯誤。請更新您的登入資訊後重試。',
        })}
      </Alert>

      {error && (
        <Alert severity="error" sx={{ mt: 1 }}>
          {error}
        </Alert>
      )}

      {/* Credential Prompt Dialog */}
      {credentialPromptData && (
        <CredentialPromptDialog
          open={showCredentialPrompt}
          moduleId={credentialPromptData.moduleId}
          moduleName={credentialPromptData.moduleName}
          requiredKeys={credentialPromptData.requiredKeys}
          onClose={handleCredentialCancel}
          onSuccess={handleCredentialSuccess}
        />
      )}
    </Box>
  );
}
