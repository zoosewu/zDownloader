// A task's new URL serves a different file: ask whether to start over (the default) or
// try another URL. Asked once per failure.
import { useState } from 'react';
import { Button, Dialog, DialogActions, DialogContent, DialogContentText, DialogTitle } from '@mui/material';
import { useAtomValue } from 'jotai';
import { useTranslation } from 'react-i18next';
import type { Task, TaskId } from '../backend';
import { downloadTasksAtom } from '../stores/downloadStore';
import { useDownloadActions } from '../hooks/useDownloadActions';
import EditTaskDialog from './EditTaskDialog';

const changed = (task: Task) => task.state === 'failed' && task.error?.kind === 'sourceChanged';

export default function SourceChangedPrompt() {
  const { t } = useTranslation();
  const tasks = useAtomValue(downloadTasksAtom);
  const actions = useDownloadActions();
  /** Failures already asked about: task id -> the revision that failed. */
  const [asked, setAsked] = useState<Record<TaskId, number>>({});
  const [editing, setEditing] = useState<Task | null>(null);
  const [error, setError] = useState<string | null>(null);

  const current = tasks.find((task) => changed(task) && asked[task.id] !== task.rev) ?? null;

  const answer = (task: Task, action: () => Promise<unknown> | void) => async () => {
    setAsked((prev) => ({ ...prev, [task.id]: task.rev }));
    setError(null);
    try {
      await action();
    } catch (e) {
      setError((e as Error).message);
    }
  };

  return (
    <>
      {current && (
        <Dialog open onClose={answer(current, () => {})} maxWidth="sm" fullWidth>
          <DialogTitle>{t('downloads.sourceChanged.title')}</DialogTitle>
          <DialogContent>
            <DialogContentText>{t('downloads.sourceChanged.message', { name: current.name })}</DialogContentText>
            {current.error?.kind === 'sourceChanged' && (
              <DialogContentText variant="body2" sx={{ mt: 1 }}>
                {current.error.detail}
              </DialogContentText>
            )}
          </DialogContent>
          <DialogActions>
            <Button onClick={answer(current, () => {})}>{t('common.later')}</Button>
            <Button onClick={answer(current, () => setEditing(current))}>{t('downloads.sourceChanged.otherUrl')}</Button>
            <Button variant="contained" autoFocus onClick={answer(current, () => actions.restart(current.id))}>
              {t('downloads.sourceChanged.restart')}
            </Button>
          </DialogActions>
        </Dialog>
      )}
      {error && (
        <Dialog open onClose={() => setError(null)}>
          <DialogContent>{error}</DialogContent>
          <DialogActions>
            <Button onClick={() => setError(null)}>{t('common.close')}</Button>
          </DialogActions>
        </Dialog>
      )}
      <EditTaskDialog task={editing} onClose={() => setEditing(null)} />
    </>
  );
}
