// A short message that pops up over the page: it never pushes the layout around.
import { useRef } from 'react';
import { Alert, Snackbar, type AlertColor } from '@mui/material';

export type ToastMessage = { severity: AlertColor; text: string };

export default function Toast({ message, onClose }: { message: ToastMessage | null; onClose: () => void }) {
  // Keep the last text while it fades out.
  const last = useRef(message);
  if (message) last.current = message;
  const shown = last.current;
  return (
    <Snackbar
      open={message !== null}
      autoHideDuration={message?.severity === 'error' ? 8000 : 3000}
      onClose={(_, reason) => reason !== 'clickaway' && onClose()}
      anchorOrigin={{ vertical: 'bottom', horizontal: 'center' }}
    >
      <Alert variant="filled" severity={shown?.severity ?? 'info'} onClose={onClose}>
        {shown?.text}
      </Alert>
    </Snackbar>
  );
}
