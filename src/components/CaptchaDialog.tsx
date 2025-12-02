// CAPTCHA dialog component for user verification
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
} from '@mui/material';
import { useAtom } from 'jotai';
import { invoke } from '../utils/tauri';
import {
  currentCaptchaAtom,
  showCaptchaDialogAtom,
} from '../stores/captchaStore';

export const CaptchaDialog: React.FC = () => {
  const [currentCaptcha, setCurrentCaptcha] = useAtom(currentCaptchaAtom);
  const [showDialog, setShowDialog] = useAtom(showCaptchaDialogAtom);
  const [solution, setSolution] = useState('');
  const [isSubmitting, setIsSubmitting] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const handleClose = async () => {
    if (currentCaptcha && !isSubmitting) {
      try {
        // Cancel the CAPTCHA challenge
        await invoke('cancel_captcha', {
          challengeId: currentCaptcha.id,
        });
      } catch (err) {
        console.error('Failed to cancel CAPTCHA:', err);
      }

      setCurrentCaptcha(null);
      setShowDialog(false);
      setSolution('');
      setError(null);
    }
  };

  const handleSubmit = async () => {
    if (!currentCaptcha || !solution.trim()) {
      setError('Please enter a solution');
      return;
    }

    setIsSubmitting(true);
    setError(null);

    try {
      // Submit the CAPTCHA solution
      await invoke('submit_captcha_solution', {
        challengeId: currentCaptcha.id,
        solutionText: solution.trim(),
        metadata: null,
      });

      // Close dialog on success
      setCurrentCaptcha(null);
      setShowDialog(false);
      setSolution('');
    } catch (err) {
      console.error('Failed to submit CAPTCHA solution:', err);
      setError(
        err instanceof Error ? err.message : 'Failed to submit solution'
      );
    } finally {
      setIsSubmitting(false);
    }
  };

  const handleKeyPress = (event: React.KeyboardEvent) => {
    if (event.key === 'Enter' && !isSubmitting) {
      handleSubmit();
    }
  };

  if (!currentCaptcha) {
    return null;
  }

  return (
    <Dialog
      open={showDialog}
      onClose={handleClose}
      maxWidth="sm"
      fullWidth
      disableEscapeKeyDown={isSubmitting}
    >
      <DialogTitle>CAPTCHA Verification Required</DialogTitle>
      <DialogContent>
        <Box sx={{ mb: 2 }}>
          <Typography variant="body2" color="text.secondary" gutterBottom>
            Module: {currentCaptcha.module_id}
          </Typography>
          <Typography variant="body2" color="text.secondary" gutterBottom>
            Download Task: {currentCaptcha.task_id}
          </Typography>
          <Typography variant="body2" color="text.secondary" gutterBottom>
            CAPTCHA Type: {currentCaptcha.captcha_type}
          </Typography>
        </Box>

        {/* Display CAPTCHA image if type is 'image' */}
        {currentCaptcha.captcha_type === 'image' && (
          <Box
            sx={{
              display: 'flex',
              justifyContent: 'center',
              mb: 2,
              p: 2,
              border: '1px solid',
              borderColor: 'divider',
              borderRadius: 1,
            }}
          >
            <img
              src={currentCaptcha.challenge_data}
              alt="CAPTCHA"
              style={{
                maxWidth: '100%',
                maxHeight: '200px',
                display: 'block',
              }}
              onError={(e) => {
                console.error('Failed to load CAPTCHA image');
                e.currentTarget.style.display = 'none';
              }}
            />
          </Box>
        )}

        {/* Display challenge data for other types */}
        {currentCaptcha.captcha_type !== 'image' && (
          <Box
            sx={{
              mb: 2,
              p: 2,
              bgcolor: 'grey.100',
              borderRadius: 1,
            }}
          >
            <Typography variant="body2" sx={{ wordBreak: 'break-all' }}>
              {currentCaptcha.challenge_data}
            </Typography>
          </Box>
        )}

        <TextField
          autoFocus
          margin="dense"
          label="Enter CAPTCHA Solution"
          type="text"
          fullWidth
          variant="outlined"
          value={solution}
          onChange={(e) => setSolution(e.target.value)}
          onKeyPress={handleKeyPress}
          disabled={isSubmitting}
          error={!!error}
          helperText={error}
          placeholder="Type the characters you see above"
        />

        {/* Display additional parameters if any */}
        {Object.keys(currentCaptcha.params).length > 0 && (
          <Box sx={{ mt: 2 }}>
            <Typography variant="caption" color="text.secondary">
              Additional Information:
            </Typography>
            {Object.entries(currentCaptcha.params).map(([key, value]) => (
              <Typography
                key={key}
                variant="caption"
                display="block"
                color="text.secondary"
              >
                {key}: {value}
              </Typography>
            ))}
          </Box>
        )}
      </DialogContent>

      <DialogActions>
        <Button onClick={handleClose} disabled={isSubmitting}>
          Cancel
        </Button>
        <Button
          onClick={handleSubmit}
          variant="contained"
          disabled={isSubmitting || !solution.trim()}
          startIcon={isSubmitting ? <CircularProgress size={16} /> : null}
        >
          {isSubmitting ? 'Submitting...' : 'Submit'}
        </Button>
      </DialogActions>
    </Dialog>
  );
};

export default CaptchaDialog;
