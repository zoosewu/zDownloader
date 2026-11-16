// The add-downloads window: a fixed one-line input on top, the list (the only part that
// scrolls), and fixed options at the bottom. Links arrive from the clipboard, the browser
// extension, an imported or dropped text file, links dragged in, or pasted here (several at
// once). Each row shows which module will resolve it and its categories (filled in from the
// rules; changing them makes them the user's choice).
import React, { useCallback, useEffect, useRef, useState } from 'react';
import {
  Alert,
  Box,
  Button,
  Checkbox,
  Dialog,
  DialogActions,
  DialogContent,
  DialogTitle,
  FormControlLabel,
  IconButton,
  ListItemText,
  MenuItem,
  Radio,
  RadioGroup,
  Select,
  TextField,
  Tooltip,
  Typography,
} from '@mui/material';
import { Close as RemoveIcon, Cookie as BrowserIcon, Upload as UploadIcon, FolderOpen as FolderIcon } from '@mui/icons-material';
import { useTranslation } from 'react-i18next';
import { open } from '@tauri-apps/plugin-dialog';
import { getCurrentWindow } from '@tauri-apps/api/window';
import { backend, type AddItem, type InstalledInfo, type StartMode } from '../backend';
import { useCategories } from '../hooks/useCategories';
import { useDownloadPath } from '../hooks/useDownloadPath';
import { invoke } from '../utils/tauri';
import { linksIn } from '../utils/links';
import HeaderFieldsEditor from '../components/HeaderFieldsEditor';
import { emptyHeaderFields, fromHeaderFields, type HeaderFields, type HeaderPairs } from '../utils/headers';

interface Row {
  key: number;
  url: string;
  /** From the browser extension; the "Headers" option adds to them. */
  headers: HeaderPairs;
  /** Shown in the name field: the link's own name, or what the user typed. */
  filename: string;
  /** The user typed the name: it is kept even if the server or module names the file. */
  filenameEdited: boolean;
  module: InstalledInfo | null;
  tags: string[];
  /** The user changed the categories: they are sent as the user's choice. */
  tagsEdited: boolean;
  error: string | null;
}

type Start = 'now' | 'paused' | 'at';

let nextKey = 1;

/** Row headers first; an option header of the same name replaces it. */
function mergeHeaders(own: HeaderPairs, extra: HeaderPairs): HeaderPairs {
  const names = new Set(extra.map(([n]) => n.toLowerCase()));
  return [...own.filter(([n]) => !names.has(n.toLowerCase())), ...extra];
}

const hostOf = (url: string) => {
  try {
    return new URL(url).host;
  } catch {
    return url;
  }
};

