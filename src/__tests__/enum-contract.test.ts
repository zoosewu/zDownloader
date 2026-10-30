/**
 * Wire contract: TS enum values must equal what the Rust backend serializes.
 * The Rust side asserts the same fixture (src-tauri/src/models/mod.rs).
 */
import { describe, test, expect } from 'vitest';
import fixtures from '../../test-fixtures/api-samples.json';
import { DownloadStatus, SegmentStatus } from '../types/download';
import { Theme, SpeedLimitMode } from '../types/settings';
import { ProxyStatus } from '../types/proxy';

const cases: [string, object, string[]][] = [
  ['DownloadStatus', DownloadStatus, fixtures.enums.downloadStatus],
  ['SegmentStatus', SegmentStatus, fixtures.enums.segmentStatus],
  ['Theme', Theme, fixtures.enums.theme],
  ['SpeedLimitMode', SpeedLimitMode, fixtures.enums.speedLimitMode],
  ['ProxyStatus', ProxyStatus, fixtures.enums.proxyStatus],
];

describe('enum wire contract', () => {
  test.each(cases)('%s matches backend serialization', (_, tsEnum, wire) => {
    expect(Object.values(tsEnum)).toEqual(wire);
  });
});
