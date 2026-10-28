// Copyright (c) 2020-2022 The xterm.js authors. All rights reserved. MIT license.
import assert from 'node:assert/strict';
import { createHash } from 'node:crypto';
import { mkdirSync, readFileSync, writeFileSync } from 'node:fs';
import { createRequire } from 'node:module';
import { delimiter, resolve } from 'node:path';
import { spawnSync } from 'node:child_process';

const root = resolve(import.meta.dirname, '..');
process.chdir(root);
const temp = resolve('tmp/osc-link-service');
const result = 'results/osc-link-service';
mkdirSync(temp, { recursive: true }); mkdirSync(result, { recursive: true });
const require = createRequire(resolve('source_repo/package.json'));
const { build } = require('esbuild');
const runtime = resolve(temp, 'runtime.cjs');
const baseline = resolve(temp, 'OscLinkService.test.cjs');
const configuration = {
  bundle: true, format: 'cjs', platform: 'node', target: 'node24', packages: 'external',
  tsconfig: resolve('source_repo/src/common/tsconfig.json'), logLevel: 'error',
  nodePaths: [resolve('source_repo/node_modules')], metafile: true,
};
const exports = [
  ['OscLinkService', 'services/OscLinkService'], ['BufferService', 'services/BufferService'],
  ['OptionsService', 'services/OptionsService'], ['AttributeData', 'buffer/AttributeData'],
];
const compiled = await build({ ...configuration, outfile: runtime, stdin: {
  contents: exports.map(([name, file]) => `export { ${name} } from './source_repo/src/common/${file}.ts';`).join('\n'),
  resolveDir: root, loader: 'ts',
} });
const tests = await build({ ...configuration, outfile: baseline,
  entryPoints: ['source_repo/src/common/services/OscLinkService.test.ts'] });
const env = { ...process.env, NODE_PATH: [resolve('source_repo/node_modules'), resolve('source_repo/out-esbuild'), process.env.NODE_PATH].filter(Boolean).join(delimiter) };
const runBaseline = spawnSync(process.execPath,
  ['source_repo/node_modules/mocha/bin/mocha.js', baseline, '--reporter=json'],
  { encoding: 'utf8', timeout: 120000, env });
