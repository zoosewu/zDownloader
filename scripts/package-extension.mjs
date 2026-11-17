// The browser extension as a store upload (Edge Add-ons, Chrome Web Store):
// dist-store/zoo-downloader-extension-<version>.zip. Same files as the folder people load by
// hand, minus its tests and README, and without manifest `key`: the store gives the extension
// its own ID (the app lets both IDs in, see zoo-native-host).
//   node scripts/package-extension.mjs
import { mkdirSync, readdirSync, readFileSync, statSync, writeFileSync } from 'node:fs';
import { join, relative, sep } from 'node:path';
import { crc32, deflateRawSync } from 'node:zlib';

const root = join(import.meta.dirname, '..');
const source = join(root, 'browser-extension');
const SKIP = new Set(['test', 'README.md', 'node_modules']);

/** Every file to ship, as [path in the zip, contents]. */
function files() {
  const out = [];
  const walk = (dir) => {
    for (const name of readdirSync(dir).sort()) {
      if (SKIP.has(name)) continue;
      const path = join(dir, name);
      if (statSync(path).isDirectory()) walk(path);
      else out.push([relative(source, path).split(sep).join('/'), readFileSync(path)]);
    }
  };
  walk(source);
  return out.map(([name, data]) => {
    if (name !== 'manifest.json') return [name, data];
    const manifest = JSON.parse(data.toString('utf8'));
    delete manifest.key;
    return [name, Buffer.from(JSON.stringify(manifest, null, 2) + '\n')];
  });
}

/** A zip file (deflate, UTF-8 names). */
function zip(entries) {
  const local = [];
  const central = [];
  let offset = 0;
  for (const [name, data] of entries) {
    const nameBytes = Buffer.from(name, 'utf8');
    const packed = deflateRawSync(data, { level: 9 });
    const crc = crc32(data);
    const header = Buffer.alloc(30);
    header.writeUInt32LE(0x04034b50, 0);
    header.writeUInt16LE(20, 4); // version needed
    header.writeUInt16LE(0x0800, 6); // UTF-8 names
    header.writeUInt16LE(8, 8); // deflate
    header.writeUInt32LE(0, 10); // time and date: 1980-01-01, so the zip is reproducible
    header.writeUInt32LE(crc, 14);
    header.writeUInt32LE(packed.length, 18);
    header.writeUInt32LE(data.length, 22);
    header.writeUInt16LE(nameBytes.length, 26);
    local.push(header, nameBytes, packed);

    const entry = Buffer.alloc(46);
    entry.writeUInt32LE(0x02014b50, 0);
    entry.writeUInt16LE(20, 4);
    entry.writeUInt16LE(20, 6);
    entry.writeUInt16LE(0x0800, 8);
    entry.writeUInt16LE(8, 10);
    entry.writeUInt32LE(0, 12);
    entry.writeUInt32LE(crc, 16);
    entry.writeUInt32LE(packed.length, 20);
    entry.writeUInt32LE(data.length, 24);
    entry.writeUInt16LE(nameBytes.length, 28);
    entry.writeUInt32LE(offset, 42);
    central.push(entry, nameBytes);
    offset += header.length + nameBytes.length + packed.length;
  }
  const directory = Buffer.concat(central);
  const end = Buffer.alloc(22);
  end.writeUInt32LE(0x06054b50, 0);
  end.writeUInt16LE(entries.length, 8);
  end.writeUInt16LE(entries.length, 10);
  end.writeUInt32LE(directory.length, 12);
  end.writeUInt32LE(offset, 16);
  return Buffer.concat([...local, directory, end]);
}

const entries = files();
const { version } = JSON.parse(entries.find(([name]) => name === 'manifest.json')[1].toString('utf8'));
const outDir = join(root, 'dist-store');
mkdirSync(outDir, { recursive: true });
const out = join(outDir, `zoo-downloader-extension-${version}.zip`);
writeFileSync(out, zip(entries));
console.log(`${relative(root, out)}: ${entries.length} files`);
for (const [name] of entries) console.log(`  ${name}`);
