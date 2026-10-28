// MIT license. Observe raw marker keys through the unchanged source service/buffer.
import assert from 'node:assert/strict';
import {createHash} from 'node:crypto';
import {mkdirSync, readFileSync, writeFileSync} from 'node:fs';
import {resolve} from 'node:path';
import {pathToFileURL} from 'node:url';
const root = resolve(import.meta.dirname, '..');
const source = resolve(process.env.XTERM_SOURCE_ROOT ?? resolve(root, 'source_repo'));
const generate = process.argv.includes('--generate');
assert(process.argv.slice(2).every(arg => arg === '--generate'), 'Usage: decoration-raw-oracle.mjs [--generate]');
const temporary = resolve(root, 'tmp/decoration-raw');
mkdirSync(temporary, {recursive: true});
const {build} = await import(pathToFileURL(resolve(source, 'node_modules/esbuild/lib/main.js')));
const bundle = resolve(temporary, 'source.cjs');
const built = await build({stdin: {contents: `
export {DecorationService} from ${JSON.stringify(resolve(source, 'src/common/services/DecorationService.ts'))};
export {Buffer} from ${JSON.stringify(resolve(source, 'src/common/buffer/Buffer.ts'))};
export {OptionsService} from ${JSON.stringify(resolve(source, 'src/common/services/OptionsService.ts'))};
`, resolveDir: root, loader: 'ts'}, bundle: true, platform: 'node', format: 'cjs', outfile: bundle,
  tsconfig: resolve(source, 'src/common/tsconfig.json'), metafile: true});
const {DecorationService, Buffer, OptionsService} = await import(pathToFileURL(bundle));
const log = {debug() {}, info() {}, warn() {}, error() {}, trace() {}, logLevel: 0};
function encode(value) {
  if (typeof value === 'number') return `n${Object.is(value, -0) ? '-0' : String(value)}`;
  return 's' + (Array.from({length: value.length}, (_, i) => value.charCodeAt(i).toString(16).padStart(4, '0')).join('') || '_');
}
const queries = ['', '0', '1', '2', '10', 'a', 'Infinity', '\ud800', NaN, -0, ...Array.from({length: 102}, (_, i) => i - 1),
  0.5, 1.5, 2.5, 2 ** 53 - 1, 2 ** 53, 2 ** 53 + 2];
const cases = [
  ['', 1], ['2', 1], ['2', 0], ['2', 3], ['02', 1], ['a', 1], ['\ud800', 1], ['Infinity', 1], ['-1', 1], ['10', 0],
  [NaN, 1], [Infinity, 1], [-Infinity, 1], [-0, 1], [0.5, 3], [2 ** 53, 1], [2 ** 53 - 1, 2],
];
let seed = 0xdec064ab;
function random() { seed ^= seed << 13; seed ^= seed >>> 17; seed ^= seed << 5; return seed >>> 0; }
for (let i = 0; i < 64; i++) {
  const number = random() % 10 - 2;
  const representation = random() % 5;
  const line = representation === 0 ? number + 0.5 : representation === 1 ? String(number)
    : representation === 2 ? ` ${number}` : representation === 3 ? `0${Math.abs(number)}`
    : String.fromCharCode(0xd800 + random() % 0x800);
  cases.push([line, random() % 4]);
}
const rows = [];
for (const [line, height] of cases) {
  const buffer = new Buffer(true, new OptionsService(), {cols: 80, rows: 24});
  const service = new DecorationService(log, {buffer, buffers: {onBufferActivate: () => ({dispose() {}})}});
  const marker = buffer.addMarker(line);
  const decoration = service.registerDecoration({marker, height});
  const snapshot = () => [encode(marker.line), +marker.isDisposed, +decoration.isDisposed,
    [...service.decorations].length, queries.map(query => [...service.getDecorationsAtCell(0, query)].length).join(',')].join('|');
  const capture = step => rows.push([encode(line), height, step, snapshot()].join('\t'));
  capture('add');
  buffer.lines.onInsertEmitter.fire({index: 1, amount: 2});
  await new Promise(done => queueMicrotask(done)); capture('insert');
  buffer.lines.onTrimEmitter.fire(1); capture('trim');
  buffer.lines.onDeleteEmitter.fire({index: 2, amount: 1});
  await new Promise(done => queueMicrotask(done)); capture('delete');
  decoration.dispose(); capture('dispose');
  service.dispose();
}
const mergeBuffer = new Buffer(true, new OptionsService(), {cols: 80, rows: 24});
const mergeService = new DecorationService(log, {buffer: mergeBuffer, buffers: {onBufferActivate: () => ({dispose() {}})}});
const merged = ['a', 'b'].map(line => mergeService.registerDecoration({marker: mergeBuffer.addMarker(line)}));
const mergedIds = query => [...mergeService.getDecorationsAtCell(0, query)].map(item => merged.indexOf(item));
assert.deepEqual(mergedIds('a'), [0]); assert.deepEqual(mergedIds('b'), [1]);
mergeBuffer.lines.onTrimEmitter.fire(1);
const nanMerge = mergedIds(NaN);
assert.deepEqual(nanMerge, [0, 1]);
mergeBuffer.lines.onTrimEmitter.fire(1);
assert.deepEqual(mergedIds(NaN), [0, 1]);
mergeService.dispose();
const fixture = resolve(root, 'crates/xterm-core/tests/fixtures/decoration-raw.tsv');
const expected = rows.join('\n') + '\n';
if (generate) writeFileSync(fixture, expected);
const actual = readFileSync(fixture, 'utf8');
const drift = actual !== expected;
if (drift) writeFileSync(resolve(temporary, 'actual.tsv'), expected);
const digest = bytes => createHash('sha256').update(bytes).digest('hex');
const hashes = Object.fromEntries(Object.keys(built.metafile.inputs).filter(path => path.endsWith('.ts'))
  .map(path => [path.replaceAll('\\', '/').slice(path.replaceAll('\\', '/').lastIndexOf('/src/') + 1), digest(readFileSync(path))]));
const results = resolve(root, 'results/decoration-raw');
mkdirSync(results, {recursive: true});
const report = {command: 'node scripts/decoration-raw-oracle.mjs', node: process.version, platform: process.platform,
  cases: cases.length, seed: '0xdec064ab', randomizedCases: 64, snapshots: rows.length, queries: queries.map(encode), nanMerge, hashes, fixtureSha256: digest(actual), drift,
  scope: 'Raw Number/String marker cache and buffer events. Integer heights only. Source non-progress spans excluded; browser DOM and engine-dependent batch sorting unproved.',
  nonprogressCase: {line: 2 ** 53 + 2, height: 3, operations: ['insert1,2', 'trim1', 'delete2,1', 'dispose'],
    observation: 'Separate actual-source diagnostic reached dispose at line9007199254740996 but never returned before10-second command timeout. Not included among completed snapshots.'}};
writeFileSync(resolve(results, 'source.json'), JSON.stringify(report, null, 2) + '\n');
console.log(JSON.stringify({cases: cases.length, snapshots: rows.length, queries: queries.length, drift}));
assert.equal(drift, false, 'raw decoration fixture differs from source');
