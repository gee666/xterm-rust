// Copyright (c) 2021 Joerg Breitbart; xterm.js authors. MIT license.
// Uses the installed, unmodified sixel 0.16.0 decoder only as a test oracle.
import { readFile, writeFile, mkdir } from 'node:fs/promises';
import { createHash } from 'node:crypto';
import { createRequire } from 'node:module';
import { strict as assert } from 'node:assert';
import { fileURLToPath } from 'node:url';
import path from 'node:path';
import { verifyOrGenerate } from './oracle-fixture.mjs';

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..');
const require = createRequire(import.meta.url);
const dependency = path.join(root, 'source_repo/node_modules/sixel');
const pkg = JSON.parse(await readFile(path.join(dependency, 'package.json'), 'utf8'));
assert.equal(pkg.version, '0.16.0');
const { Decoder } = require(path.join(dependency, 'lib/Decoder.js'));
const colors = require(path.join(dependency, 'lib/Colors.js'));
const limits = require(path.join(dependency, 'lib/wasm.js')).LIMITS;
assert.equal(colors.BIG_ENDIAN, false, 'the original dependency documents only little-endian support');
assert.deepEqual([limits.CHUNK_SIZE, limits.PALETTE_SIZE, limits.MAX_WIDTH], [16384, 4096, 16384]);
const addonPalette = Uint32Array.from(colors.PALETTE_ANSI_256);
addonPalette.set(colors.PALETTE_VT340_COLOR);
const cases = [];
const units = text => Array.from({ length: text.length }, (_, i) => text.charCodeAt(i));
const rawNumber = value => value === null ? 0 : Number(value);
function runLengths(values) {
  const result = [];
  for (const value of values) {
    const last = result.at(-1);
    if (last?.[0] === value) last[1]++;
    else result.push([value, 1]);
  }
  return result;
}
function errorInfo(error, phase) { return { phase, name: error.name, message: error.message }; }
function add(name, input, extra = {}) {
  const config = { memoryLimit: '134217728', paletteLimit: '4096',
    fill: 0, truncate: true, addonPalette: false, ...extra };
  const step = (config.step ?? input.length) || 1;
  const chunks = [];
  for (let i = 0; i < input.length; i += step) chunks.push(input.slice(i, i + step));
  if (!chunks.length) chunks.push([]);
  let decoder;
  let constructorError = null;
  try {
    decoder = new Decoder({ memoryLimit: rawNumber(config.memoryLimit),
      paletteLimit: rawNumber(config.constructorPaletteLimit ?? config.paletteLimit),
      palette: config.addonPalette ? addonPalette : colors.PALETTE_VT340_COLOR });
  } catch (error) { constructorError = errorInfo(error, 'constructor'); }
  const snapshots = [];
  let initialPalette = [];
  if (decoder) {
    decoder.init(config.fill, null, rawNumber(config.paletteLimit), config.truncate);
    const paletteBasis = Uint32Array.from(decoder._palette);
    initialPalette = Array.from(paletteBasis, (value, index) => [index, value]).filter(pair => pair[1] !== 0);
    for (let index = 0; index < chunks.length; index++) {
      let error = null;
      try { decoder.decode(Uint32Array.from(chunks[index])); }
      catch (caught) { error = errorInfo(caught, 'decode'); decoder.release(); }
      if (config.observeEach !== false || error || index + 1 === chunks.length) {
        const before = decoder.properties;
        let pixels = [];
        if (!error) {
          try { pixels = runLengths(decoder.data32); }
          catch (caught) { error = errorInfo(caught, 'pixels'); decoder.release(); }
        }
        const after = decoder.properties;
        snapshots.push({ chunk: index, error, before,
          after: JSON.stringify(after) === JSON.stringify(before) ? null : after,
          pixels, paletteLength: decoder.palette.length,
          palette: Array.from(decoder._palette, (value, index) => [index, value]).filter(pair => pair[1] !== paletteBasis[pair[0]]) });
      }
      if (error) break;
    }
  }
  cases.push({ name, config, chunks, initialPalette, constructorError, snapshots });
}

