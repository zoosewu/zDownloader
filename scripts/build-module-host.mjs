// Builds the site-module host (zoo-module) in release mode and puts it where Tauri's
// externalBin expects it: src-tauri/binaries/zoo-module-<target triple>[.exe].
// Run by `tauri build --config src-tauri/tauri.bundle.json` (beforeBuildCommand).
import { execFileSync } from 'node:child_process';
import { copyFileSync, mkdirSync } from 'node:fs';
import { join, resolve } from 'node:path';

const tauriDir = resolve(import.meta.dirname, '..', 'src-tauri');
const triple =
  process.env.TAURI_ENV_TARGET_TRIPLE ||
  execFileSync('rustc', ['-vV'], { encoding: 'utf8' }).match(/^host: (.+)$/m)[1];
const windowsTarget = triple.includes('windows');
// Cross-building Windows binaries elsewhere goes through cargo-xwin, like the app itself.
const build = windowsTarget && process.platform !== 'win32' ? ['xwin', 'build'] : ['build'];

execFileSync('cargo', [...build, '--release', '-p', 'zoo-module', '--target', triple], { cwd: tauriDir, stdio: 'inherit' });

const targetDir = process.env.CARGO_TARGET_DIR ? resolve(process.env.CARGO_TARGET_DIR) : join(tauriDir, 'target');
const ext = windowsTarget ? '.exe' : '';
mkdirSync(join(tauriDir, 'binaries'), { recursive: true });
const from = join(targetDir, triple, 'release', `zoo-module${ext}`);
const to = join(tauriDir, 'binaries', `zoo-module-${triple}${ext}`);
copyFileSync(from, to);
console.log(`module host: ${to}`);
