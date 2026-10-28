// Copyright (c) 2023, 2026 The xterm.js authors. All rights reserved.
// Licensed under MIT. Reference is the installed original WASM decoder, not Rust.
import assert from 'node:assert/strict';
import { createRequire } from 'node:module';
import { readFile, writeFile, mkdir } from 'node:fs/promises';
import { createHash } from 'node:crypto';
const require = createRequire(import.meta.url);
const modulePath = 'source_repo/node_modules/xterm-wasm-parts/lib/base64/Base64Decoder.wasm.js';
const Decoder = require(`../${modulePath}`).default;
const hex = bytes => Array.from(bytes, value => value.toString(16)).join(',');
function packed(data) {
  // Compress only exact periodic byte sequences. Rust checks every resulting
  // byte, not a lossy checksum, including the 131072-byte deferred-decode cases.
  if (data.length > 32) for (let period = 1; period <= 16; period++) {
    if (data.length % period === 0 && data.every((value, index) => value === data[index % period])) {
      return `r:${data.length / period}:${hex(data.slice(0, period))}`;
    }
  }
  return `h:${hex(data)}`;
}
const rows = [];
let calls = 0;
function trace(keep, maximum, initial, operations) {
  const decoder = new Decoder(keep, maximum, initial);
  const snapshots = [];
  for (const operation of operations) {
    let status;
    const [kind, first, second] = operation.split(':');
    if (kind === 'i') { decoder.init(); status = 'i'; }
    else if (kind === 'r') { decoder.release(); status = 'r'; }
    else if (kind === 'e') status = String(decoder.end());
    else {
      const pattern = (kind === 'p' ? first : second).split(',').filter(Boolean).map(value => parseInt(value, 16));
      const data = kind === 'p' ? pattern : Array.from({ length: Number(first) * pattern.length }, (_, i) => pattern[i % pattern.length]);
      status = String(decoder.put(Uint32Array.from(data)));
    }
    calls++;
    snapshots.push(`${status}/${decoder.loadedBytes}/${decoder.freeBytes}/${packed(decoder.data8)}`);
  }
  rows.push([keep, maximum, initial, operations.join('|'), snapshots.join('~')].join('\t'));
}
const inputs = ['', 'A', 'AA', 'AAA', 'AAAA', 'AA=', 'AA==', 'AAA=', 'A===', '====', 'QUJDRA==', 'QUJDRA', 'QUJDRA=', 'QUJD=AAA', 'QUJD\nA', 'QUJD-A', 'QUJD_A', 'AB', 'ABC', '//', '/w==', '////', 'aGVsbG8='];
for (const text of inputs) for (const width of [0, 1, 2, 3, 4, 7]) {
  const units = Array.from(text, char => char.charCodeAt(0));
  const ops = ['e', 'p:51', 'i'];
  for (let offset = 0; offset < units.length; offset += width || units.length) ops.push(`p:${hex(units.slice(offset, offset + (width || units.length)))}`);
  ops.push('e', 'e', 'p:51', 'r', 'p:51', 'i', 'p:51,51,3d,3d', 'e');
  trace(32, 128, 8, ops);
}
const seed = 0x0b64c0de;
let state = seed;
function random() { state ^= state << 13; state ^= state >>> 17; state ^= state << 5; return state >>> 0; }
const alphabet = [0, 10, 32, 43, 47, 48, 61, 65, 66, 81, 97, 0x10041, 0x10051, 0xd800, 0xffffffff];
for (let iteration = 0; iteration < 128; iteration++) {
  const ops = [];
  for (let step = 0; step < 48; step++) {
    const choice = random() % 12;
    if (choice < 2) ops.push('i');
    else if (choice === 2) ops.push('r');
    else if (choice < 6) ops.push('e');
    else ops.push(`p:${hex(Array.from({ length: random() % 12 }, () => alphabet[random() % alphabet.length]))}`);
  }
  trace(16, 64, 8, ops);
}
trace(262144, 300000, 32768, ['i', 'q:32768:51,55,4a,44', 'e', 'e', 'r']);
trace(262144, 300000, 32768, ['i', 'q:32768:51,55,4a,44', 'p:52,41,3d,3d', 'e', 'r']);
trace(262144, 300000, 32768, ['i', 'p:21', 'q:32768:51,55,4a,44', 'e', 'r']);
trace(32, 8, 4, ['i', 'p:51,55,4a,44', 'p:51,55,4a,44', 'p:51', 'e', 'r', 'i', 'p:10051,10051', 'e']);
const corpus = rows.join('\n') + '\n';
const fixture = 'crates/xterm-addons/tests/fixtures/image-base64.tsv';
await mkdir('crates/xterm-addons/tests/fixtures', { recursive: true });
if (process.argv.includes('--generate')) await writeFile(fixture, corpus);
else assert.equal((await readFile(fixture, 'utf8')).replace(/\r\n/g, '\n'), corpus, 'image base64 oracle drift');
const sources = await Promise.all([modulePath, 'source_repo/node_modules/xterm-wasm-parts/LICENSE', 'source_repo/node_modules/xterm-wasm-parts/package.json'].map(async path => ({ path, sha256: createHash('sha256').update(await readFile(path)).digest('hex') })));
const report = { node: process.version, platform: process.platform, seed, traces: rows.length, calls, sources,
  fixtureBytes: Buffer.byteLength(corpus), fixtureSha256: createHash('sha256').update(corpus).digest('hex'),
  status: process.argv.includes('--generate') ? 'generated' : 'matched', scope: 'strict original payload decoder bytes/status/offset/release semantics only, not live image support' };
await mkdir('results/kitty-graphics', { recursive: true });
await writeFile('results/kitty-graphics/base64-oracle.json', JSON.stringify(report, null, 2) + '\n');
console.log(JSON.stringify(report, null, 2));
