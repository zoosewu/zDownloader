// Change a stopped task's URL (a mirror, a fresh share link) or its headers.
import { useState } from 'react';
import { Button, Dialog, DialogActions, DialogContent, DialogTitle, TextField } from '@mui/material';
import DialogError from './DialogError';
import { useSetAtom } from 'jotai';
import { useTranslation } from 'react-i18next';
import { backend, type Task } from '../backend';
import { upsertTaskAtom } from '../stores/downloadStore';
import { fromHeaderFields, toHeaderFields, type HeaderFields } from '../utils/headers';
import HeaderFieldsEditor from './HeaderFieldsEditor';

/** States the backend lets `update` change. */
export const editable = (task: Task) => ['paused', 'failed', 'queued', 'scheduled'].includes(task.state);

interface EditTaskDialogProps {
  task: Task | null;
  onClose: () => void;
}

export default function EditTaskDialog({ task, onClose }: EditTaskDialogProps) {
  // Mounted per opening, so the fields start from the task as it is then.
  return task ? <EditTask task={task} onClose={onClose} /> : null;
}

function EditTask({ task, onClose }: { task: Task; onClose: () => void }) {
  const { t } = useTranslation();
  const upsert = useSetAtom(upsertTaskAtom);
  const [url, setUrl] = useState(task.url);
  const [fields, setFields] = useState<HeaderFields>(() => toHeaderFields(task.headers).fields);
  const [error, setError] = useState<string | null>(null);
  const [saving, setSaving] = useState(false);

  const save = async () => {
    setSaving(true);
    try {
      const headers = fromHeaderFields(fields, toHeaderFields(task.headers).rest);
      const changedUrl = url.trim() !== task.url ? url.trim() : null;
      upsert(await backend.downloads.update(task.id, { url: changedUrl, headers }));
      onClose();
    } catch (e) {
      setError((e as Error).message);
    } finally {
      setSaving(false);
    }
  };

  return (
    <Dialog open onClose={onClose} fullWidth maxWidth="sm">
      <DialogTitle>{t('downloads.edit.title')}</DialogTitle>
      <DialogContent>
        <TextField
          label={t('downloads.urlLabel')}
          value={url}
          onChange={(e) => setUrl(e.target.value)}
          helperText={t('downloads.edit.urlHelp')}
          fullWidth
          margin="normal"
          autoFocus
        />
        <HeaderFieldsEditor value={fields} onChange={setFields} />
      </DialogContent>
      <DialogActions>
        <DialogError error={error} />
        <Button onClick={onClose}>{t('common.cancel')}</Button>
        <Button variant="contained" onClick={save} disabled={saving || !url.trim()}>
          {t('common.save')}
        </Button>
      </DialogActions>
    </Dialog>
  );
}
