/**
 * Type-check a project with the installed `tsc` and return its error messages (the text after
 * `error TSxxxx:`, continuation lines joined with '\n'). The CLI is TypeScript's stable
 * interface; its JavaScript API changes between majors.
 */
import { spawnSync } from 'node:child_process';
import { createRequire } from 'node:module';
import { dirname, join } from 'node:path';

const tsc = join(dirname(createRequire(import.meta.url).resolve('typescript/package.json')), 'bin', 'tsc');

export function typeErrors(project: string): string[] {
  const run = spawnSync(process.execPath, [tsc, '-p', project, '--noEmit', '--pretty', 'false'], { encoding: 'utf8' });
  if (run.error) throw run.error;
  const errors: string[] = [];
  for (const line of run.stdout.split(/\r?\n/)) {
    const start = /error TS\d+: (.*)$/.exec(line);
    if (start) errors.push(start[1]);
    else if (/^\s+\S/.test(line) && errors.length > 0) errors[errors.length - 1] += `\n${line.trim()}`;
  }
  if (run.status !== 0 && errors.length === 0) throw new Error(`tsc failed:\n${run.stdout}${run.stderr}`);
  return errors;
}
