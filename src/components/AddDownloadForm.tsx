// Add Download Form - Form for adding new downloads with validation
import React, { useState } from 'react';
import {
  Box,
  TextField,
  Button,
  Alert,
  InputAdornment,
} from '@mui/material';
import { useAtomValue } from 'jotai';
import { invoke } from '@tauri-apps/api/core';
import { useNavigation } from '../hooks/useNavigation';
import { useDownloadPath } from '../hooks/useDownloadPath';
import { settingsAtom } from '../stores/settingsStore';

interface UrlDetectionResult {
  url: string;
  isValid: boolean;
  detectorName?: string;
  detectionMode?: string;
  suggestedFilename?: string;
  fileExtension?: string;
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
  const { navigateTo } = useNavigation();

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

    if (!url.trim()) {
      setError('請輸入下載網址');
      return;
    }

    if (!isValidUrl(url.trim())) {
      setError('無效的網址格式');
      return;
    }

    setError(null);
    setLoading(true);

    try {
      // Validate URL with backend detector
      const detectionResult = await invoke<UrlDetectionResult>('detect_url', { url: url.trim() });
      if (!detectionResult.isValid) {
        setError('此網址不是有效的下載連結');
        setLoading(false);
        return;
      }

      const taskId = await invoke<string>('create_download_with_path', {
        url: url.trim(),
        filename: filename.trim() || undefined,
        downloadPath,
      });

      await invoke('start_download', { taskId });
      navigateTo('main');
    } catch (e: any) {
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

      <TextField
        label="檔案名稱（選填）"
        value={filename}
        onChange={(e) => setFilename(e.target.value)}
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