writeFileSync(`${result}/original-stdout.json`, runBaseline.stdout ?? '');
writeFileSync(`${result}/original-stderr.log`, runBaseline.stderr ?? '');
if (runBaseline.error) throw runBaseline.error;
assert.equal(runBaseline.status, 0, runBaseline.stderr);
const baselineReport = JSON.parse(runBaseline.stdout);
assert.equal(baselineReport.stats.tests, 3); assert.equal(baselineReport.stats.passes, 3);
assert.equal(baselineReport.stats.failures, 0); assert.equal(baselineReport.stats.pending, 0);
const { OscLinkService, BufferService, OptionsService, AttributeData } = require(runtime);
const buffers = new BufferService(new OptionsService({ cols: 10, rows: 3, scrollback: 2 }), { debug() {} });
const links = new OscLinkService(buffers);
const data = [], ids = new Set(), markers = [], operations = [], expected = [];
function decode(value) {
  if (value === '_') return '';
  return String.fromCharCode(...value.match(/.{4}/g).map(unit => parseInt(unit, 16)));
}
function encode(value) {
  if (!value.length) return '_';
  return Array.from({ length: value.length }, (_, i) => value.charCodeAt(i).toString(16).padStart(4, '0')).join('');
}
function object(id, uri) { return { ...(id === '-' ? {} : { id: decode(id) }), uri: decode(uri) }; }
function step(operation) {
  const [op, a, b, c] = operation.split(' ');
  let reply = '-';
  switch (op) {
    case 'data': data.push(object(a, b)); break;
    case 'mut': data[+a].id = b === '-' ? undefined : decode(b); data[+a].uri = decode(c); break;
    case 'reg': reply = links.registerLink(data[+a]); ids.add(reply); break;
    case 'line': links.addLineToLink(+a, +b); break;
    case 'cursor': buffers.buffer.ybase = +a; buffers.buffer.y = +b; break;
    case 'alt': buffers.buffers.activateAltBuffer(); break;
    case 'normal': buffers.buffers.activateNormalBuffer(); break;
    case 'scroll': buffers.scroll(new AttributeData()); break;
    case 'dispose': markers[+a]?.dispose(); break;
    case 'move': if (markers[+a]) markers[+a].line = +b; break;
    default: throw new Error(`Unknown operation ${op}`);
  }
  for (const buffer of [buffers.buffers.normal, buffers.buffers.alt]) {
    for (const marker of buffer.markers) if (!markers.includes(marker)) markers.push(marker);
  }
  let snapshot = `${reply};${buffers.buffer === buffers.buffers.normal ? 'n' : 'a'};`;
  for (const id of [...ids].sort((a, b) => a - b)) {
    const entry = links.getLinkData(id);
    snapshot += `${id}:${entry ? `${entry.id === undefined ? '-' : encode(entry.id)}/${encode(entry.uri)}` : '-'},`;
  }
  snapshot += ';' + markers.map(marker => `${marker.line}:${+marker.isDisposed},`).join('');
  operations.push(operation); expected.push(snapshot);
}
for (const operation of [
  'data 0066006f006f 006200610072', 'data - 006200610072', 'data _ _',
  'data 0061003b003b0062 0063', 'data 0061 0062003b003b0063', 'data d800 dc000000d83dde00',
  'reg 0', 'reg 0', 'reg 1', 'reg 1', 'reg 2', 'reg 2', 'reg 3', 'reg 4', 'reg 5',
  'cursor 0 2', 'reg 0', 'line 1 2', 'move 0 8', 'line 1 8', 'dispose 0',
  'mut 0 - 006200610072', 'dispose 7', 'data 0066006f006f 006200610072', 'reg 6',
  'alt', 'reg 6', 'scroll', 'normal', 'line 999 -1', 'line 3 -1',
]) step(operation);
const directedOperations = operations.length;
const seed = 0x4f53434c;
let state = seed;
function random() { state ^= state << 13; state ^= state >>> 17; state ^= state << 5; return state >>> 0; }
const strings = ['-', '_', '0061', '0062', '0061003b003b0062', 'd800', 'dc00', 'd83dde00'];
for (let i = 0; i < 1200; i++) {
  const choice = random() % 10;
  if (choice < 3) step(`reg ${random() % data.length}`);
  else if (choice === 3) step(`line ${1 + random() % Math.max(ids.size, 1)} ${random() % 8 - 1}`);
  else if (choice === 4) step(`dispose ${random() % Math.max(markers.length, 1)}`);
  else if (choice === 5) step(`move ${random() % Math.max(markers.length, 1)} ${random() % 8 - 1}`);
  else if (choice === 6) step(`cursor ${buffers.buffer.ybase} ${random() % 3}`);
  else if (choice === 7) step(random() % 2 ? 'alt' : 'normal');
  else if (choice === 8) step('scroll');
  else step(`mut ${random() % data.length} ${strings[random() % strings.length]} ${strings[1 + random() % (strings.length - 1)]}`);
}
// This otherwise unreachable test boundary is observable in source because its
// number counter is mutable. It also checks old callbacks after map overwrite.
const limitBuffers = new BufferService(new OptionsService({ cols: 10, rows: 3 }), { debug() {} });
const limitLinks = new OscLinkService(limitBuffers);
limitLinks._nextId = 2 ** 53;
const limitData = { uri: '' };
const limitId = limitLinks.registerLink({ uri: '' });
assert.equal(limitLinks.registerLink(limitData), limitId);
assert.equal(limitLinks.getLinkData(limitId), limitData);
const oldMarker = limitBuffers.buffer.markers[0], secondMarker = limitBuffers.buffer.markers[1];
oldMarker.dispose();
assert.equal(limitLinks.getLinkData(limitId), undefined);
const thirdData = { uri: 'third' };
assert.equal(limitLinks.registerLink(thirdData), limitId);
assert.equal(limitLinks.getLinkData(limitId), thirdData);
secondMarker.dispose();
assert.equal(limitLinks.getLinkData(limitId), undefined);
const fixture = operations.map((operation, i) => `${operation}\t${expected[i]}`).join('\n') + '\n';
const fixturePath = 'crates/xterm-core/tests/fixtures/osc-link-traces.tsv';
if (process.argv.includes('--write-fixtures')) writeFileSync(fixturePath, fixture);
else assert.equal(readFileSync(fixturePath, 'utf8').replaceAll('\r\n', '\n'), fixture);
const cargo = spawnSync(process.env.CARGO ?? 'cargo', ['run', '--quiet', '-p', 'xterm-core', '--example', 'osc_link_oracle'], {
  input: operations.join('\n') + '\n', encoding: 'utf8', timeout: 120000, maxBuffer: 32 * 1024 * 1024,
});
writeFileSync(`${result}/native-stderr.log`, cargo.stderr ?? '');
if (cargo.error) throw cargo.error;
assert.equal(cargo.status, 0, cargo.stderr);
const actual = cargo.stdout.trimEnd().split(/\r?\n/);
assert.equal(actual.length, expected.length);
for (let i = 0; i < actual.length; i++) {
  if (actual[i] !== expected[i]) {
    writeFileSync(`${result}/counterexample.json`, JSON.stringify({ seed, index: i, prefix: operations.slice(0, i + 1), expected: expected[i], actual: actual[i] }, null, 2));
    assert.fail(`OSC link mismatch at ${i}, see counterexample.json`);
  }
}
const inputs = new Set([...Object.keys(compiled.metafile.inputs), ...Object.keys(tests.metafile.inputs)]);
const hashes = {};
for (const input of [...inputs].sort()) if (input.startsWith('source_repo/')) hashes[input] = createHash('sha256').update(readFileSync(input)).digest('hex');
const rustc = spawnSync(process.env.RUSTC ?? 'rustc', ['--version'], { encoding: 'utf8', timeout: 10000 });
if (rustc.error) throw rustc.error;
assert.equal(rustc.status, 0, rustc.stderr);
const summary = {
  scope: 'OSCLinkService and actual BufferService markers only; OSC8 terminal/browser integration NOT PROVEN',
  command: 'node scripts/check-osc-link-service.mjs', platform: process.platform, node: process.version,
  rustc: rustc.stdout.trim(), esbuild: require('esbuild/package.json').version,
  mocha: require('mocha/package.json').version,
  baselineCases: baselineReport.stats.tests, baselinePassed: baselineReport.stats.passes,
  baselineAssertions: (readFileSync('source_repo/src/common/services/OscLinkService.test.ts', 'utf8').match(/assert\./g) ?? []).length,
  seed, directedOperations, randomizedOperations: 1200, compared: actual.length, mismatches: 0,
  numericLimitAssertions: 6,
  fixtureSha256: createHash('sha256').update(fixture).digest('hex'), hashes, status: 'passed',
};
writeFileSync(`${result}/summary.json`, JSON.stringify(summary, null, 2) + '\n');
console.log(JSON.stringify(summary, null, 2));
