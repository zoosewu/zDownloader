// Asks before finished downloads' files go to the recycle bin with their downloads.
import React from 'react';
import { Button, Dialog, DialogActions, DialogContent, DialogTitle } from '@mui/material';
import { useTranslation } from 'react-i18next';

const DeleteFilesDialog: React.FC<{ open: boolean; count: number; onConfirm: () => void; onClose: () => void }> = ({
  open,
  count,
  onConfirm,
  onClose,
}) => {
  const { t } = useTranslation();
  // Opened from a row, its events would bubble (through the portal) to the row: selection,
  // dragging, the row's menu.
  const stop = (e: React.SyntheticEvent) => e.stopPropagation();
  return (
    <Dialog open={open} onClose={onClose} onClick={stop} onPointerDown={stop} onKeyDown={stop} onContextMenu={stop}>
      <DialogTitle>{t('downloads.menu.deleteTitle')}</DialogTitle>
      <DialogContent>{t('downloads.menu.deleteConfirm', { count })}</DialogContent>
      <DialogActions>
        <Button color="inherit" onClick={onClose}>
          {t('common.cancel')}
        </Button>
        <Button color="error" variant="contained" onClick={onConfirm}>
          {t('downloads.menu.deleteAction')}
        </Button>
      </DialogActions>
    </Dialog>
  );
};

export default DeleteFilesDialog;
