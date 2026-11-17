// A component that throws while rendering takes only its part of the window down: the user
// sees what broke and can reload, instead of a blank window. The error also goes to the log.
import React from 'react';
import { Alert, AlertTitle, Box, Button, Typography } from '@mui/material';
import i18n from '../i18n';

interface State {
  error: Error | null;
}

export default class ErrorBoundary extends React.Component<{ children: React.ReactNode }, State> {
  state: State = { error: null };

  static getDerivedStateFromError(error: Error): State {
    return { error };
  }

  componentDidCatch(error: Error, info: React.ErrorInfo) {
    console.error('[ui] render failed:', error, info.componentStack);
  }

  render() {
    const { error } = this.state;
    if (!error) return this.props.children;
    return (
      <Box sx={{ p: 3, display: 'flex', justifyContent: 'center' }}>
        <Alert
          severity="error"
          sx={{ maxWidth: 640, width: '100%' }}
          action={
            <Button color="inherit" onClick={() => window.location.reload()}>
              {i18n.t('errorBoundary.reload')}
            </Button>
          }
        >
          <AlertTitle>{i18n.t('errorBoundary.title')}</AlertTitle>
          {i18n.t('errorBoundary.message')}
          <Typography component="pre" variant="caption" sx={{ mt: 1, whiteSpace: 'pre-wrap', wordBreak: 'break-word', userSelect: 'text' }}>
            {error.message}
          </Typography>
        </Alert>
      </Box>
    );
  }
}
