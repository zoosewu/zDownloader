// A dialog's error, on the left of its buttons: the row is there anyway, so the dialog keeps
// its size and nothing above moves when the error shows up.
import { Typography } from '@mui/material';

export default function DialogError({ error }: { error: string | null | undefined }) {
  return (
    <Typography variant="body2" color="error" role={error ? 'alert' : undefined} sx={{ flex: 1, minWidth: 0, mr: 1, pl: 1 }}>
      {error}
    </Typography>
  );
}