const AddDownloads: React.FC = () => {
  const { t } = useTranslation();
  const { categories } = useCategories();
  const { path, openDirectoryPicker } = useDownloadPath();
  const [rows, setRows] = useState<Row[]>([]);
  const [input, setInput] = useState('');
  const [start, setStart] = useState<Start>('now');
  const [scheduleAt, setScheduleAt] = useState('');
  const [headers, setHeaders] = useState<HeaderFields>(emptyHeaderFields);
  const [editingHeaders, setEditingHeaders] = useState(false);
  const [dragging, setDragging] = useState(false);
  const [adding, setAdding] = useState(false);
  const [message, setMessage] = useState<string | null>(null);
  /** Everything submitted was added: close once the list is empty. */
  const [done, setDone] = useState(false);
  const rowsRef = useRef(rows);
  rowsRef.current = rows;

  const update = (key: number, patch: Partial<Row>) => setRows((all) => all.map((r) => (r.key === key ? { ...r, ...patch } : r)));

  /** Module, the link's own name, and the categories the rules give it. */
  const inspect = useCallback(async (key: number, url: string, filename: string | null) => {
    const info = await backend.urls.inspect(url).catch(() => null);
    const name = filename ?? (info?.module ? null : (info?.filename ?? null));
    const tags = await backend.categories.match(url, name).catch(() => [] as string[]);
    setRows((all) =>
      all.map((r) =>
        r.key !== key
          ? r
          : {
              ...r,
              module: info?.module ?? null,
              filename: r.filenameEdited ? r.filename : (name ?? ''),
              tags: r.tagsEdited ? r.tags : tags,
            },
      ),
    );
  }, []);

  const addItems = useCallback(
    (items: AddItem[]) => {
      const known = new Set(rowsRef.current.map((r) => r.url));
      const added: Row[] = [];
      for (const i of items) {
        if (known.has(i.url)) continue;
        known.add(i.url);
        added.push({
          key: nextKey++,
          url: i.url,
          headers: i.headers,
          filename: i.filename ?? '',
          filenameEdited: !!i.filename,
          module: null,
          tags: [],
          tagsEdited: false,
          error: null,
        });
      }
      setRows((all) => [...all, ...added]);
      added.forEach((r) => inspect(r.key, r.url, r.filenameEdited ? r.filename : null));
    },
    [inspect],
  );
  const addLinks = useCallback((text: string) => {
    const links = linksIn(text);
    addItems(links.map((url) => ({ url, filename: null, headers: [] })));
    return links.length;
  }, [addItems]);

  // Links waiting in the backend: now, and whenever more arrive while the window is open.
  useEffect(() => {
    const take = () => backend.addWindow.take().then(addItems).catch(() => {});
    take();
    let stop: (() => void) | undefined;
    let disposed = false;
    backend.events.onAddItemsReady(take).then((u) => (disposed ? u() : (stop = u)));
    return () => {
      disposed = true;
      stop?.();
    };
  }, [addItems]);

  // Opened with nothing to add: offer the link on the clipboard, if it is one.
  useEffect(() => {
    const timer = setTimeout(async () => {
      if (rowsRef.current.length > 0) return;
      const content = (await invoke<string>('get_clipboard_content').catch(() => '')).trim();
      if (linksIn(content).length === 1) setInput((v) => v || content);
    }, 300);
    return () => clearTimeout(timer);
  }, []);

  const addInput = () => {
    if (addLinks(input) > 0) setInput('');
  };

  const importFile = async () => {
    const file = await open({ multiple: false, filters: [{ name: 'Text', extensions: ['txt'] }] });
    // The backend reads the file and hands the links back through `take`.
    if (file) await backend.addWindow.import(file).catch((e) => setMessage(e.message));
  };

  // Links dragged from a browser, or text files from Explorer.
  const onDrop = async (e: React.DragEvent) => {
    e.preventDefault();
    setDragging(false);
    const texts = await Promise.all([...e.dataTransfer.files].filter((f) => /\.txt$/i.test(f.name)).map((f) => f.text()));
    texts.push(e.dataTransfer.getData('text/uri-list'), e.dataTransfer.getData('text/plain'));
    addLinks(texts.join('\n'));
  };

  const submit = async () => {
    setAdding(true);
    setMessage(null);
    const how: StartMode =
      start === 'at' && scheduleAt ? { mode: 'at', at: new Date(scheduleAt).toISOString() } : start === 'paused' ? { mode: 'later' } : { mode: 'now' };
    const extra = fromHeaderFields(headers);
    let failed = 0;
    for (const row of rowsRef.current) {
      try {
        await backend.downloads.add({
          url: row.url,
          // A name the user left alone may still be replaced by the server's or the module's.
          filename: row.filenameEdited ? row.filename.trim() || null : null,
          dir: path,
          start: how,
          headers: mergeHeaders(row.headers, extra),
          tags: row.tagsEdited ? row.tags : null,
        });
        setRows((all) => all.filter((r) => r.key !== row.key));
      } catch (e) {
        failed++;
        update(row.key, { error: e instanceof Error ? e.message : String(e) });
      }
    }
    setAdding(false);
    setDone(failed === 0);
  };

  const close = () => getCurrentWindow().close().catch(() => {});
  // Links that arrived while adding keep the window open.
  useEffect(() => {
    if (done && rows.length === 0) close();
    else if (done) setDone(false);
  }, [done, rows.length]);
  const customHeaders = fromHeaderFields(headers).length;
  const fixed = { flexShrink: 0, bgcolor: 'background.paper', borderColor: 'divider' };

  return (
    <Box
      sx={{ display: 'flex', flexDirection: 'column', height: '100%' }}
      onDragOver={(e) => {
        e.preventDefault();
        setDragging(true);
      }}
      onDragLeave={(e) => e.currentTarget === e.target && setDragging(false)}
      onDrop={onDrop}
    >
      {/* Input: one line; pasting several links adds them all to the list. */}
      <Box sx={{ ...fixed, display: 'flex', gap: 0.75, p: 1, alignItems: 'center', borderBottom: 1 }}>
        <TextField
          value={input}
          onChange={(e) => setInput(e.target.value)}
          onKeyDown={(e) => e.key === 'Enter' && addInput()}
          onPaste={(e) => {
            const text = e.clipboardData.getData('text');
            if (linksIn(text).length > 1) {
              e.preventDefault();
              addLinks(text);
            }
          }}
          placeholder={t('addWindow.pastePlaceholder')}
          fullWidth
          autoFocus
          slotProps={{ htmlInput: { spellCheck: false } }}
        />
        <Button variant="outlined" onClick={addInput} disabled={linksIn(input).length === 0}>
          {t('addWindow.add')}
        </Button>
        <Button color="inherit" startIcon={<UploadIcon fontSize="small" />} onClick={importFile}>
          {t('addWindow.importList')}
        </Button>
      </Box>

      {/* The list: the only part that scrolls. */}
      <Box sx={{ flex: 1, overflow: 'auto', position: 'relative', outline: dragging ? 2 : 0, outlineColor: 'primary.main', outlineOffset: -2 }}>
        {rows.length === 0 ? (
          <Typography variant="body2" color="text.secondary" sx={{ p: 4, textAlign: 'center' }}>
            {t('addWindow.empty')}
          </Typography>
        ) : (
          rows.map((row) => (
            <Box
              key={row.key}
              sx={{
                display: 'grid',
                gridTemplateColumns: 'minmax(0, 1fr) 180px 28px',
                gap: 1,
                alignItems: 'center',
                px: 1,
                py: 0.75,
                borderBottom: 1,
                borderColor: 'divider',
                '&:hover .remove': { visibility: 'visible' },
              }}
            >
              <Box sx={{ minWidth: 0 }}>
                <TextField
                  value={row.filename}
                  placeholder={row.module ? t('addWindow.namedBySite') : ''}
                  onChange={(e) => update(row.key, { filename: e.target.value, filenameEdited: true })}
                  onBlur={() => !row.tagsEdited && row.filenameEdited && inspect(row.key, row.url, row.filename.trim() || null)}
                  fullWidth
                  variant="standard"
                  slotProps={{ htmlInput: { spellCheck: false } }}
                />
                <Box sx={{ display: 'flex', alignItems: 'center', gap: 0.5, mt: 0.25, minWidth: 0 }}>
                  {row.headers.length > 0 && (
                    <Tooltip title={`${t('addWindow.browserHeaders')}: ${row.headers.map(([n]) => n).join(', ')}`}>
                      <BrowserIcon sx={{ fontSize: 13, color: 'text.secondary' }} />
                    </Tooltip>
                  )}
                  <Typography
                    variant="caption"
                    noWrap
                    color={row.error ? 'error' : 'text.secondary'}
                    title={row.error ?? row.url}
                    sx={{ minWidth: 0 }}
                  >
                    {row.error ??
                      `${hostOf(row.url)} · ${
                        row.module ? t('modules.match.resolvedBy', { name: row.module.name }) : t('downloads.details.directDownload')
                      }`}
                  </Typography>
                </Box>
              </Box>
              <Select
                multiple
                fullWidth
                variant="standard"
                value={row.tags}
                displayEmpty
                disabled={categories.length === 0}
                onChange={(e) => update(row.key, { tags: typeof e.target.value === 'string' ? [] : e.target.value, tagsEdited: true })}
                renderValue={(tags) =>
                  tags.length === 0 ? (
                    <Typography variant="body2" color="text.disabled" component="span">
                      {categories.length === 0 ? t('categories.none') : t('addWindow.noCategory')}
                    </Typography>
                  ) : (
                    categories.filter((c) => tags.includes(c.id)).map((c) => c.name).join(', ')
                  )
                }
              >
                {categories.map((c) => (
                  <MenuItem key={c.id} value={c.id} sx={{ py: 0 }}>
                    <Checkbox checked={row.tags.includes(c.id)} sx={{ p: 0.5, mr: 0.5 }} />
                    <ListItemText primary={c.name} />
                  </MenuItem>
                ))}
              </Select>
              <Tooltip title={t('addWindow.remove')}>
                <IconButton className="remove" sx={{ visibility: 'hidden' }} onClick={() => setRows((all) => all.filter((r) => r.key !== row.key))}>
                  <RemoveIcon fontSize="small" />
                </IconButton>
              </Tooltip>
            </Box>
          ))
        )}
      </Box>

      {/* Options for every row: fixed at the bottom. */}
      <Box sx={{ ...fixed, borderTop: 1, p: 1, display: 'grid', gridTemplateColumns: 'auto 1fr', columnGap: 1.5, rowGap: 0.75, alignItems: 'center' }}>
        <Typography variant="body2" color="text.secondary">
          {t('addWindow.saveTo')}
        </Typography>
        <Box sx={{ display: 'flex', gap: 0.75 }}>
          <TextField value={path} fullWidth slotProps={{ input: { readOnly: true } }} />
          <Button color="inherit" startIcon={<FolderIcon fontSize="small" />} onClick={openDirectoryPicker}>
            {t('common.browse')}
          </Button>
        </Box>

        <Typography variant="body2" color="text.secondary">
          {t('addWindow.start')}
        </Typography>
        <Box sx={{ display: 'flex', alignItems: 'center', gap: 1, flexWrap: 'wrap' }}>
          <RadioGroup row value={start} onChange={(e) => setStart(e.target.value as Start)}>
            <FormControlLabel value="now" control={<Radio />} label={t('addWindow.startNow')} />
            <FormControlLabel value="paused" control={<Radio />} label={t('addWindow.startPaused')} />
            <FormControlLabel value="at" control={<Radio />} label={t('addWindow.startAt')} />
          </RadioGroup>
          <TextField
            type="datetime-local"
            value={scheduleAt}
            disabled={start !== 'at'}
            onChange={(e) => setScheduleAt(e.target.value)}
            sx={{ width: 220 }}
          />
          <Box sx={{ flex: 1 }} />
          <Button color="inherit" onClick={() => setEditingHeaders(true)}>
            {customHeaders ? t('addWindow.headersCount', { count: customHeaders }) : t('addWindow.headers')}
          </Button>
        </Box>

        {message && (
          <Alert severity="error" sx={{ gridColumn: '1 / -1' }}>
            {message}
          </Alert>
        )}
        <Box sx={{ gridColumn: '1 / -1', display: 'flex', alignItems: 'center', gap: 1, pt: 0.5 }}>
          <Typography variant="body2" color="text.secondary" sx={{ flex: 1 }}>
            {t('addWindow.count', { count: rows.length })}
          </Typography>
          <Button color="inherit" onClick={close}>
            {t('common.cancel')}
          </Button>
          <Button variant="contained" onClick={submit} disabled={adding || rows.length === 0 || (start === 'at' && !scheduleAt)}>
            {t('addWindow.addAll')}
          </Button>
        </Box>
      </Box>

      <Dialog open={editingHeaders} onClose={() => setEditingHeaders(false)} maxWidth="sm" fullWidth>
        <DialogTitle>{t('addWindow.headersTitle')}</DialogTitle>
        <DialogContent>
          <HeaderFieldsEditor value={headers} onChange={setHeaders} />
        </DialogContent>
        <DialogActions>
          <Button variant="contained" onClick={() => setEditingHeaders(false)}>
            {t('common.close')}
          </Button>
        </DialogActions>
      </Dialog>
    </Box>
  );
};

export default AddDownloads;
