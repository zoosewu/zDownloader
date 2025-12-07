// Batch import button component - opens file picker dialog
import React from 'react';
import { Button, CircularProgress } from '@mui/material';
import { Upload as UploadIcon } from '@mui/icons-material';
import { useTranslation } from 'react-i18next';
import { open } from '@tauri-apps/plugin-dialog';
import { invoke } from '../utils/tauri';

interface BatchImportButtonProps {
  onImportSuccess?: (taskIds: string[]) => void;
  onImportError?: (error: string) => void;
}

const BatchImportButton: React.FC<BatchImportButtonProps> = ({
  onImportSuccess,
  onImportError,
}) => {
  const { t } = useTranslation();
  const [loading, setLoading] = React.useState(false);

  const handleImport = async () => {
    try {
      setLoading(true);

      // Open file picker dialog for text files
      const selected = await open({
        multiple: false,
        filters: [
          {
            name: 'Text Files',
            extensions: ['txt'],
          },
        ],
      });

      if (!selected) {
        setLoading(false);
        return;
      }

      // Call Tauri command to import batch downloads
      const taskIds = await invoke<string[]>('import_batch_downloads', {
        filePath: selected,
      });

      onImportSuccess?.(taskIds);
    } catch (error) {
      console.error('Failed to import batch downloads:', error);

      // Extract error message from various error formats
      let errorMessage = 'Unknown error';

      if (error instanceof Error) {
        errorMessage = error.message;
      } else if (typeof error === 'string') {
        errorMessage = error;
      } else if (error && typeof error === 'object') {
        // Handle Tauri error objects
        if ('message' in error && typeof error.message === 'string') {
          errorMessage = error.message;
        } else {
          // Try to stringify the error object
          try {
            errorMessage = JSON.stringify(error);
          } catch {
            errorMessage = String(error);
          }
        }
      } else if (error !== null && error !== undefined) {
        errorMessage = String(error);
      }

      console.error('Extracted error message:', errorMessage);
      onImportError?.(errorMessage);
    } finally {
      setLoading(false);
    }
  };

  return (
    <Button
      variant="outlined"
      startIcon={loading ? <CircularProgress size={20} /> : <UploadIcon />}
      onClick={handleImport}
      disabled={loading}
    >
      {t('downloads.batch.import')}
    </Button>
  );
};

export default BatchImportButton;
