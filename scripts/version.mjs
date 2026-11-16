// The app version lives in package.json (tauri.conf.json reads it from there); the Rust
// crate's version must match, because the app logs and reports CARGO_PKG_VERSION.
//   node scripts/version.mjs check [vX.Y.Z]   fail unless every version (and the tag) agrees
//   node scripts/version.mjs set X.Y.Z        write X.Y.Z everywhere
import { readFileSync, writeFileSync } from 'node:fs';
import { join, resolve } from 'node:path';

const root = resolve(import.meta.dirname, '..');
const files = {
  packageJson: join(root, 'package.json'),
  cargoToml: join(root, 'src-tauri', 'Cargo.toml'),
  cargoLock: join(root, 'src-tauri', 'Cargo.lock'),
};
const CRATE_VERSION = /(\[package\]\r?\nname = "zoo-downloader"\r?\nversion = ")([^"]+)(")/;
const LOCK_VERSION = /(name = "zoo-downloader"\r?\nversion = ")([^"]+)(")/;
const SEMVER = /^\d+\.\d+\.\d+(-[0-9A-Za-z.-]+)?$/;

export function versions() {
  return {
    'package.json': JSON.parse(readFileSync(files.packageJson, 'utf8')).version,
    'src-tauri/Cargo.toml': readFileSync(files.cargoToml, 'utf8').match(CRATE_VERSION)?.[2],
    'src-tauri/Cargo.lock': readFileSync(files.cargoLock, 'utf8').match(LOCK_VERSION)?.[2],
  };
}

export function setVersion(version) {
  if (!SEMVER.test(version)) throw new Error(`not a version: ${version} (use MAJOR.MINOR.PATCH)`);
  const pkg = readFileSync(files.packageJson, 'utf8');
  writeFileSync(files.packageJson, pkg.replace(/("version":\s*")[^"]+(")/, `$1${version}$2`));
  for (const [file, pattern] of [[files.cargoToml, CRATE_VERSION], [files.cargoLock, LOCK_VERSION]]) {
    const text = readFileSync(file, 'utf8');
    if (!pattern.test(text)) throw new Error(`no zoo-downloader version in ${file}`);
    writeFileSync(file, text.replace(pattern, `$1${version}$3`));
  }
}

if (import.meta.main ?? process.argv[1] === import.meta.filename) {
  const [command, arg] = process.argv.slice(2);
  if (command === 'set') {
    setVersion(arg);
    console.log(`version ${arg}`);
  } else if (command === 'check') {
    const found = versions();
    if (arg) found[`tag ${arg}`] = arg.replace(/^v/, '');
    const distinct = new Set(Object.values(found));
    if (distinct.size !== 1 || distinct.has(undefined)) {
      console.error('versions disagree:', found);
      process.exit(1);
    }
    console.log(`version ${[...distinct][0]}`);
  } else {
    console.error('usage: node scripts/version.mjs check [vX.Y.Z] | set X.Y.Z');
    process.exit(2);
  }
}
