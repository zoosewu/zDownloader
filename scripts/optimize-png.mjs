// Lossless PNG shrinking with Node alone (no tools to install): decode, re-filter each row
// for the smallest output, deflate at the highest level, and drop chunks a browser does not
// need (text, time, physical size). The pixels stay exactly the same.
//   node scripts/optimize-png.mjs <file.png>...
// Handles 8-bit non-interlaced grey, RGB, grey+alpha and RGBA (what scripts/make-icon.ps1
// renders); other PNGs are left alone.
import { readFileSync, writeFileSync } from 'node:fs';
import { crc32, deflateSync, inflateSync } from 'node:zlib';

const SIGNATURE = Buffer.from([0x89, 0x50, 0x4e, 0x47, 0x0d, 0x0a, 0x1a, 0x0a]);
const CHANNELS = { 0: 1, 2: 3, 4: 2, 6: 4 };
const KEEP = new Set(['IHDR', 'PLTE', 'tRNS', 'IDAT', 'IEND', 'sRGB', 'gAMA', 'cHRM', 'iCCP']);

function chunks(png) {
  const out = [];
  for (let at = 8; at < png.length; ) {
    const length = png.readUInt32BE(at);
    out.push({ type: png.toString('latin1', at + 4, at + 8), data: png.subarray(at + 8, at + 8 + length) });
    at += 12 + length;
  }
  return out;
}

function chunk(type, data) {
  const head = Buffer.alloc(8);
  head.writeUInt32BE(data.length, 0);
  head.write(type, 4, 'latin1');
  const crc = Buffer.alloc(4);
  crc.writeUInt32BE(crc32(Buffer.concat([head.subarray(4), data])) >>> 0);
  return Buffer.concat([head, data, crc]);
}

const paeth = (a, b, c) => {
  const p = a + b - c;
  const [pa, pb, pc] = [Math.abs(p - a), Math.abs(p - b), Math.abs(p - c)];
  return pa <= pb && pa <= pc ? a : pb <= pc ? b : c;
};

/** Raw rows (without filter bytes) from the filtered scanlines. */
function unfilter(data, width, height, bpp) {
  const stride = width * bpp;
  const rows = [];
  let prev = Buffer.alloc(stride);
  for (let y = 0; y < height; y++) {
    const type = data[y * (stride + 1)];
    const line = Buffer.from(data.subarray(y * (stride + 1) + 1, (y + 1) * (stride + 1)));
    for (let i = 0; i < stride; i++) {
      const a = i >= bpp ? line[i - bpp] : 0;
      const b = prev[i];
      const c = i >= bpp ? prev[i - bpp] : 0;
      line[i] = (line[i] + [0, a, b, (a + b) >> 1, paeth(a, b, c)][type]) & 0xff;
    }
    rows.push(line);
    prev = line;
  }
  return rows;
}

function filterRow(type, line, prev, bpp) {
  const out = Buffer.alloc(line.length + 1);
  out[0] = type;
  for (let i = 0; i < line.length; i++) {
    const a = i >= bpp ? line[i - bpp] : 0;
    const b = prev[i];
    const c = i >= bpp ? prev[i - bpp] : 0;
    out[i + 1] = (line[i] - [0, a, b, (a + b) >> 1, paeth(a, b, c)][type]) & 0xff;
  }
  return out;
}

/** Filtered scanlines: one filter for every row, or (-1) the row-by-row smallest guess. */
function refilter(rows, bpp, fixed) {
  let prev = Buffer.alloc(rows[0].length);
  const out = [];
  for (const line of rows) {
    let best;
    for (const type of fixed >= 0 ? [fixed] : [0, 1, 2, 3, 4]) {
      const candidate = filterRow(type, line, prev, bpp);
      const cost = candidate.reduce((sum, v, i) => (i ? sum + (v < 128 ? v : 256 - v) : sum), 0);
      if (!best || cost < best.cost) best = { candidate, cost };
    }
    out.push(best.candidate);
    prev = line;
  }
  return Buffer.concat(out);
}

for (const file of process.argv.slice(2)) {
  const png = readFileSync(file);
  if (!png.subarray(0, 8).equals(SIGNATURE)) throw new Error(`${file}: not a PNG`);
  const all = chunks(png);
  const ihdr = all[0].data;
  const [width, height, depth, color, interlace] = [ihdr.readUInt32BE(0), ihdr.readUInt32BE(4), ihdr[8], ihdr[9], ihdr[12]];
  if (depth !== 8 || interlace !== 0 || !CHANNELS[color]) {
    console.log(`${file}: left as is (depth ${depth}, color type ${color}, interlace ${interlace})`);
    continue;
  }
  const bpp = CHANNELS[color];
  const rows = unfilter(inflateSync(Buffer.concat(all.filter((c) => c.type === 'IDAT').map((c) => c.data))), width, height, bpp);
  const idat = [-1, 0, 1, 2, 3, 4]
    .map((strategy) => deflateSync(refilter(rows, bpp, strategy), { level: 9, memLevel: 9 }))
    .reduce((a, b) => (b.length < a.length ? b : a));
  const kept = all.filter((c) => KEEP.has(c.type) && c.type !== 'IDAT' && c.type !== 'IEND');
  const out = Buffer.concat([SIGNATURE, ...kept.map((c) => chunk(c.type, c.data)), chunk('IDAT', idat), chunk('IEND', Buffer.alloc(0))]);
  // Same pixels or nothing: decode the result and compare.
  const check = unfilter(inflateSync(idat), width, height, bpp);
  if (!check.every((row, y) => row.equals(rows[y]))) throw new Error(`${file}: re-encoding changed the pixels`);
  if (out.length < png.length) {
    writeFileSync(file, out);
    console.log(`${file}: ${png.length} -> ${out.length} bytes`);
  } else {
    console.log(`${file}: already small (${png.length} bytes)`);
  }
}
