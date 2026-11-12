// Settings → Modules: installed modules, the marketplaces they come from, and what those offer.
import { useState } from 'react';
import {
  Alert,
  Box,
  Button,
  Chip,
  CircularProgress,
  Dialog,
  DialogActions,
  DialogContent,
  DialogContentText,
  FormControlLabel,
  IconButton,
  Paper,
  Stack,
  Switch,
  Tab,
  Tabs,
  TextField,
  Tooltip,
  Typography,
} from '@mui/material';
import {
  ArrowDownward as DownIcon,
  ArrowUpward as UpIcon,
  DeleteOutline as RemoveIcon,
  Key as CredentialsIcon,
  Refresh as RefreshIcon,
} from '@mui/icons-material';
import { useAtom } from 'jotai';
import { useTranslation } from 'react-i18next';
import { backend, type InstalledInfo, type SourceInfo } from '../../backend';
import { useSiteModules } from '../../hooks/useSiteModules';
import { settingsAtom } from '../../stores/settingsStore';
import { useInstall } from './ConsentDialog';
import CredentialsDialog from './CredentialsDialog';

export default function ModulesSettings() {
  const { t } = useTranslation();
  const { installed, sources, error: loadError, loaded } = useSiteModules();
  const [tab, setTab] = useState(0);
  const [error, setError] = useState<string | null>(null);
  const { install, busy, dialog } = useInstall(setError);

  /** Run a command; a failure shows above the tabs. */
  const run = (action: () => Promise<unknown>) => () => {
    setError(null);
    action().catch((e) => setError((e as Error).message));
  };

  if (!loaded) {
    return (
      <Box display="flex" justifyContent="center" p={4}>
        <CircularProgress />
      </Box>
    );
  }

  return (
    <Box>
      {(error ?? loadError) && (
        <Alert severity="error" onClose={() => setError(null)} sx={{ mb: 2 }}>
          {error ?? loadError}
        </Alert>
      )}
      <Tabs value={tab} onChange={(_, next) => setTab(next)} sx={{ mb: 2 }}>
        <Tab label={t('modules.installed.title')} />
        <Tab label={t('modules.sources.title')} />
        <Tab label={t('modules.browse.title')} />
      </Tabs>
      {tab === 0 && <Installed modules={installed} run={run} install={install} busy={busy} />}
      {tab === 1 && <Sources sources={sources} run={run} />}
      {tab === 2 && <Browse sources={sources} installed={installed} install={install} busy={busy} />}
      {dialog}
    </Box>
  );
}

type Run = (action: () => Promise<unknown>) => () => void;

