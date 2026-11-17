// Left sidebar: status views and the user's categories, each with its task count. Collapses
// to an icon rail. Reads only the view counts, so download progress does not re-render it.
import React, { useState } from 'react';
import {
  Badge,
  Box,
  Button,
  Dialog,
  DialogActions,
  DialogContent,
  DialogContentText,
  IconButton,
  List,
  ListItemButton,
  ListItemIcon,
  ListItemText,
  Menu,
  MenuItem,
  Tooltip,
  Typography,
} from '@mui/material';
import {
  AllInbox as AllIcon,
  Download as RunningIcon,
  Schedule as WaitingIcon,
  PauseCircleOutlined as PausedIcon,
  CheckCircleOutlined as CompletedIcon,
  ErrorOutlined as FailedIcon,
  Block as CancelledIcon,
  LabelOutlined as CategoryIcon,
  Add as AddIcon,
  KeyboardDoubleArrowLeft as CollapseIcon,
  KeyboardDoubleArrowRight as ExpandIcon,
} from '@mui/icons-material';
import { useAtom, useAtomValue } from 'jotai';
import { useTranslation } from 'react-i18next';
import { STATUS_VIEWS, categoryView, viewAtom, viewCountsAtom, type StatusView, type View } from '../stores/viewStore';
import { useCategories } from '../hooks/useCategories';
import CategoryDialog from './CategoryDialog';
import type { Category } from '../backend';

const STATUS_ICON: Record<StatusView, React.ReactElement> = {
  all: <AllIcon fontSize="small" />,
  running: <RunningIcon fontSize="small" />,
  waiting: <WaitingIcon fontSize="small" />,
  paused: <PausedIcon fontSize="small" />,
  completed: <CompletedIcon fontSize="small" />,
  failed: <FailedIcon fontSize="small" />,
  cancelled: <CancelledIcon fontSize="small" />,
};

const COLLAPSED_KEY = 'zoo.sidebar.collapsed';
const loadCollapsed = () => {
  try {
    return localStorage.getItem(COLLAPSED_KEY) === '1';
  } catch {
    return false;
  }
};

const NEW_CATEGORY: Category = { id: '', name: '', match: 'any', rules: [] };

