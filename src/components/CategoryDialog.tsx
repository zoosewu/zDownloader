// Create or edit a category: a name, "all / any rules match", and the rules. Rules tag
// downloads when they are added; existing downloads keep their categories.
import React, { useEffect, useState } from 'react';
import {
  Alert,
  Box,
  Button,
  Dialog,
  DialogActions,
  DialogContent,
  DialogTitle,
  IconButton,
  MenuItem,
  Stack,
  TextField,
  ToggleButton,
  ToggleButtonGroup,
  Tooltip,
  Typography,
} from '@mui/material';
import { Add as AddIcon, Close as RemoveIcon } from '@mui/icons-material';
import { useTranslation } from 'react-i18next';
import type { Category, Matching, Rule, Target } from '../backend';

type Kind = Rule['kind'];
const KINDS: Kind[] = ['extension', 'host', 'contains', 'regex'];

/** Rule as edited: list values are typed as one comma-separated string. */
interface Draft {
  kind: Kind;
  target: Target;
  text: string;
}

const toDraft = (rule: Rule): Draft => {
  switch (rule.kind) {
    case 'extension':
    case 'host':
      return { kind: rule.kind, target: 'filename', text: rule.values.join(', ') };
    case 'contains':
      return { kind: rule.kind, target: rule.target, text: rule.text };
    case 'regex':
      return { kind: rule.kind, target: rule.target, text: rule.pattern };
  }
};

const toRule = ({ kind, target, text }: Draft): Rule => {
  const values = text.split(/[,\s]+/).filter(Boolean);
  switch (kind) {
    case 'extension':
    case 'host':
      return { kind, values };
    case 'contains':
      return { kind, target, text: text.trim() };
    case 'regex':
      return { kind, target, pattern: text };
  }
};

interface Props {
  /** `null`: closed. A category without an id is a new one. */
  category: Category | null;
  onClose: () => void;
  /** Throws the backend's message for an invalid rule. */
  onSave: (category: Category) => Promise<void>;
}

const CategoryDialog: React.FC<Props> = ({ category, onClose, onSave }) => {
  const { t } = useTranslation();
  const [name, setName] = useState('');
  const [matching, setMatching] = useState<Matching>('any');
  const [rules, setRules] = useState<Draft[]>([]);
  const [error, setError] = useState<string | null>(null);
  const [saving, setSaving] = useState(false);

  useEffect(() => {
    if (!category) return;
    setName(category.name);
    setMatching(category.match);
    setRules(category.rules.map(toDraft));
    setError(null);
  }, [category]);

  const change = (index: number, patch: Partial<Draft>) =>
    setRules((all) => all.map((r, i) => (i === index ? { ...r, ...patch } : r)));

  const save = async () => {
    setSaving(true);
    setError(null);
    try {
      await onSave({
        id: category?.id || crypto.randomUUID(),
        name: name.trim(),
        match: matching,
        rules: rules.filter((r) => r.text.trim()).map(toRule),
      });
      onClose();
    } catch (e) {
      setError(e instanceof Error ? e.message : String(e));
    } finally {
      setSaving(false);
    }
  };

  const placeholder = (kind: Kind) => t(`categories.placeholder.${kind}`);

  return (
    <Dialog open={!!category} onClose={onClose} maxWidth="sm" fullWidth>
      <DialogTitle>{category?.id ? t('categories.edit') : t('categories.add')}</DialogTitle>
      <DialogContent>
        <Stack spacing={1.5} sx={{ pt: 1 }}>
          <TextField label={t('categories.name')} value={name} onChange={(e) => setName(e.target.value)} autoFocus />

          <Box sx={{ display: 'flex', alignItems: 'center', gap: 1 }}>
            <Typography variant="body2" color="text.secondary">
              {t('categories.rules')}
            </Typography>
            <ToggleButtonGroup exclusive color="primary" value={matching} onChange={(_, v: Matching | null) => v && setMatching(v)}>
              <ToggleButton value="all" sx={{ py: 0.25, px: 1, textTransform: 'none' }}>
                {t('categories.matchAll')}
              </ToggleButton>
              <ToggleButton value="any" sx={{ py: 0.25, px: 1, textTransform: 'none' }}>
                {t('categories.matchAny')}
              </ToggleButton>
            </ToggleButtonGroup>
          </Box>

          {rules.map((rule, index) => (
            <Box key={index} sx={{ display: 'flex', gap: 0.75, alignItems: 'center' }}>
              <TextField
                select
                value={rule.kind}
                onChange={(e) => change(index, { kind: e.target.value as Kind })}
                sx={{ width: 130, flexShrink: 0 }}
              >
                {KINDS.map((k) => (
                  <MenuItem key={k} value={k}>
                    {t(`categories.kind.${k}`)}
                  </MenuItem>
                ))}
              </TextField>
              {(rule.kind === 'contains' || rule.kind === 'regex') && (
                <TextField
                  select
                  value={rule.target}
                  onChange={(e) => change(index, { target: e.target.value as Target })}
                  sx={{ width: 90, flexShrink: 0 }}
                >
                  <MenuItem value="filename">{t('categories.target.filename')}</MenuItem>
                  <MenuItem value="url">{t('categories.target.url')}</MenuItem>
                </TextField>
              )}
              <TextField
                value={rule.text}
                placeholder={placeholder(rule.kind)}
                onChange={(e) => change(index, { text: e.target.value })}
                sx={{ flex: 1 }}
                slotProps={{ htmlInput: { spellCheck: false, style: rule.kind === 'regex' ? { fontFamily: 'Consolas, monospace' } : undefined } }}
              />
              <Tooltip title={t('categories.removeRule')}>
                <IconButton onClick={() => setRules((all) => all.filter((_, i) => i !== index))}>
                  <RemoveIcon fontSize="small" />
                </IconButton>
              </Tooltip>
            </Box>
          ))}
          <Box>
            <Button
              color="inherit"
              startIcon={<AddIcon fontSize="small" />}
              onClick={() => setRules((all) => [...all, { kind: 'extension', target: 'filename', text: '' }])}
            >
              {t('categories.addRule')}
            </Button>
          </Box>
          <Typography variant="caption" color="text.secondary">
            {t('categories.help')}
          </Typography>
          {error && <Alert severity="error">{error}</Alert>}
        </Stack>
      </DialogContent>
      <DialogActions>
        <Button onClick={onClose} color="inherit">
          {t('common.cancel')}
        </Button>
        <Button variant="contained" onClick={save} disabled={saving || !name.trim()}>
          {t('common.save')}
        </Button>
      </DialogActions>
    </Dialog>
  );
};

export default CategoryDialog;
