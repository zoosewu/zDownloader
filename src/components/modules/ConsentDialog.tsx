// Installing or updating a module: show what it may access and install only once accepted.
// Updates that ask for nothing new install without asking.
import { useState } from 'react';
import {
  Alert,
  Box,
  Button,
  Chip,
  Dialog,
  DialogActions,
  DialogContent,
  DialogTitle,
  List,
  ListItem,
  ListItemText,
  Typography,
} from '@mui/material';
import { useTranslation } from 'react-i18next';
import { backend, type PlanSummary } from '../../backend';
import { localized } from '../../utils/localized';

/** `install(key)` plans, asks if needed, installs. Render `dialog` somewhere. */
export function useInstall(onError: (message: string) => void) {
  const [plan, setPlan] = useState<PlanSummary | null>(null);
  const [busy, setBusy] = useState<string | null>(null);

  const install = async (key: string) => {
    setBusy(key);
    try {
      const summary = await backend.modules.planInstall(key);
      if (summary.needsConsent) setPlan(summary);
      else await backend.modules.install(key);
    } catch (e) {
      onError((e as Error).message);
    } finally {
      setBusy(null);
    }
  };

  const close = (accept: boolean) => async () => {
    if (!plan) return;
    const key = plan.key;
    setPlan(null);
    try {
      await (accept ? backend.modules.install(key) : backend.modules.cancelInstall(key));
    } catch (e) {
      onError((e as Error).message);
    }
  };

  const dialog = plan && <ConsentDialog plan={plan} onAccept={close(true)} onDecline={close(false)} />;
  return { install, busy, dialog };
}

interface ConsentDialogProps {
  plan: PlanSummary;
  onAccept: () => void;
  onDecline: () => void;
}

function ConsentDialog({ plan, onAccept, onDecline }: ConsentDialogProps) {
  const { t, i18n } = useTranslation();
  // On an update, what was not granted before stands out.
  const isNewDomain = (domain: string) => plan.currentVersion !== null && plan.added.allowDomains.includes(domain);
  const isNewKey = (key: string) => plan.currentVersion !== null && plan.added.credentials.some((c) => c.key === key);

  return (
    <Dialog open onClose={onDecline} maxWidth="sm" fullWidth>
      <DialogTitle>
        {plan.name}{' '}
        <Typography component="span" color="text.secondary">
          {plan.currentVersion ? `${plan.currentVersion} → ${plan.version}` : plan.version}
        </Typography>
      </DialogTitle>
      <DialogContent>
        {plan.trust === 'community' && (
          <Alert severity="warning" sx={{ mb: 2 }}>
            {t('modules.consent.community')}
          </Alert>
        )}
        <Typography variant="subtitle2">{t('modules.consent.domains')}</Typography>
        <Box sx={{ display: 'flex', flexWrap: 'wrap', gap: 0.5, my: 1 }}>
          {plan.permissions.allowDomains.map((domain) => (
            <Chip
              key={domain}
              label={domain}
              size="small"
              color={isNewDomain(domain) ? 'warning' : 'default'}
              variant={isNewDomain(domain) ? 'filled' : 'outlined'}
            />
          ))}
        </Box>
        {plan.permissions.credentials.length > 0 && (
          <>
            <Typography variant="subtitle2" sx={{ mt: 2 }}>
              {t('modules.consent.credentials')}
            </Typography>
            <List dense>
              {plan.permissions.credentials.map((c) => (
                <ListItem key={c.key} disableGutters>
                  <ListItemText
                    primary={localized(c.label, i18n.language)}
                    secondary={c.optional ? t('modules.credentials.optional') : undefined}
                    slotProps={{ primary: { color: isNewKey(c.key) ? 'warning.main' : undefined } }}
                  />
                </ListItem>
              ))}
            </List>
          </>
        )}
      </DialogContent>
      <DialogActions>
        <Button onClick={onDecline}>{t('common.cancel')}</Button>
        <Button variant="contained" onClick={onAccept}>
          {plan.currentVersion ? t('modules.consent.acceptUpdate') : t('modules.consent.accept')}
        </Button>
      </DialogActions>
    </Dialog>
  );
}
