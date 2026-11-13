// Bundles every module of the official marketplace (src-tauri/marketplace/<id>/src/index.ts)
// into its dist/index.js, the file the app runs. The bundles are committed; --check only
// reports the ones that differ from their sources (a test runs it).
import { existsSync, readdirSync, readFileSync } from 'node:fs';
import { join, resolve } from 'node:path';
import { build } from 'esbuild';

export const marketplace = resolve(import.meta.dirname, '..', 'src-tauri', 'marketplace');

/** Directories of the official modules written in TypeScript. */
export function officialModules() {
  return readdirSync(marketplace, { withFileTypes: true })
    .filter((e) => e.isDirectory() && existsSync(join(marketplace, e.name, 'src', 'index.ts')))
    .map((e) => join(marketplace, e.name));
}

/** The same options as the module template's `pnpm build`. */
export async function bundle(dir, write = true) {
  const result = await build({
    entryPoints: [join(dir, 'src', 'index.ts')],
    bundle: true,
    format: 'esm',
    target: 'es2023',
    external: ['zoo-module'],
    outfile: join(dir, 'dist', 'index.js'),
    write,
    logLevel: 'warning',
  });
  return result.outputFiles?.[0]?.text;
}

if (process.argv[1] && resolve(process.argv[1]) === resolve(import.meta.filename)) {
  const check = process.argv.includes('--check');
  for (const dir of officialModules()) {
    const dist = join(dir, 'dist', 'index.js');
    if (!check) {
      await bundle(dir);
      console.log(`built ${dist}`);
    } else if (!existsSync(dist) || readFileSync(dist, 'utf8').replace(/\r\n/g, '\n') !== (await bundle(dir, false))) {
      console.error(`${dist} is out of date: run pnpm build:official-modules`);
      process.exitCode = 1;
    }
  }
}
