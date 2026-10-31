// Segment grid: which parts of the file are done, being fetched, or waiting to retry.
import React, { useEffect, useMemo, useState } from 'react';
import { Box, Tooltip, Typography, Stack } from '@mui/material';
import { useTranslation } from 'react-i18next';
import { backend, type SegmentMap, type Task } from '../backend';
import { buildSegmentCells, type CellStatus } from '../utils/segments';
import { formatBytes } from '../utils/downloadCalculations';

const POLL_MS = 500;

export const CELL_COLOR: Record<CellStatus, string> = {
  pending: '#9e9e9e',
  partial: '#a5d6a7',
  downloading: '#2196f3',
  retrying: '#f44336',
  completed: '#4caf50',
};

const LEGEND: CellStatus[] = ['completed', 'partial', 'downloading', 'retrying', 'pending'];

interface SegmentVisualizationProps {
  task: Task;
}

const SegmentVisualization: React.FC<SegmentVisualizationProps> = ({ task }) => {
  const { t } = useTranslation();
  const [map, setMap] = useState<SegmentMap | null>(null);
  const [error, setError] = useState<string | null>(null);

  // Poll while the task runs; otherwise one fetch per state change is enough.
  useEffect(() => {
    let cancelled = false;
    const load = async () => {
      try {
        const next = await backend.downloads.segments(task.id);
        if (cancelled) return;
        setMap(next);
        setError(null);
      } catch (e) {
        if (!cancelled) setError(e instanceof Error ? e.message : String(e));
      }
    };
    load();
    const timer = task.state === 'running' ? setInterval(load, POLL_MS) : undefined;
    return () => {
      cancelled = true;
      if (timer) clearInterval(timer);
    };
  }, [task.id, task.state, task.rev]);

  const cells = useMemo(() => (map ? buildSegmentCells(map) : []), [map]);

  if (error) {
    return (
      <Typography sx={{ p: 3 }} color="error">
        {error}
      </Typography>
    );
  }
  if (!map) return null;
  if (cells.length === 0) {
    return (
      <Typography sx={{ p: 3 }} color="text.secondary">
        {t('downloads.segments.unknownSize')}
      </Typography>
    );
  }

  return (
    <Box sx={{ p: 3 }}>
      <Stack direction="row" spacing={2} sx={{ mb: 2, flexWrap: 'wrap' }}>
        {LEGEND.map((status) => (
          <Stack key={status} direction="row" spacing={0.5} alignItems="center">
            <Box sx={{ width: 12, height: 12, borderRadius: '2px', backgroundColor: CELL_COLOR[status] }} />
            <Typography variant="caption">{t(`downloads.segments.${status}`)}</Typography>
          </Stack>
        ))}
      </Stack>

      <Box sx={{ display: 'grid', gridTemplateColumns: 'repeat(auto-fill, 12px)', gap: '2px', maxHeight: 220, overflowY: 'auto' }}>
        {cells.map((cell) => (
          <Tooltip
            key={cell.index}
            arrow
            title={`#${cell.index + 1} · ${formatBytes(cell.start)} – ${formatBytes(cell.end)} · ${Math.floor(cell.done * 100)}% · ${t(`downloads.segments.${cell.status}`)}`}
          >
            <Box
              data-status={cell.status}
              sx={{
                width: 12,
                height: 12,
                borderRadius: '2px',
                backgroundColor: CELL_COLOR[cell.status],
                transition: 'transform 0.1s',
                '&:hover': { transform: 'scale(1.3)' },
              }}
            />
          </Tooltip>
        ))}
      </Box>
    </Box>
  );
};

export default SegmentVisualization;