function Installed({
  modules,
  run,
  install,
  busy,
}: {
  modules: InstalledInfo[];
  run: Run;
  install: (key: string) => void;
  busy: string | null;
}) {
  const { t } = useTranslation();
  const [settings, setSettings] = useAtom(settingsAtom);
  const [credentials, setCredentials] = useState<InstalledInfo | null>(null);
  const [removing, setRemoving] = useState<InstalledInfo | null>(null);
  const editing = modules.find((m) => m.key === credentials?.key) ?? null;

  const toggleUpdateCheck = run(async () =>
    setSettings(await backend.settings.update({ ...settings, checkModuleUpdates: !settings.checkModuleUpdates }))
  );

  return (
    <Stack spacing={1.5}>
      <FormControlLabel
        control={<Switch checked={settings.checkModuleUpdates} onChange={toggleUpdateCheck} />}
        label={t('modules.installed.checkOnStart')}
      />
      {modules.length === 0 && (
        <Typography color="text.secondary" sx={{ p: 2 }}>
          {t('modules.installed.empty')}
        </Typography>
      )}
      {modules.map((m, index) => {
        const missing = m.credentials.filter((c) => !c.optional && !c.set);
        return (
          <Paper key={m.key} variant="outlined" sx={{ p: 2, opacity: m.enabled ? 1 : 0.6 }}>
            <Box sx={{ display: 'flex', alignItems: 'center', gap: 1 }}>
              <Box sx={{ flex: 1, minWidth: 0 }}>
                <Typography variant="subtitle1" noWrap>
                  {m.name}{' '}
                  <Typography component="span" variant="body2" color="text.secondary">
                    {m.version} · {m.key}
                  </Typography>
                </Typography>
                <Typography variant="body2" color="text.secondary" noWrap title={m.urlPatterns.join('\n')}>
                  {m.urlPatterns.join('  ')}
                </Typography>
              </Box>
              {m.trust === 'official' && <Chip size="small" label={t('modules.trust.official')} color="primary" variant="outlined" />}
              {m.local && <Chip size="small" label={t('modules.installed.local')} variant="outlined" />}
              {m.update && (
                <Button size="small" variant="contained" disabled={busy === m.key} onClick={() => install(m.key)}>
                  {t('modules.installed.update', { version: m.update })}
                </Button>
              )}
              <Tooltip title={t('modules.installed.moveUp')}>
                <span>
                  <IconButton size="small" disabled={index === 0} onClick={run(() => backend.modules.move(m.key, index - 1))}>
                    <UpIcon fontSize="small" />
                  </IconButton>
                </span>
              </Tooltip>
              <Tooltip title={t('modules.installed.moveDown')}>
                <span>
                  <IconButton
                    size="small"
                    disabled={index === modules.length - 1}
                    onClick={run(() => backend.modules.move(m.key, index + 1))}
                  >
                    <DownIcon fontSize="small" />
                  </IconButton>
                </span>
              </Tooltip>
              {m.credentials.length > 0 && (
                <Tooltip title={t('modules.credentials.title')}>
                  <IconButton size="small" onClick={() => setCredentials(m)} color={missing.length > 0 ? 'warning' : 'default'}>
                    <CredentialsIcon fontSize="small" />
                  </IconButton>
                </Tooltip>
              )}
              <Tooltip title={t('modules.installed.remove')}>
                <IconButton size="small" onClick={() => setRemoving(m)}>
                  <RemoveIcon fontSize="small" />
                </IconButton>
              </Tooltip>
              <Switch checked={m.enabled} onChange={run(() => backend.modules.setEnabled(m.key, !m.enabled))} />
            </Box>
            {m.needsConsent && (
              <Alert
                severity="warning"
                sx={{ mt: 1 }}
                action={
                  <Button color="inherit" size="small" onClick={() => install(m.key)}>
                    {t('modules.installed.review')}
                  </Button>
                }
              >
                {t('modules.installed.needsConsent')}
              </Alert>
            )}
            {missing.length > 0 && (
              <Typography variant="body2" color="warning.main" sx={{ mt: 1 }}>
                {t('modules.installed.missingCredentials', { fields: missing.map((c) => c.label).join(', ') })}
              </Typography>
            )}
          </Paper>
        );
      })}
      <CredentialsDialog module={editing} onClose={() => setCredentials(null)} />
      {removing && (
        <Dialog open onClose={() => setRemoving(null)}>
          <DialogContent>
            <DialogContentText>{t('modules.installed.confirmRemove', { name: removing.name })}</DialogContentText>
          </DialogContent>
          <DialogActions>
            <Button onClick={() => setRemoving(null)}>{t('common.cancel')}</Button>
            <Button
              color="error"
              onClick={() => {
                const key = removing.key;
                setRemoving(null);
                run(() => backend.modules.remove(key))();
              }}
            >
              {t('modules.installed.remove')}
            </Button>
          </DialogActions>
        </Dialog>
      )}
    </Stack>
  );
}

