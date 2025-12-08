// Add Download Form - Form for adding new downloads with validation
import React, { useState, useEffect } from 'react';
import {
  Box,
  TextField,
  Button,
  Alert,
  InputAdornment,
} from '@mui/material';
import { useAtomValue } from 'jotai';
import { invoke } from '../utils/tauri';
import { useNavigation } from '../hooks/useNavigation';
import { useDownloadPath } from '../hooks/useDownloadPath';
import { settingsAtom } from '../stores/settingsStore';
import { isTauriAvailable } from '../utils/tauri';
import UrlDetectionIndicator, { UrlDetectionStatus } from './UrlDetectionIndicator';

interface UrlDetectionResult {
  url: string;
  isValid: boolean;
  detectorName?: string;
  detectionMode?: string;
  suggestedFilename?: string;
  fileExtension?: string;
}

interface UrlModuleDetectionResult {
  url: string;
  moduleId?: string;
  moduleName?: string;
  isSupported: boolean;
  requiresAuth: boolean;
}

interface AddDownloadFormProps {
  initialUrl?: string;
  onCancel: () => void;
}

const AddDownloadForm: React.FC<AddDownloadFormProps> = ({
  initialUrl = '',
  onCancel,
}) => {
  const settings = useAtomValue(settingsAtom);
  const [url, setUrl] = useState(initialUrl);
  const [filename, setFilename] = useState('');
  const { path: downloadPath, openDirectoryPicker, errorMessage: pathError } = useDownloadPath(settings.defaultDownloadDir);
  const [error, setError] = useState<string | null>(null);
  const [loading, setLoading] = useState(false);
  const [isUserEditedFilename, setIsUserEditedFilename] = useState(false); // Track if user manually edited filename
  const [detectionStatus, setDetectionStatus] = useState<UrlDetectionStatus | null>(null); // T146: URL detection indicator
  const { navigateTo } = useNavigation();

  // Auto-paste from clipboard when form opens (if no initialUrl)
  useEffect(() => {
    const checkClipboard = async () => {
      // Skip if we already have an initial URL
      if (initialUrl) {
        console.log('[AddDownloadForm] Initial URL provided, skipping clipboard check');
        return;
      }

      // Check if Tauri is available
      if (!isTauriAvailable()) {
        console.log('[AddDownloadForm] Tauri not available, skipping clipboard check');
        return;
      }

      try {
        console.log('[AddDownloadForm] Checking clipboard for download URL...');
        const content = await invoke<string>('get_clipboard_content');

        if (!content) {
          console.log('[AddDownloadForm] Clipboard is empty');
          return;
        }

        console.log('[AddDownloadForm] Clipboard content:', content.substring(0, 100));

        // Validate if it's a download URL
        const result = await invoke<UrlDetectionResult>('detect_url', { url: content });
        console.log('[AddDownloadForm] Detection result:', result);

        if (result.isValid) {
          console.log('[AddDownloadForm] Valid download URL found in clipboard, auto-filling');
          setUrl(content);
          if (result.suggestedFilename) {
            setFilename(result.suggestedFilename);
          }
        } else {
          console.log('[AddDownloadForm] Clipboard content is not a valid download URL');
        }
      } catch (e) {
        // Silently fail - clipboard auto-paste is optional
        console.log('[AddDownloadForm] Failed to check clipboard:', e);
      }
    };

    checkClipboard();
  }, []); // Only run once on mount

  // Auto-parse filename when URL changes
  useEffect(() => {
    // Skip if URL is empty or if user has manually edited the filename
    if (!url || isUserEditedFilename) {
      return;
    }

    // Skip if Tauri is not available
    if (!isTauriAvailable()) {
      return;
    }

    // Debounce the URL detection
    const timeoutId = setTimeout(async () => {
      try {
        console.log('[AddDownloadForm] URL changed, detecting filename...');
        const result = await invoke<UrlDetectionResult>('detect_url', { url: url.trim() });
        console.log('[AddDownloadForm] Detection result:', result);

        if (result.isValid && result.suggestedFilename) {
          console.log('[AddDownloadForm] Auto-filling filename:', result.suggestedFilename);
          setFilename(result.suggestedFilename);
        }
      } catch (e) {
        // Silently fail - auto-parsing is optional
        console.log('[AddDownloadForm] Failed to auto-parse filename:', e);
      }
    }, 500); // 500ms debounce

    return () => clearTimeout(timeoutId);
  }, [url, isUserEditedFilename]);

  // T145: Detect module support when URL changes
  useEffect(() => {
    // Reset detection status if URL is empty
    if (!url.trim()) {
      setDetectionStatus(null);
      return;
    }

    // Skip if Tauri is not available
    if (!isTauriAvailable()) {
      return;
    }

    // Set detecting state
    setDetectionStatus({
      isDetecting: true,
      isSupported: false,
      requiresAuth: false,
    });

    // Debounce the module detection
    const timeoutId = setTimeout(async () => {
      try {
        console.log('[AddDownloadForm] Checking module support...');
        const result = await invoke<UrlModuleDetectionResult>('detect_url_module', {
          url: url.trim(),
        });
        console.log('[AddDownloadForm] Module detection result:', result);

        setDetectionStatus({
          isDetecting: false,
          isSupported: result.isSupported,
          moduleId: result.moduleId,
          moduleName: result.moduleName,
          requiresAuth: result.requiresAuth,
        });
      } catch (e) {
        console.log('[AddDownloadForm] Module detection failed:', e);
        setDetectionStatus({
          isDetecting: false,
          isSupported: false,
          requiresAuth: false,
          error: String(e),
        });
      }
    }, 600); // 600ms debounce (slightly longer than filename detection)

    return () => clearTimeout(timeoutId);
  }, [url]);

  const isValidUrl = (urlString: string): boolean => {
    try {
      const urlObj = new URL(urlString);
      return urlObj.protocol === 'http:' || urlObj.protocol === 'https:';
    } catch {
      return false;
    }
  };

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();

    console.log('[AddDownloadForm] ========== Form Submission Started ==========');
    console.log('[AddDownloadForm] URL:', url);
    console.log('[AddDownloadForm] Filename:', filename);
    console.log('[AddDownloadForm] Download Path:', downloadPath);

    if (!url.trim()) {
      console.log('[AddDownloadForm] ✗ Validation failed: URL is empty');
      setError('請輸入下載網址');
      return;
    }

    if (!isValidUrl(url.trim())) {
      console.log('[AddDownloadForm] ✗ Validation failed: Invalid URL format');
      setError('無效的網址格式');
      return;
    }

    console.log('[AddDownloadForm] ✓ Basic validation passed');

    setError(null);
    setLoading(true);

    try {
      // Check if Tauri API is available
      console.log('[AddDownloadForm] Checking Tauri API availability...');
      const tauriAvailable = isTauriAvailable();
      console.log('[AddDownloadForm] Tauri API available:', tauriAvailable);

      if (!tauriAvailable) {
        console.log('[AddDownloadForm] ✗ Tauri API not available, aborting');
        setError('下載管理器尚未準備就緒，請稍後再試');
        setLoading(false);
        return;
      }

      // Try to validate URL with backend detector (optional)
      // If Tauri API is not ready, skip validation and let backend handle it
      console.log('[AddDownloadForm] Calling detect_url command...');
      try {
        const detectionResult = await invoke<UrlDetectionResult>('detect_url', { url: url.trim() });
        console.log('[AddDownloadForm] detect_url result:', detectionResult);

        if (!detectionResult.isValid) {
          console.log('[AddDownloadForm] ✗ URL is not a valid download link');
          setError('此網址不是有效的下載連結');
          setLoading(false);
          return;
        }
        console.log('[AddDownloadForm] ✓ URL validation passed');
      } catch (detectionError) {
        // Silently skip frontend validation if detect_url fails
        // Backend will validate the URL when creating the download
        console.log('[AddDownloadForm] ⚠ URL detection unavailable, skipping frontend validation:', detectionError);
      }

      console.log('[AddDownloadForm] Calling create_download_with_path command...');
      const taskId = await invoke<string>('create_download_with_path', {
        url: url.trim(),
        filename: filename.trim() || undefined,
        downloadPath,
      });
      console.log('[AddDownloadForm] ✓ Download task created, taskId:', taskId);

      console.log('[AddDownloadForm] Calling start_download command...');
      await invoke('start_download', { taskId });
      console.log('[AddDownloadForm] ✓ Download started successfully');

      console.log('[AddDownloadForm] Navigating to main page...');
      navigateTo('main');
      console.log('[AddDownloadForm] ========== Form Submission Completed ==========');
    } catch (e: any) {
      console.error('[AddDownloadForm] ✗ Error during form submission:', e);
      console.error('[AddDownloadForm] Error message:', e.message);
      console.error('[AddDownloadForm] Error stack:', e.stack);
      setError(e.message || '建立下載任務失敗');
    } finally {
      setLoading(false);
    }
  };

  const handleKeyPress = (event: React.KeyboardEvent) => {
    if (event.key === 'Enter' && !loading) {
      handleSubmit(event as any);
    }
  };

  return (
    <Box component="form" onSubmit={handleSubmit}>
      {error && (
        <Alert severity="error" sx={{ mb: 2 }}>
          {error}
        </Alert>
      )}

      <TextField
        label="下載網址"
        value={url}
        onChange={(e) => setUrl(e.target.value)}
        onKeyPress={handleKeyPress}
        fullWidth
        required
        margin="normal"
        placeholder="https://example.com/file.zip"
        helperText="請輸入要下載的檔案網址"
      />

      {/* T146: URL Detection Indicator */}
      <UrlDetectionIndicator status={detectionStatus} />

      <TextField
        label="檔案名稱（選填）"
        value={filename}
        onChange={(e) => {
          setFilename(e.target.value);
          setIsUserEditedFilename(true); // Mark as user-edited to prevent auto-overwrite
        }}
        onKeyPress={handleKeyPress}
        fullWidth
        margin="normal"
        placeholder="留空則使用網址中的檔名"
      />

      <TextField
        label="下載路徑"
        value={downloadPath}
        fullWidth
        margin="normal"
        helperText={pathError || "檔案將儲存到此路徑"}
        error={!!pathError}
        InputProps={{
          readOnly: true,
          endAdornment: (
            <InputAdornment position="end">
              <Button onClick={openDirectoryPicker}>瀏覽</Button>
            </InputAdornment>
          ),
        }}
      />

      <Box sx={{ display: 'flex', gap: 2, mt: 3 }}>
        <Button
          variant="contained"
          type="submit"
          disabled={loading || !url.trim()}
          sx={{ flex: 1 }}
        >
          {loading ? '建立中...' : '新增下載'}
        </Button>
        <Button
          variant="outlined"
          onClick={onCancel}
          disabled={loading}
          sx={{ flex: 1 }}
        >
          取消
        </Button>
      </Box>
    </Box>
  );
};

export default AddDownloadForm;
