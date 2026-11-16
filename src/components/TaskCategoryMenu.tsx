// Right-click menu of a download: tick the categories it belongs to. Choosing by hand stops
// the category rules from changing this download.
import React, { useMemo } from 'react';
import { Checkbox, ListItemIcon, ListItemText, Menu, MenuItem } from '@mui/material';
import { useAtomValue, useSetAtom } from 'jotai';
import { useTranslation } from 'react-i18next';
import { backend, type TaskId } from '../backend';
import { taskAtom, upsertTaskAtom } from '../stores/downloadStore';
import { useCategories } from '../hooks/useCategories';

export interface MenuAnchor {
  id: TaskId;
  x: number;
  y: number;
}

const TaskCategoryMenu: React.FC<{ anchor: MenuAnchor; onClose: () => void; onError?: (message: string) => void }> = ({
  anchor,
  onClose,
  onError,
}) => {
  const { t } = useTranslation();
  const task = useAtomValue(useMemo(() => taskAtom(anchor.id), [anchor.id]));
  const upsert = useSetAtom(upsertTaskAtom);
  const { categories } = useCategories();
  if (!task) return null;

  const toggle = (id: string) => {
    const tags = task.tags.includes(id) ? task.tags.filter((tag) => tag !== id) : [...task.tags, id];
    backend.downloads
      .setTags(task.id, tags)
      .then(upsert)
      .catch((e) => onError?.(e.message));
  };

  return (
    <Menu open onClose={onClose} anchorReference="anchorPosition" anchorPosition={{ top: anchor.y, left: anchor.x }}>
      {categories.length === 0 && <MenuItem disabled>{t('categories.none')}</MenuItem>}
      {categories.map((c) => (
        <MenuItem key={c.id} onClick={() => toggle(c.id)} sx={{ py: 0 }}>
          <ListItemIcon>
            <Checkbox edge="start" checked={task.tags.includes(c.id)} tabIndex={-1} disableRipple sx={{ p: 0.5 }} />
          </ListItemIcon>
          <ListItemText>{c.name}</ListItemText>
        </MenuItem>
      ))}
    </Menu>
  );
};

export default TaskCategoryMenu;
