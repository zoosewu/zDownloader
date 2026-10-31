// Turns a backend SegmentMap into fixed-size cells for the segment grid.
import type { SegmentMap } from '../backend';

export type CellStatus = 'pending' | 'partial' | 'downloading' | 'retrying' | 'completed';

export interface SegmentCell {
  index: number;
  /** Byte range `[start, end)`. */
  start: number;
  end: number;
  status: CellStatus;
  /** Share of the cell's bytes already downloaded, 0-1. */
  done: number;
}

/** Cells use the transfer's segment size, merged into larger cells once there would be more than this. */
export const MAX_CELLS = 400;

function overlap(aStart: number, aEnd: number, bStart: number, bEnd: number): number {
  return Math.max(0, Math.min(aEnd, bEnd) - Math.max(aStart, bStart));
}

/** Empty when the size is unknown (nothing to divide). */
export function buildSegmentCells(map: SegmentMap, maxCells = MAX_CELLS): SegmentCell[] {
  const total = map.total;
  if (!total || total <= 0) return [];
  const cellSize = Math.max(map.segmentSize, Math.ceil(total / maxCells), 1);
  const count = Math.ceil(total / cellSize);

  return Array.from({ length: count }, (_, index) => {
    const start = index * cellSize;
    const end = Math.min(total, start + cellSize);
    const covered = map.done.reduce((sum, [s, e]) => sum + overlap(start, end, s, e), 0);
    const active = map.active.filter((r) => overlap(start, end, r.start, r.end) > 0);
    const done = covered / (end - start);

    let status: CellStatus;
    if (done >= 1) status = 'completed';
    else if (active.some((r) => r.retrying)) status = 'retrying';
    else if (active.length > 0) status = 'downloading';
    else if (covered > 0) status = 'partial';
    else status = 'pending';

    return { index, start, end, status, done };
  });
}
