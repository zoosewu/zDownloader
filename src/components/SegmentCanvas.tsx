// The segment grid, drawn on a canvas in device pixels. A CSS grid of 12px cells with 2px gaps
// lands on half pixels at 125% display scaling, so the gaps alternate between 2 and 3 pixels;
// here every cell and gap is a whole number of device pixels. One tooltip follows the pointer.
import React, { useEffect, useRef, useState } from 'react';
import { Box, Tooltip } from '@mui/material';
import type { SegmentCell, CellStatus } from '../utils/segments';

const CELL = 12; // CSS px
const GAP = 2;

interface Props {
  cells: SegmentCell[];
  colors: Record<CellStatus, string>;
  describe: (cell: SegmentCell) => string;
}

const SegmentCanvas: React.FC<Props> = ({ cells, colors, describe }) => {
  const box = useRef<HTMLDivElement>(null);
  const canvas = useRef<HTMLCanvasElement>(null);
  const [width, setWidth] = useState(0);
  const [hover, setHover] = useState<SegmentCell | null>(null);

  useEffect(() => {
    const el = box.current;
    if (!el) return;
    const observer = new ResizeObserver(([entry]) => setWidth(entry.contentRect.width));
    observer.observe(el);
    return () => observer.disconnect();
  }, []);

  const dpr = window.devicePixelRatio || 1;
  const cell = Math.round(CELL * dpr);
  const gap = Math.max(1, Math.round(GAP * dpr));
  const pitch = cell + gap;
  const columns = Math.max(1, Math.floor((Math.floor(width * dpr) + gap) / pitch));
  const rows = Math.ceil(cells.length / columns);

  useEffect(() => {
    const c = canvas.current;
    if (!c || width === 0) return;
    c.width = columns * pitch - gap;
    c.height = Math.max(0, rows * pitch - gap);
    const ctx = c.getContext('2d');
    if (!ctx) return;
    ctx.clearRect(0, 0, c.width, c.height);
    cells.forEach((s, i) => {
      ctx.fillStyle = colors[s.status];
      ctx.fillRect((i % columns) * pitch, Math.floor(i / columns) * pitch, cell, cell);
    });
  }, [cells, colors, width, columns, rows, pitch, cell, gap]);

  const at = (e: React.MouseEvent<HTMLCanvasElement>) => {
    const rect = e.currentTarget.getBoundingClientRect();
    const x = Math.floor(((e.clientX - rect.left) * dpr) / pitch);
    const y = Math.floor(((e.clientY - rect.top) * dpr) / pitch);
    const index = y * columns + x;
    return x < columns && index < cells.length ? cells[index] : null;
  };

  return (
    <Box ref={box} sx={{ maxHeight: 220, overflowY: 'auto', lineHeight: 0 }}>
      <Tooltip title={hover ? describe(hover) : ''} followCursor placement="top" enterDelay={0}>
        <canvas
          ref={canvas}
          style={{ width: (columns * pitch - gap) / dpr, height: Math.max(0, rows * pitch - gap) / dpr }}
          onMouseMove={(e) => setHover(at(e))}
          onMouseLeave={() => setHover(null)}
        />
      </Tooltip>
    </Box>
  );
};

export default SegmentCanvas;
