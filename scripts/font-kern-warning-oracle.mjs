// Unmodified opentype.js remains the constructor oracle, never a Rust dependency.
import fs from 'node:fs/promises';
import {createRequire} from 'node:module';
import {createHash} from 'node:crypto';
import {deflateSync} from 'node:zlib';
import assert from 'node:assert/strict';
import path from 'node:path';
import {withKern} from '../crates/xterm-font-addons/tests/support/font_metadata_scenarios.mjs';
if (process.argv.slice(2).some(arg => arg !== '--generate')) throw new Error('Use only --generate; SOURCE_PACKAGE_JSON selects the reference dependencies');
const require = createRequire(process.env.SOURCE_PACKAGE_JSON ?? path.resolve('source_repo/package.json'));
const {parse} = require('opentype.js'), dependency = require.resolve('opentype.js');
const hash = bytes => createHash('sha256').update(bytes).digest('hex');
const base = new Uint8Array(await fs.readFile('crates/xterm-addons/tests/fixtures/ligature-fonts/UbuntuMono-Regular.ttf'));
function woff(input) {
  const bytes = Buffer.from(input), count = bytes.readUInt16BE(4), header = Buffer.alloc(44 + count * 20), pieces = [header];
  header.write('wOFF'); bytes.copy(header, 4, 0, 4); header.writeUInt16BE(count, 12);
  let cursor = header.length;
  for (let index = 0; index < count; index++) {
    const at = 12 + index * 16, out = 44 + index * 20;
    const offset = bytes.readUInt32BE(at + 8), length = bytes.readUInt32BE(at + 12);
    const data = bytes.subarray(offset, offset + length), compressed = deflateSync(data);
    const payload = compressed.length < data.length ? compressed : data;
    bytes.copy(header, out, at, at + 4);
    header.writeUInt32BE(cursor, out + 4); header.writeUInt32BE(payload.length, out + 8); header.writeUInt32BE(length, out + 12);
    pieces.push(payload); cursor += payload.length;
  }
  header.writeUInt32BE(cursor, 8); return Buffer.concat(pieces);
}
const cases = [], sentinel = {};
const descriptor = Object.getOwnPropertyDescriptor(console, 'warn');
try {
  for (const compressed of [false, true]) for (const count of [0, 1, 2, 0xffffffff])
    for (const size of [0, 1, 2, 7, 8, 12, 13, 14, 16, 17, 18, 24])
      for (const throws of [false, true]) {
        const payload = new Uint8Array(24), view = new DataView(payload.buffer);
        view.setUint16(0, 1); view.setUint32(4, count);
        const data = payload.slice(0, size), raw = withKern(base, data), bytes = compressed ? woff(raw) : raw;
        const warnings = [];
        Object.defineProperty(console, 'warn', {configurable: true, value(message) {warnings.push(message); if (throws) throw sentinel;}});
        let outcome;
        try {parse(bytes.buffer.slice(bytes.byteOffset, bytes.byteOffset + bytes.byteLength)); outcome = {status: 'ok'};}
        catch(error) {outcome = error === sentinel ? {status: 'warning'} : {status: 'error', name: error.name, message: error.message};}
        cases.push({name: `Mac count${count} size${size} throw${throws} ${compressed ? 'WOFF' : 'SFNT'}`, tag: 'kern', data: [...data], woff: compressed, throws, warnings, outcome});
      }
} finally {
  if (descriptor) Object.defineProperty(console, 'warn', descriptor); else delete console.warn;
}
assert.equal(cases.length, 192);
assert(cases.some(item => item.outcome.status === 'warning'));
const fixture = JSON.stringify({sourceSha256: hash(await fs.readFile(dependency)), cases}) + '\n';
const destination = 'crates/xterm-addons/tests/fixtures/font-kern-warnings.json';
if (process.argv.includes('--generate')) await fs.writeFile(destination, fixture);
else assert.equal(await fs.readFile(destination, 'utf8'), fixture);
await fs.mkdir('results/font-kern-warning', {recursive: true});
await fs.writeFile('results/font-kern-warning/oracle.json', JSON.stringify({
  node: process.version, sourceSha256: hash(await fs.readFile(dependency)), fixtureSha256: hash(fixture),
  cases: cases.length, accepted: cases.filter(c => c.outcome.status === 'ok').length,
  warningFailures: cases.filter(c => c.outcome.status === 'warning').length,
  boundsFailures: cases.filter(c => c.outcome.name === 'RangeError').length
}, null, 2) + '\n');
console.log('192 Mac kern warning constructor cases match unchanged source');