for (const text of ['', '?', '@', 'A', '~', '-', '---', '~$#1@', '!2~-!1~-',
  '"1;1;3;7#1~-', '"1;1;0;32#', '"1;1;3;0#', '"1;1;3;7', '"1;1~',
  '#1;2;100;0;0!2~-#2;1;0;50;100@', '#1;1;0;1000;100~', '#1;1;0;50;1000~',
  '!0~', '!16384~', '!2147483648~', '!4294967295~', '#4095~', '#1;2~',
  '123"1;1;3;2#1@', '~"1;1;1;1~', '"1;1;16384;1!16384~']) {
  for (const step of [1, 2, 3, 16384]) add(`directed:${JSON.stringify(text)}:${step}`, units(text), { step });
}
for (const memoryLimit of [null, '0', '-0', '-1', '0.5', '4', '8', 'NaN', 'Infinity']) {
  for (const text of ['~', '"1;1;2;1~', '!200~-']) add(`memory:${memoryLimit}:${text}`, units(text), { memoryLimit });
}
for (const paletteLimit of ['0', '0.5', '-1', '1', '16', '256', '4096', '4096.5', 'NaN', 'Infinity']) {
  for (const text of ['~', '#4095~']) add(`palette:${paletteLimit}:${text}`, units(text), { paletteLimit });
}
for (const dimension of ['2147483647', '2147483648', '4294967292', '4294967293', '4294967295']) {
  add(`raster-width:${dimension}`, units(`"1;1;${dimension};2#~`));
  add(`raster-height:${dimension}`, units(`"1;1;2;${dimension}#~`));
}
const seed = 0x516e1;
let state = seed;
function random() { state ^= state << 13; state ^= state >>> 17; state ^= state << 5; return state >>> 0; }
for (let iteration = 0; iteration < 512; iteration++) {
  let text = random() % 3 ? '' : `"1;1;${random() % 33};${random() % 25}`;
  for (let op = 0, count = 1 + random() % 20; op < count; op++) {
    switch (random() % 8) {
      case 0: text += String.fromCharCode(63 + random() % 64); break;
      case 1: text += `!${random() % 50}${String.fromCharCode(63 + random() % 64)}`; break;
      case 2: text += `#${random() % 4200};2;${random() % 101};${random() % 101};${random() % 101}`; break;
      case 3: text += `#${random() % 4200};1;${random() % 361};${random() % 101};${random() % 101}`; break;
      case 4: text += '#'; break;
      case 5: text += '$'; break;
      case 6: text += '-'; break;
      default: text += String.fromCharCode(random() % 256); break;
    }
  }
  const input = units(text).map(value => random() % 7 ? value : (value + 0x123400) >>> 0);
  add(`seed:${iteration}`, input, { step: 1 + random() % 17, fill: random(),
    paletteLimit: String([1, 16, 256, 4096][random() % 4]), truncate: !!(random() % 2),
    addonPalette: !!(random() % 2) });
}
const fixturePath = 'source_repo/addons/addon-image/fixture/palette.sixel';
const fullFixture = await readFile(path.join(root, fixturePath));
const begin = fullFixture.indexOf(0x71);
const finish = fullFixture.lastIndexOf(Buffer.from([0x1b, 0x5c]));
assert.ok(begin >= 0 && finish > begin, 'palette.sixel must contain a complete DCS q picture');
const picture = Array.from(fullFixture.subarray(begin + 1, finish));
for (const step of [1, 17, 16384]) add(`palette.sixel:${step}`, picture,
  { step, addonPalette: true, observeEach: false });
const corpus = cases.map(value => JSON.stringify(value)).join('\n') + '\n';
assert.ok(cases.length > 600);
assert.ok(Buffer.byteLength(corpus) < 20 * 1024 * 1024, 'source-only oracle output must stay below20MiB');
const fixture = 'crates/xterm-addons/tests/fixtures/sixel-oracle.ndjson';
await mkdir(path.dirname(path.join(root, fixture)), { recursive: true });
await verifyOrGenerate(path.join(root, fixture), corpus, process.argv.includes('--generate'));
const sourceHashes = {};
for (const file of ['package.json', 'LICENSE', 'lib/Decoder.js', 'lib/Colors.js', 'lib/wasm.js']) {
  sourceHashes[`source_repo/node_modules/sixel/${file}`] = createHash('sha256').update(await readFile(path.join(dependency, file))).digest('hex');
}
sourceHashes[fixturePath] = createHash('sha256').update(fullFixture).digest('hex');
const report = { scope: 'Exact dependency decoder pixels/properties, not live terminal/rendering proof',
  dependency: { name: 'sixel', version: pkg.version, license: pkg.license,
    tagCommit: '35f542f8b1b9a8867fab133aea5d43cb3da6108a',
    referenceUrl: 'https://github.com/jerch/node-sixel/tree/35f542f8b1b9a8867fab133aea5d43cb3da6108a' },
  environment: { node: process.version, platform: process.platform, arch: process.arch },
  limits: { chunkSize: limits.CHUNK_SIZE, paletteSize: limits.PALETTE_SIZE, maxWidth: limits.MAX_WIDTH },
  schemaVersion: 2, paletteFormat: 'full4096-word initial sparse basis plus per-snapshot deltas; unchanged properties repeat via null after',
  seed, cases: cases.length, snapshots: cases.reduce((sum, row) => sum + row.snapshots.length, 0),
  fixtureBytes: Buffer.byteLength(corpus), fixtureSha256: createHash('sha256').update(corpus).digest('hex'),
  runtimeErrorPolicy: 'Original error name/message retained; Rust replay must compare phase/name and separately report any platform-message differences',
  sourceHashes };
await mkdir(path.join(root, 'results/sixel'), { recursive: true });
await writeFile(path.join(root, 'results/sixel/oracle.json'), JSON.stringify(report, null, 2) + '\n');
console.log(JSON.stringify(report, null, 2));
