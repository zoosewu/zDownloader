import { describe, test, expect } from 'vitest';
import { buildSegmentCells } from '../utils/segments';
import type { SegmentMap } from '../backend';

const map = (overrides: Partial<SegmentMap>): SegmentMap => ({
  total: 400,
  segmentSize: 100,
  done: [],
  active: [],
  ...overrides,
});

describe('buildSegmentCells', () => {
  test('one cell per segment, last cell trimmed to the file size', () => {
    const cells = buildSegmentCells(map({ total: 350 }));
    expect(cells.map((c) => [c.start, c.end])).toEqual([
      [0, 100],
      [100, 200],
      [200, 300],
      [300, 350],
    ]);
  });

  test('status reflects done ranges and connections', () => {
    const cells = buildSegmentCells(
      map({
        done: [[0, 150]],
        active: [
          { start: 200, end: 300, retrying: false },
          { start: 300, end: 400, retrying: true },
        ],
      })
    );
    expect(cells.map((c) => c.status)).toEqual(['completed', 'partial', 'downloading', 'retrying']);
    expect(cells[1].done).toBeCloseTo(0.5);
  });

  test('a fully downloaded cell is completed even if a connection still lists it', () => {
    const cells = buildSegmentCells(map({ done: [[0, 100]], active: [{ start: 0, end: 100, retrying: false }] }));
    expect(cells[0].status).toBe('completed');
  });

  test('large files merge segments so the grid stays bounded', () => {
    const cells = buildSegmentCells(map({ total: 10_000, segmentSize: 1 }), 50);
    expect(cells).toHaveLength(50);
    expect(cells[49].end).toBe(10_000);
  });

  test('unknown size yields no cells', () => {
    expect(buildSegmentCells(map({ total: null }))).toEqual([]);
  });
});