function Sources({ sources, run }: { sources: SourceInfo[]; run: Run }) {
  const { t } = useTranslation();
  const [location, setLocation] = useState('');
  const [adding, setAdding] = useState(false);
  const [refreshing, setRefreshing] = useState(false);

  const add = (e: React.FormEvent) => {
    e.preventDefault();
    setAdding(true);
    run(async () => {
      try {
        await backend.modules.addSource(location.trim());
        setLocation('');
      } finally {
        setAdding(false);
      }
    })();
  };

  const refresh = () => {
    setRefreshing(true);
    run(() => backend.modules.refresh().finally(() => setRefreshing(false)))();
  };

  return (
    <Stack spacing={1.5}>
      <Box component="form" onSubmit={add} sx={{ display: 'flex', gap: 1, alignItems: 'flex-start' }}>
        <TextField
          value={location}
          onChange={(e) => setLocation(e.target.value)}
          label={t('modules.sources.add')}
          placeholder="owner/repo"
          helperText={t('modules.sources.addHelp')}
          size="small"
          fullWidth
        />
        <Button type="submit" variant="contained" disabled={adding || !location.trim()} sx={{ mt: 0.25 }}>
          {adding ? <CircularProgress size={20} /> : t('common.add')}
        </Button>
        <Tooltip title={t('modules.sources.refresh')}>
          <span>
            <IconButton onClick={refresh} disabled={refreshing}>
              {refreshing ? <CircularProgress size={20} /> : <RefreshIcon />}
            </IconButton>
          </span>
        </Tooltip>
      </Box>
      {sources.map((s) => (
        <Paper key={s.name} variant="outlined" sx={{ p: 2 }}>
          <Box sx={{ display: 'flex', alignItems: 'center', gap: 1 }}>
            <Box sx={{ flex: 1, minWidth: 0 }}>
              <Typography variant="subtitle1">
                {s.name}{' '}
                {s.owner && (
                  <Typography component="span" variant="body2" color="text.secondary">
                    {s.owner}
                  </Typography>
                )}
              </Typography>
              <Typography variant="body2" color="text.secondary" noWrap title={s.location}>
                {s.location}
              </Typography>
            </Box>
            <Chip size="small" label={t('modules.sources.count', { count: s.modules.length })} variant="outlined" />
            {s.trust === 'official' ? (
              <Chip size="small" label={t('modules.trust.official')} color="primary" variant="outlined" />
            ) : (
              <Tooltip title={t('modules.sources.remove')}>
                <IconButton size="small" onClick={run(() => backend.modules.removeSource(s.name))}>
                  <RemoveIcon fontSize="small" />
                </IconButton>
              </Tooltip>
            )}
          </Box>
          {s.error && (
            <Alert severity="warning" sx={{ mt: 1 }}>
              {s.error}
            </Alert>
          )}
        </Paper>
      ))}
    </Stack>
  );
}

function Browse({
  sources,
  installed,
  install,
  busy,
}: {
  sources: SourceInfo[];
  installed: InstalledInfo[];
  install: (key: string) => void;
  busy: string | null;
}) {
  const { t } = useTranslation();
  const offered = sources.flatMap((s) => s.modules.map((m) => ({ source: s, module: m, key: `${m.id}@${s.name}` })));

  if (offered.length === 0) {
    return <Typography color="text.secondary" sx={{ p: 2 }}>{t('modules.browse.empty')}</Typography>;
  }

  return (
    <Stack spacing={1.5}>
      {offered.map(({ source, module, key }) => {
        const current = installed.find((m) => m.key === key);
        return (
          <Paper key={key} variant="outlined" sx={{ p: 2, display: 'flex', alignItems: 'center', gap: 1 }}>
            <Box sx={{ flex: 1, minWidth: 0 }}>
              <Typography variant="subtitle1">
                {module.id}{' '}
                <Typography component="span" variant="body2" color="text.secondary">
                  {module.version} · {source.name}
                </Typography>
              </Typography>
              {module.description && (
                <Typography variant="body2" color="text.secondary">
                  {module.description}
                </Typography>
              )}
            </Box>
            {source.trust === 'official' && <Chip size="small" label={t('modules.trust.official')} color="primary" variant="outlined" />}
            {current && !current.update ? (
              <Chip size="small" label={t('modules.browse.installed')} />
            ) : (
              <Button variant="contained" size="small" disabled={busy === key} onClick={() => install(key)}>
                {busy === key ? (
                  <CircularProgress size={18} />
                ) : current ? (
                  t('modules.installed.update', { version: module.version })
                ) : (
                  t('modules.browse.install')
                )}
              </Button>
            )}
          </Paper>
        );
      })}
    </Stack>
  );
}
