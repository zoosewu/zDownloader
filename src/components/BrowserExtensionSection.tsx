// Settings: installing the browser extension, from the Edge Add-ons page (in either browser), or
// by loading the folder the app ships, in two steps, each with the button that does it.
import React, { useEffect, useState } from 'react';
import { Alert, Box, Button, Stack, Typography } from '@mui/material';
import { FolderOpen as FolderIcon, OpenInNew as OpenIcon, Storefront as StoreIcon } from '@mui/icons-material';
import { useTranslation } from 'react-i18next';
import { backend, type ExtensionSetup } from '../backend';

const BrowserExtensionSection: React.FC = () => {
  const { t } = useTranslation();
  const [setup, setSetup] = useState<ExtensionSetup | null>(null);
  const [error, setError] = useState<string | null>(null);
  /** The extensions page address just put on the clipboard. */
  const [copied, setCopied] = useState<string | null>(null);
  const fail = (e: unknown) => setError(e instanceof Error ? e.message : String(e));

  useEffect(() => {
    backend.extension.setup().then(setSetup).catch(fail);
  }, []);

  const row = (text: string, actions: React.ReactNode) => (
    <Box sx={{ display: 'flex', alignItems: 'center', gap: 1, flexWrap: 'wrap' }}>
      <Typography variant="body2" sx={{ minWidth: 0 }}>
        {text}
      </Typography>
      {actions}
    </Box>
  );
  const step = (n: number, text: string, actions: React.ReactNode) => row(`${n}. ${text}`, actions);
  const openStore = (browser?: string) => backend.extension.openStore(browser).catch(fail);

  return (
    <Stack spacing={1}>
      {error && <Alert severity="error">{error}</Alert>}
      {row(
        t('settings.extension.store'),
        setup?.browsers.length
          ? setup.browsers.map((b) => (
              <Button key={b.id} color="inherit" startIcon={<StoreIcon fontSize="small" />} onClick={() => openStore(b.id)}>
                {b.name}
              </Button>
            ))
          : setup && (
              <Button color="inherit" startIcon={<StoreIcon fontSize="small" />} onClick={() => openStore()}>
                {t('settings.extension.openStore')}
              </Button>
            ),
      )}
      <Typography variant="subtitle2" sx={{ pt: 1 }}>
        {t('settings.extension.manual')}
      </Typography>
      {setup && setup.browsers.length === 0 && <Alert severity="info">{t('settings.extension.noBrowser')}</Alert>}
      {step(
        1,
        t('settings.extension.step1'),
        setup?.browsers.map((b) => (
          <Button key={b.id} color="inherit" startIcon={<OpenIcon fontSize="small" />} onClick={() => backend.extension.openPage(b.id).then(setCopied, fail)}>
            {b.name}
          </Button>
        )),
      )}
      {copied && <Alert severity="info">{t('settings.extension.pasteAddress', { address: copied })}</Alert>}
      {step(
        2,
        t('settings.extension.step2'),
        <Button color="inherit" startIcon={<FolderIcon fontSize="small" />} onClick={() => backend.extension.showFolder().catch(fail)}>
          {t('settings.extension.showFolder')}
        </Button>,
      )}
      {setup && (
        <Typography variant="caption" color="text.secondary" sx={{ fontFamily: 'monospace', userSelect: 'text', wordBreak: 'break-all' }}>
          {setup.folder}
        </Typography>
      )}
      <Typography variant="caption" color="text.secondary">
        {t('settings.extension.afterUpdate')}
      </Typography>
    </Stack>
  );
};

export default BrowserExtensionSection;
