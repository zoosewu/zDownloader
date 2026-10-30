/**
 * Every command the frontend invokes must be registered in `generate_handler!`
 * (src-tauri/src/lib.rs). ts-rs generates the types but not the command names,
 * so this catches typos and calls to commands that were renamed or removed.
 */
import { describe, test, expect } from 'vitest';
import { readFileSync, readdirSync, statSync } from 'node:fs';
import { join, relative } from 'node:path';

const root = join(__dirname, '..', '..');

function registeredCommands(): Set<string> {
  const lib = readFileSync(join(root, 'src-tauri', 'src', 'lib.rs'), 'utf8');
  const list = lib.match(/generate_handler!\[([\s\S]*?)\]/)?.[1] ?? '';
  return new Set(
    list
      .split('\n')
      .map((line) => line.replace(/\/\/.*$/, '').trim().replace(/,$/, ''))
      .filter(Boolean)
  );
}

function sourceFiles(dir: string): string[] {
  return readdirSync(dir).flatMap((name) => {
    const path = join(dir, name);
    if (statSync(path).isDirectory()) {
      return name === '__tests__' || name === 'bindings' ? [] : sourceFiles(path);
    }
    return /\.(ts|tsx)$/.test(name) ? [path] : [];
  });
}

/** `invoke('x'`, `invoke<T>('x'`, `call<T>('x'` and the double-quoted forms. */
function invokedCommands(): Map<string, string[]> {
  const found = new Map<string, string[]>();
  const pattern = /\b(?:invoke|call)(?:<[^()]*?>)?\(\s*['"]([a-z_]+)['"]/g;
  for (const file of sourceFiles(join(root, 'src'))) {
    for (const [, name] of readFileSync(file, 'utf8').matchAll(pattern)) {
      found.set(name, [...(found.get(name) ?? []), relative(root, file)]);
    }
  }
  return found;
}

describe('command names', () => {
  test('every invoked command is registered in the backend', () => {
    const registered = registeredCommands();
    const invoked = invokedCommands();
    expect(registered.size).toBeGreaterThan(10);
    expect(invoked.size).toBeGreaterThan(10);

    const unknown = [...invoked].filter(([name]) => !registered.has(name));
    expect(unknown).toEqual([]);
  });
});
