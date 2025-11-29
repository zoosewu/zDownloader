// Login dialog component for module authentication
import React, { useState } from 'react';
import {
  Dialog,
  DialogTitle,
  DialogContent,
  DialogActions,
  TextField,
  Button,
  Typography,
  Box,
  CircularProgress,
  FormControlLabel,
  Checkbox,
} from '@mui/material';
import { useAtom } from 'jotai';
import {
  showLoginDialogAtom,
  authModuleIdAtom,
  moduleCredentialsAtom,
} from '../stores/captchaStore';

interface LoginDialogProps {
  onSubmit?: (username: string, password: string, remember: boolean) => Promise<void>;
}

export const LoginDialog: React.FC<LoginDialogProps> = ({ onSubmit }) => {
  const [showDialog, setShowDialog] = useAtom(showLoginDialogAtom);
  const [moduleId, setModuleId] = useAtom(authModuleIdAtom);
  const [, setModuleCredentials] = useAtom(moduleCredentialsAtom);

  const [username, setUsername] = useState('');
  const [password, setPassword] = useState('');
  const [rememberCredentials, setRememberCredentials] = useState(false);
  const [isSubmitting, setIsSubmitting] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const handleClose = () => {
    if (!isSubmitting) {
      setShowDialog(false);
      setModuleId(null);
      setUsername('');
      setPassword('');
      setRememberCredentials(false);
      setError(null);
    }
  };

  const handleSubmit = async () => {
    if (!username.trim() || !password.trim()) {
      setError('Please enter both username and password');
      return;
    }

    if (!moduleId) {
      setError('Module ID not specified');
      return;
    }

    setIsSubmitting(true);
    setError(null);

    try {
      // Call the provided onSubmit callback if available
      if (onSubmit) {
        await onSubmit(username.trim(), password, rememberCredentials);
      }

      // Store credentials in memory if remember is checked
      if (rememberCredentials && moduleId) {
        setModuleCredentials((prev) => ({
          ...prev,
          [moduleId]: {
            username: username.trim(),
            password,
          },
        }));
      }

      // Close dialog on success
      handleClose();
    } catch (err) {
      console.error('Login failed:', err);
      setError(err instanceof Error ? err.message : 'Login failed');
    } finally {
      setIsSubmitting(false);
    }
  };

  const handleKeyPress = (event: React.KeyboardEvent) => {
    if (event.key === 'Enter' && !isSubmitting) {
      handleSubmit();
    }
  };

  return (
    <Dialog
      open={showDialog}
      onClose={handleClose}
      maxWidth="xs"
      fullWidth
      disableEscapeKeyDown={isSubmitting}
    >
      <DialogTitle>Login Required</DialogTitle>
      <DialogContent>
        <Box sx={{ mb: 2 }}>
          <Typography variant="body2" color="text.secondary" gutterBottom>
            This module requires authentication to continue.
          </Typography>
          {moduleId && (
            <Typography variant="body2" color="text.secondary" gutterBottom>
              Module: <strong>{moduleId}</strong>
            </Typography>
          )}
        </Box>

        <TextField
          autoFocus
          margin="dense"
          label="Username or Email"
          type="text"
          fullWidth
          variant="outlined"
          value={username}
          onChange={(e) => setUsername(e.target.value)}
          onKeyPress={handleKeyPress}
          disabled={isSubmitting}
          sx={{ mb: 2 }}
        />

        <TextField
          margin="dense"
          label="Password"
          type="password"
          fullWidth
          variant="outlined"
          value={password}
          onChange={(e) => setPassword(e.target.value)}
          onKeyPress={handleKeyPress}
          disabled={isSubmitting}
          error={!!error}
          helperText={error}
          sx={{ mb: 1 }}
        />

        <FormControlLabel
          control={
            <Checkbox
              checked={rememberCredentials}
              onChange={(e) => setRememberCredentials(e.target.checked)}
              disabled={isSubmitting}
            />
          }
          label={
            <Typography variant="body2">
              Remember credentials (stored securely in OS Keychain)
            </Typography>
          }
        />

        <Box sx={{ mt: 2 }}>
          <Typography variant="caption" color="text.secondary">
            Note: Credentials are stored securely using your operating system's
            keychain/credential manager.
          </Typography>
        </Box>
      </DialogContent>

      <DialogActions>
        <Button onClick={handleClose} disabled={isSubmitting}>
          Cancel
        </Button>
        <Button
          onClick={handleSubmit}
          variant="contained"
          disabled={isSubmitting || !username.trim() || !password.trim()}
          startIcon={isSubmitting ? <CircularProgress size={16} /> : null}
        >
          {isSubmitting ? 'Logging in...' : 'Login'}
        </Button>
      </DialogActions>
    </Dialog>
  );
};

export default LoginDialog;
