// Batch import button component - opens file picker dialog
import React from 'react';
import { Button, CircularProgress } from '@mui/material';
import { Upload as UploadIcon } from '@mui/icons-material';
import { useTranslation } from 'react-i18next';
import { open } from '@tauri-apps/plugin-dialog';
import { useSetAtom } from 'jotai';
import { backend, type BatchResult } from '../backend';
import { upsertTaskAtom } from '../stores/downloadStore';

interface BatchImportButtonProps {
  onImportSuccess?: (result: BatchResult) => void;
  onImportError?: (error: string) => void;
}

const BatchImportButton: React.FC<BatchImportButtonProps> = ({ onImportSuccess, onImportError }) => {
  const { t } = useTranslation();
  const upsert = useSetAtom(upsertTaskAtom);
  const [loading, setLoading] = React.useState(false);

  const handleImport = async () => {
    setLoading(true);
    try {
      const selected = await open({
        multiple: false,
        filters: [{ name: 'Text Files', extensions: ['txt'] }],
      });
      if (!selected) return;

      // Imported URLs queue right away; the concurrency limit decides when each starts.
      const result = await backend.downloads.importBatch(selected, { mode: 'now' });
      result.added.forEach(upsert);
      onImportSuccess?.(result);
    } catch (error) {
      onImportError?.(error instanceof Error ? error.message : String(error));
    } finally {
      setLoading(false);
    }
  };

  return (
    <Button
      color="inherit"
      startIcon={loading ? <CircularProgress size={14} /> : <UploadIcon fontSize="small" />}
      onClick={handleImport}
      disabled={loading}
    >
      {t('downloads.batch.import')}
    </Button>
  );
};

export default BatchImportButton;
