// Builds the programs that ship next to the app in release mode and puts them where Tauri's
// externalBin expects them: src-tauri/binaries/<name>-<target triple>[.exe].
//   zoo-module       site-module host
//   zoo-native-host  browser extension bridge
// Run by `tauri build --config src-tauri/tauri.bundle.json` (beforeBuildCommand).
import { execFileSync } from 'node:child_process';
import { copyFileSync, mkdirSync } from 'node:fs';
import { join, resolve } from 'node:path';

const SIDECARS = ['zoo-module', 'zoo-native-host'];

const tauriDir = resolve(import.meta.dirname, '..', 'src-tauri');
const triple =
  process.env.TAURI_ENV_TARGET_TRIPLE ||
  execFileSync('rustc', ['-vV'], { encoding: 'utf8' }).match(/^host: (.+)$/m)[1];
const windowsTarget = triple.includes('windows');
// Cross-building Windows binaries elsewhere goes through cargo-xwin, like the app itself.
const build = windowsTarget && process.platform !== 'win32' ? ['xwin', 'build'] : ['build'];
const packages = SIDECARS.flatMap((name) => ['-p', name]);

execFileSync('cargo', [...build, '--release', ...packages, '--target', triple], { cwd: tauriDir, stdio: 'inherit' });

const targetDir = process.env.CARGO_TARGET_DIR ? resolve(process.env.CARGO_TARGET_DIR) : join(tauriDir, 'target');
const ext = windowsTarget ? '.exe' : '';
mkdirSync(join(tauriDir, 'binaries'), { recursive: true });
for (const name of SIDECARS) {
  const to = join(tauriDir, 'binaries', `${name}-${triple}${ext}`);
  copyFileSync(join(targetDir, triple, 'release', `${name}${ext}`), to);
  console.log(`${name}: ${to}`);
}