const Sidebar: React.FC = () => {
  const { t } = useTranslation();
  const [view, setView] = useAtom(viewAtom);
  const counts = useAtomValue(viewCountsAtom);
  const { categories, save } = useCategories();
  const [collapsed, setCollapsed] = useState(loadCollapsed);
  const [editing, setEditing] = useState<Category | null>(null);
  const [menu, setMenu] = useState<{ x: number; y: number; category: Category } | null>(null);
  const [deleting, setDeleting] = useState<Category | null>(null);

  const toggle = () => {
    setCollapsed((c) => {
      try {
        localStorage.setItem(COLLAPSED_KEY, c ? '0' : '1');
      } catch {
        // Not remembered; fine.
      }
      return !c;
    });
  };

  const upsert = (category: Category) => {
    const exists = categories.some((c) => c.id === category.id);
    return save(exists ? categories.map((c) => (c.id === category.id ? category : c)) : [...categories, category]);
  };

  const item = (key: View, label: string, icon: React.ReactElement, onContextMenu?: (e: React.MouseEvent) => void) => {
    const count = counts[key] ?? 0;
    const button = (
      <ListItemButton
        key={key}
        selected={view === key}
        onClick={() => setView(key)}
        onContextMenu={onContextMenu}
        sx={{ minHeight: 28, px: 1, py: 0.25, mx: 0.5, justifyContent: collapsed ? 'center' : undefined }}
      >
        <ListItemIcon sx={{ minWidth: collapsed ? 0 : 28, color: count === 0 ? 'text.disabled' : undefined }}>
          {collapsed ? (
            <Badge badgeContent={count} max={99} color="default" variant="standard" sx={{ '& .MuiBadge-badge': { fontSize: 10, height: 15, minWidth: 15, px: 0.5 } }}>
              {icon}
            </Badge>
          ) : (
            icon
          )}
        </ListItemIcon>
        {!collapsed && (
          <>
            <ListItemText
              primary={label}
              slotProps={{ primary: { noWrap: true, sx: { fontSize: '0.8125rem', color: count === 0 ? 'text.secondary' : undefined } } }}
            />
            <Typography variant="caption" color="text.secondary" sx={{ ml: 1, fontVariantNumeric: 'tabular-nums' }}>
              {count || ''}
            </Typography>
          </>
        )}
      </ListItemButton>
    );
    return collapsed ? (
      <Tooltip key={key} title={label} placement="right">
        {button}
      </Tooltip>
    ) : (
      button
    );
  };

  const heading = (label: string, action?: React.ReactNode) =>
    collapsed ? (
      <Box sx={{ height: 9 }} />
    ) : (
      <Box sx={{ display: 'flex', alignItems: 'center', px: 1.5, pt: 1.25, pb: 0.25, minHeight: 28 }}>
        <Typography variant="caption" color="text.secondary" sx={{ flex: 1, fontWeight: 600 }}>
          {label}
        </Typography>
        {action}
      </Box>
    );

  return (
    <Box
      component="nav"
      sx={{
        width: collapsed ? 48 : 200,
        flexShrink: 0,
        display: 'flex',
        flexDirection: 'column',
        borderRight: 1,
        borderColor: 'divider',
        backgroundColor: 'background.paper',
        transition: 'width 0.15s',
        overflow: 'hidden',
      }}
    >
      <Box sx={{ flex: 1, overflowY: 'auto', overflowX: 'hidden' }}>
        {heading(t('sidebar.status'))}
        <List disablePadding>{STATUS_VIEWS.map((s) => item(s, t(`sidebar.views.${s}`), STATUS_ICON[s]))}</List>

        {heading(
          t('sidebar.categories'),
          <Tooltip title={t('categories.add')}>
            <IconButton onClick={() => setEditing(NEW_CATEGORY)} sx={{ p: '2px' }}>
              <AddIcon sx={{ fontSize: 16 }} />
            </IconButton>
          </Tooltip>,
        )}
        <List disablePadding>
          {categories.map((c) =>
            item(categoryView(c.id), c.name, <CategoryIcon fontSize="small" />, (e) => {
              e.preventDefault();
              setMenu({ x: e.clientX, y: e.clientY, category: c });
            }),
          )}
          {collapsed && (
            <Tooltip title={t('categories.add')} placement="right">
              <ListItemButton onClick={() => setEditing(NEW_CATEGORY)} sx={{ minHeight: 28, mx: 0.5, justifyContent: 'center' }}>
                <ListItemIcon sx={{ minWidth: 0 }}>
                  <AddIcon fontSize="small" />
                </ListItemIcon>
              </ListItemButton>
            </Tooltip>
          )}
        </List>
      </Box>

      <Box sx={{ borderTop: 1, borderColor: 'divider', display: 'flex', justifyContent: collapsed ? 'center' : 'flex-end', p: 0.25 }}>
        <Tooltip title={collapsed ? t('sidebar.expand') : t('sidebar.collapse')} placement="right">
          <IconButton onClick={toggle}>{collapsed ? <ExpandIcon fontSize="small" /> : <CollapseIcon fontSize="small" />}</IconButton>
        </Tooltip>
      </Box>

      <Menu
        open={!!menu}
        onClose={() => setMenu(null)}
        anchorReference="anchorPosition"
        anchorPosition={menu ? { top: menu.y, left: menu.x } : undefined}
      >
        <MenuItem
          onClick={() => {
            setEditing(menu!.category);
            setMenu(null);
          }}
        >
          {t('categories.edit')}
        </MenuItem>
        <MenuItem
          onClick={() => {
            setDeleting(menu!.category);
            setMenu(null);
          }}
          sx={{ color: 'error.main' }}
        >
          {t('categories.delete')}
        </MenuItem>
      </Menu>

      <CategoryDialog category={editing} onClose={() => setEditing(null)} onSave={upsert} />

      <Dialog open={!!deleting} onClose={() => setDeleting(null)}>
        <DialogContent>
          <DialogContentText>{t('categories.confirmDelete', { name: deleting?.name })}</DialogContentText>
        </DialogContent>
        <DialogActions>
          <Button color="inherit" onClick={() => setDeleting(null)}>
            {t('common.cancel')}
          </Button>
          <Button
            color="error"
            variant="contained"
            onClick={() => {
              const gone = deleting!;
              setDeleting(null);
              save(categories.filter((c) => c.id !== gone.id)).catch((e) => console.error('delete category:', e.message));
            }}
          >
            {t('categories.delete')}
          </Button>
        </DialogActions>
      </Dialog>
    </Box>
  );
};

export default Sidebar;
