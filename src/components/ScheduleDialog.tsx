// Schedule Dialog Component - Dialog for scheduling downloads
import {
  Button,
  Dialog,
  DialogActions,
  DialogContent,
  DialogTitle,
  Stack,
  TextField,
  Alert,
} from '@mui/material';
import { useState } from 'react';
import { useTranslation } from 'react-i18next';
import { scheduleDownload } from '../api/scheduler';
import { DownloadTask } from '../types/download';

interface ScheduleDialogProps {
  open: boolean;
  task: DownloadTask | null;
  onClose: () => void;
  onScheduled?: () => void;
}

export default function ScheduleDialog({ open, task, onClose, onScheduled }: ScheduleDialogProps) {
  const { t } = useTranslation();
  const [scheduledDate, setScheduledDate] = useState('');
  const [scheduledTime, setScheduledTime] = useState('');
  const [error, setError] = useState<string | null>(null);
  const [loading, setLoading] = useState(false);

  // Get current datetime for min attribute
  const now = new Date();
  const minDate = now.toISOString().split('T')[0];

  const handleSchedule = async () => {
    if (!task || !scheduledDate || !scheduledTime) {
      setError(t('scheduler.errors.invalidDateTime') || 'Please select date and time');
      return;
    }

    try {
      setLoading(true);
      setError(null);

      // Combine date and time into ISO 8601 format
      const scheduledDateTime = new Date(`${scheduledDate}T${scheduledTime}`);

      // Check if time is in the future
      if (scheduledDateTime <= now) {
        setError(t('scheduler.errors.pastTime') || 'Scheduled time must be in the future');
        return;
      }

      const isoDateTime = scheduledDateTime.toISOString();

      await scheduleDownload(task.id, isoDateTime);

      if (onScheduled) {
        onScheduled();
      }

      handleClose();
    } catch (err) {
      console.error('Failed to schedule download:', err);
      setError(t('scheduler.errors.scheduleFailed') || 'Failed to schedule download');
    } finally {
      setLoading(false);
    }
  };

  const handleClose = () => {
    setScheduledDate('');
    setScheduledTime('');
    setError(null);
    onClose();
  };

  return (
    <Dialog open={open} onClose={handleClose} maxWidth="sm" fullWidth>
      <DialogTitle>{t('scheduler.scheduleDownload') || 'Schedule Download'}</DialogTitle>
      <DialogContent>
        <Stack spacing={3} sx={{ mt: 1 }}>
          {error && (
            <Alert severity="error" onClose={() => setError(null)}>
              {error}
            </Alert>
          )}

          {task && (
            <TextField
              fullWidth
              label={t('downloads.filenameLabel') || 'Filename'}
              value={task.filename}
              InputProps={{ readOnly: true }}
              size="small"
            />
          )}

          <TextField
            fullWidth
            type="date"
            label={t('scheduler.selectDate') || 'Select Date'}
            value={scheduledDate}
            onChange={(e) => setScheduledDate(e.target.value)}
            InputLabelProps={{ shrink: true }}
            inputProps={{ min: minDate }}
          />

          <TextField
            fullWidth
            type="time"
            label={t('scheduler.selectTime') || 'Select Time'}
            value={scheduledTime}
            onChange={(e) => setScheduledTime(e.target.value)}
            InputLabelProps={{ shrink: true }}
            inputProps={{
              step: 300, // 5 minutes
            }}
          />

          <Alert severity="info">
            {t('scheduler.scheduleHint') ||
              'The download will automatically start at the specified time.'}
          </Alert>
        </Stack>
      </DialogContent>
      <DialogActions>
        <Button onClick={handleClose} disabled={loading}>
          {t('common.cancel')}
        </Button>
        <Button
          onClick={handleSchedule}
          variant="contained"
          disabled={loading || !scheduledDate || !scheduledTime}
        >
          {loading ? t('scheduler.scheduling') || 'Scheduling...' : t('scheduler.schedule') || 'Schedule'}
        </Button>
      </DialogActions>
    </Dialog>
  );
}
