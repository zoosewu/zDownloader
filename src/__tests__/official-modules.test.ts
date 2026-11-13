/**
 * The official marketplace's modules: their committed bundles must match their sources
 * (`pnpm build:official-modules`), their types/ must be the current SDK, and the sources
 * must type-check against it. Bundling runs in a child process: esbuild refuses jsdom.
 */
import { spawnSync } from 'node:child_process';
import { existsSync, readdirSync, readFileSync } from 'node:fs';
import { join, resolve } from 'node:path';
import ts from 'typescript';
import { describe, expect, test } from 'vitest';

const root = resolve(__dirname, '../..');
const marketplace = join(root, 'src-tauri/marketplace');
const sdk = join(root, 'src-tauri/crates/zoo-module/sdk');
const modules = readdirSync(marketplace, { withFileTypes: true })
  .filter((e) => e.isDirectory() && existsSync(join(marketplace, e.name, 'src', 'index.ts')))
  .map((e) => join(marketplace, e.name));

describe('official modules', () => {
  test('there is at least one', () => {
    expect(modules.length).toBeGreaterThan(0);
  });

  test('every dist/index.js is built from its src', () => {
    const run = spawnSync(process.execPath, [join(root, 'scripts/build-official-modules.mjs'), '--check'], { encoding: 'utf8' });
    expect(run.stderr).toBe('');
    expect(run.status).toBe(0);
  }, 30_000);

  test.each(modules)('%s: types/ is the current SDK', (dir) => {
    for (const file of ['zoo-module.d.ts', 'globals.d.ts', 'manifest.schema.json']) {
      expect(readFileSync(join(dir, 'types', file), 'utf8'), `copy sdk/${file} to types/`).toBe(readFileSync(join(sdk, file), 'utf8'));
    }
  });

  test.each(modules)('%s: type-checks', (dir) => {
    const config = ts.parseJsonConfigFileContent(ts.readConfigFile(join(dir, 'tsconfig.json'), ts.sys.readFile).config, ts.sys, dir);
    const program = ts.createProgram(config.fileNames, config.options);
    expect(ts.getPreEmitDiagnostics(program).map((d) => ts.flattenDiagnosticMessageText(d.messageText, '\n'))).toEqual([]);
  }, 30_000);
});
