// User-Agent, Referer and Cookie for a direct download.
import { TextField } from '@mui/material';
import { useTranslation } from 'react-i18next';
import type { HeaderFields } from '../utils/headers';
import UserAgentField from './UserAgentField';

interface HeaderFieldsEditorProps {
  value: HeaderFields;
  onChange: (value: HeaderFields) => void;
}

export default function HeaderFieldsEditor({ value, onChange }: HeaderFieldsEditorProps) {
  const { t } = useTranslation();
  const set = (key: keyof HeaderFields) => (next: string) => onChange({ ...value, [key]: next });
  return (
    <>
      <UserAgentField value={value.userAgent} onChange={set('userAgent')} placeholder={t('headers.userAgentDefault')} />
      <TextField
        label={t('headers.referer')}
        value={value.referer}
        onChange={(e) => set('referer')(e.target.value)}
        placeholder="https://example.com/page"
        fullWidth
        margin="normal"
      />
      <TextField
        label={t('headers.cookie')}
        value={value.cookie}
        onChange={(e) => set('cookie')(e.target.value)}
        placeholder="name=value; other=value"
        helperText={t('headers.cookieStored')}
        fullWidth
        margin="normal"
      />
    </>
  );
}
