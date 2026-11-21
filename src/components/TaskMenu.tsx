// Right-click menu of the selected downloads. Each action goes to the selected downloads whose
// state allows it, and shows only when at least one does. Choosing categories by hand stops the
// category rules from changing those downloads.
import React, { useState } from 'react';
import {
  Checkbox,
  Divider,
  ListItemIcon,
  ListItemText,
  Menu,
  MenuItem,
} from '@mui/material';
import { ChevronRight as SubmenuIcon } from '@mui/icons-material';
import { useAtomValue, useSetAtom } from 'jotai';
import { useTranslation } from 'react-i18next';
import { backend, type Task, type TaskId } from '../backend';
import { tasksByIdAtom, upsertTaskAtom } from '../stores/downloadStore';
import { useCategories } from '../hooks/useCategories';
import { useDownloadActions } from '../hooks/useDownloadActions';
import { actionsFor, canRestart, type Action } from '../utils/taskActions';
import DeleteFilesDialog from './DeleteFilesDialog';

export interface MenuAnchor {
  x: number;
  y: number;
}

const TaskMenu: React.FC<{ anchor: MenuAnchor; ids: TaskId[]; onClose: () => void; onError?: (message: string) => void }> = ({
  anchor,
  ids,
  onClose,
  onError,
}) => {
  const { t } = useTranslation();
  const byId = useAtomValue(tasksByIdAtom);
  const upsert = useSetAtom(upsertTaskAtom);
  const actions = useDownloadActions();
  const { categories } = useCategories();
  const [categoriesAt, setCategoriesAt] = useState<HTMLElement | null>(null);
  const [confirming, setConfirming] = useState(false);

  const tasks = ids.map((id) => byId[id]).filter((task): task is Task => task !== undefined);
  if (tasks.length === 0) return null;

  const fail = (e: unknown) => onError?.(e instanceof Error ? e.message : String(e));
  /** Close the menu, then do `work` for each of `on`. */
  const run = (on: Task[], work: (task: Task) => Promise<unknown>) => () => {
    onClose();
    Promise.all(on.map(work)).catch(fail);
  };
  const can = (action: Action) => tasks.filter((task) => actionsFor(task.state).includes(action));
  const pausable = can('pause');
  const resumable = can('resume');
  const retryable = can('retry');
  const restartable = tasks.filter((task) => canRestart(task.state));
  const finished = tasks.filter((task) => task.state === 'completed');
  const only = tasks.length === 1 ? tasks[0] : null;

  const toggleCategory = (id: string) => {
    // On for all unless every one already has it.
    const on = !tasks.every((task) => task.tags.includes(id));
    Promise.all(
      tasks.map((task) =>
        backend.downloads
          .setTags(task.id, on ? [...new Set([...task.tags, id])] : task.tags.filter((tag) => tag !== id))
          .then(upsert),
      ),
    ).catch(fail);
  };

  const item = (label: string, onClick: () => void) => (
    <MenuItem key={label} onClick={onClick}>
      {label}
    </MenuItem>
  );

  return (
    <>
      <Menu
        open={!confirming}
        onClose={onClose}
        anchorReference="anchorPosition"
        anchorPosition={{ top: anchor.y, left: anchor.x }}
        slotProps={{ list: { dense: true } }}
      >
        {resumable.length > 0 && item(t('downloads.actions.resume'), run(resumable, (task) => actions.resume(task.id)))}
        {pausable.length > 0 && item(t('downloads.actions.pause'), run(pausable, (task) => actions.pause(task.id)))}
        {retryable.length > 0 && item(t('downloads.actions.retry'), run(retryable, (task) => actions.retry(task.id)))}
        {restartable.length > 0 && item(t('downloads.menu.restart'), run(restartable, (task) => actions.restart(task.id)))}
        {(resumable.length > 0 || pausable.length > 0 || retryable.length > 0 || restartable.length > 0) && <Divider />}

        {only?.state === 'completed' && item(t('downloads.actions.openFile'), run([only], (task) => backend.downloads.openFile(task.id)))}
        {only?.state === 'completed' && item(t('downloads.actions.openFolder'), run([only], (task) => backend.downloads.showInFolder(task.id)))}
        {item(t('downloads.actions.copyUrl'), () => {
          onClose();
          backend.clipboard.copy(tasks.map((task) => task.url).join('\n')).catch(fail);
        })}
        {categories.length > 0 && (
          <MenuItem onClick={(e) => setCategoriesAt(e.currentTarget)}>
            <ListItemText>{t('downloads.menu.categories')}</ListItemText>
            <SubmenuIcon fontSize="small" sx={{ ml: 2, color: 'text.secondary' }} />
          </MenuItem>
        )}
        <Divider />

        {item(t('downloads.actions.remove'), run(tasks, (task) => actions.remove(task.id)))}
        {finished.length > 0 && item(t('downloads.menu.removeWithFile'), () => setConfirming(true))}
      </Menu>

      <Menu
        open={categoriesAt !== null && !confirming}
        anchorEl={categoriesAt}
        onClose={() => setCategoriesAt(null)}
        anchorOrigin={{ vertical: 'top', horizontal: 'right' }}
        slotProps={{ list: { dense: true } }}
      >
        {categories.map((c) => {
          const all = tasks.every((task) => task.tags.includes(c.id));
          const some = !all && tasks.some((task) => task.tags.includes(c.id));
          return (
            <MenuItem key={c.id} onClick={() => toggleCategory(c.id)} sx={{ py: 0 }}>
              <ListItemIcon>
                <Checkbox edge="start" checked={all} indeterminate={some} tabIndex={-1} disableRipple sx={{ p: 0.5 }} />
              </ListItemIcon>
              <ListItemText>{c.name}</ListItemText>
            </MenuItem>
          );
        })}
      </Menu>

      <DeleteFilesDialog
        open={confirming}
        count={finished.length}
        onConfirm={run(tasks, (task) => actions.remove(task.id, task.state === 'completed'))}
        onClose={onClose}
      />
    </>
  );
};

export default TaskMenu;
