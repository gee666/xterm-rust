// Copyright (c) 2017-2026 The xterm.js authors. All rights reserved. MIT license.
import assert from 'node:assert/strict';
import { createHash } from 'node:crypto';
import { mkdirSync, readFileSync, writeFileSync } from 'node:fs';
import { createRequire } from 'node:module';
import { delimiter, resolve } from 'node:path';
import { spawnSync } from 'node:child_process';
import { runScopedOracle, buildNativeOracle, recordSyncCommand } from './oracle-evidence.mjs';

const root = resolve(import.meta.dirname, '..'); process.chdir(root);
const attemptReport = await runScopedOracle(resolve('results/terminal-osc'), {
  kind: 'terminal-osc', command: 'node scripts/check-terminal-osc.mjs', scope: 'Terminal OSC only; full equivalence NOT PROVEN'
}, async context => {
const result = context.directory;
const temp = resolve('tmp/terminal-osc', context.runId);
mkdirSync(temp, { recursive: true });
context.checkpoint({ phase: 'source-build' });
const require = createRequire(resolve('source_repo/package.json'));
const { build } = require('esbuild');
const configuration = { bundle: true, format: 'cjs', platform: 'node', target: 'node24', packages: 'external',
  tsconfig: resolve('source_repo/src/common/tsconfig.json'), logLevel: 'error', nodePaths: [resolve('source_repo/node_modules')], metafile: true };
const runtime = resolve(temp, 'runtime.cjs'), baseline = resolve(temp, 'InputHandler.test.cjs');
const exports = [['InputHandler', 'InputHandler'], ['BufferService', 'services/BufferService'], ['OptionsService', 'services/OptionsService'],
  ['CoreService', 'services/CoreService'], ['CharsetService', 'services/CharsetService'], ['UnicodeService', 'services/UnicodeService'],
  ['OscLinkService', 'services/OscLinkService'], ['UnicodeV6', 'input/UnicodeV6'], ['MockLogService, MockMouseStateService', 'TestUtils.test'], ['CellData', 'buffer/CellData']];
const compiled = await build({ ...configuration, outfile: runtime, stdin: {
  contents: exports.map(([name, path]) => `export { ${name} } from './source_repo/src/common/${path}.ts';`).join('\n'), resolveDir: root, loader: 'ts' } });
const testBuild = await build({ ...configuration, outfile: baseline, entryPoints: ['source_repo/src/common/InputHandler.test.ts'] });
const env = { ...process.env, NODE_PATH: [resolve('source_repo/node_modules'), resolve('source_repo/out-esbuild'), process.env.NODE_PATH].filter(Boolean).join(delimiter) };
const original = spawnSync(process.execPath, ['source_repo/node_modules/mocha/bin/mocha.js', baseline, '--grep', '^InputHandler OSC ', '--reporter=json'], { encoding: 'utf8', timeout: 120000, env });
writeFileSync(`${result}/original-stdout.json`, original.stdout ?? ''); writeFileSync(`${result}/original-stderr.log`, original.stderr ?? '');
recordSyncCommand(context, [process.execPath, 'source_repo/node_modules/mocha/bin/mocha.js', baseline, '--grep', '^InputHandler OSC ', '--reporter=json'], original,
  { stdout: `${result}/original-stdout.json`, stderr: `${result}/original-stderr.log` });
if (original.error) throw original.error; assert.equal(original.status, 0, original.stderr);
context.checkpoint({ phase: 'original-output-parse' });
const report = JSON.parse(original.stdout);
assert.equal(report.stats.tests, 12); assert.equal(report.stats.passes, 12); assert.equal(report.stats.pending, 0); assert.equal(report.stats.failures, 0);
const sourceTests = readFileSync('source_repo/src/common/InputHandler.test.ts', 'utf8');
const start = sourceTests.indexOf("  describe('OSC',");
const end = sourceTests.indexOf('  // issue #3362', start);
assert(start > 0 && end > start);
const originalAssertions = (sourceTests.slice(start, end).match(/assert\./g) ?? []).length;
const { InputHandler, BufferService, OptionsService, CoreService, CharsetService, UnicodeService, OscLinkService, MockLogService, MockMouseStateService, CellData, UnicodeV6 } = require(runtime);
function hex(text) { return text.length ? Array.from({ length: text.length }, (_, i) => text.charCodeAt(i).toString(16).padStart(4, '0')).join('') : '_'; }
function unhex(text) { return text === '_' ? '' : String.fromCharCode(...text.match(/.{4}/g).map(unit => parseInt(unit, 16))); }
function colorBatch(batch) { return 'c[' + batch.map(request => request.type === 0 ? `r${request.index},` : request.type === 1 ? `s${request.index}:${request.color.join('.')},` : `x${request.index ?? '*'},`).join('') + ']'; }
let buffers, links, input, events;
function reset() {
  const options = new OptionsService({ cols: 4, rows: 3, scrollback: 2 });
  const log = new MockLogService();
  buffers = new BufferService(options, log); links = new OscLinkService(buffers);
  const unicode = new UnicodeService(); unicode.register(new UnicodeV6()); unicode.activeVersion = '6';
  input = new InputHandler(buffers, new CharsetService(), new CoreService(buffers, log, options), log, options, links, new MockMouseStateService(), unicode);
  events = [];
  input.onColor(batch => events.push(colorBatch(batch)));
  input.onTitleChange(title => events.push(`t${hex(title)}:${hex(input._windowTitle)}:${hex(input._iconName)}`));
}
function snapshot() {
  const attr = input._curAttrData;
  let out = `${buffers.buffer === buffers.buffers.normal ? 'n' : 'a'};${hex(input._windowTitle)};${hex(input._iconName)};${attr.fg >>> 0},${attr.bg >>> 0},${attr.extended.ext >>> 0},${attr.extended.urlId};${events.join('|')};`;
  events = [];
  for (const buffer of [buffers.buffers.normal, buffers.buffers.alt]) {
    out += `${buffer.x},${buffer.y},${buffer.ybase},${buffer.ydisp},${buffer.scrollTop},${buffer.scrollBottom}/`;
    for (let row = 0; row < buffer.lines.length; row++) {
      const line = buffer.lines.get(row); out += `${+line.isWrapped}[`;
      for (let column = 0; column < line.length; column++) {
        const cell = line.loadCell(column, new CellData());
        out += `${cell.content >>> 0},${cell.fg >>> 0},${cell.bg >>> 0},${cell.extended.ext >>> 0},${cell.extended.urlId},${hex(cell.getChars())}|`;
      }
      out += ']';
    }
    out += 'm' + buffer.markers.map(marker => `${marker.id}:${marker.line}:${+marker.isDisposed},`).join('') + ';';
  }
  for (const id of [...links._dataByLinkId.keys()].sort((a, b) => a - b)) {
    const data = links.getLinkData(id);
    if (data) out += `${id}:${data.id === undefined ? '-' : hex(data.id)}/${hex(data.uri)}|`;
  }
  return out;
}
reset();
const operations = [], expected = [];
async function step(operation) {
  if (operation === 'reset') reset();
  else {
    let previous; let pending;
    const text = unhex(operation);
    while ((pending = input.parse(text, previous))) previous = await pending;
  }
  operations.push(operation); expected.push(snapshot());
}
const directed = [
  '\x1b]1;old\x07', '\x1b]0;new\x07', '\x1b]2;\x07', '\x1b]2;\x07', '\x1b]2;\ud800X\udc00\x07',
  '\x1b]4;000;?;255;#fff;256;?;-1;?; 1;?;１２;?;2;?;99\x07',
  '\x1b]10;bad;?;#fff;?\x07', '\x1b]11;?;?;?\x07', '\x1b]12;?;?\x07', '\x1b]104\x07', '\x1b]104;;0; 1;255;255\x07', '\x1b]110;ignored\x07', '\x1b]111\x07', '\x1b]112\x07',
  '\x1b]8;id=a;uri;a\x07abcdX', 'Y', '\xad', '\x1b]8;nonempty;\x07', '\x1b]8;malformed\x07', '\x1b]8;;\x07', '\r\n\r\n\r\n',
  '\x1b]8;id=;uri\x07', '\x1b]8;id=a:id=b;uri\x07A', '\x1b]8;;next\x07好😀', '\x1b]8;;\x07', '\x1b[?1049h', '\x1b]8;;alternate\x07AB', '\x1b[?1049l',
];
for (let i = 0; i < directed.length; i++) { if (i % 16 === 0) await step('reset'); await step(hex(directed[i])); }
const directedOperations = operations.length;
const seed = 0x4f534354; let state = seed;
function random() { state ^= state << 13; state ^= state >>> 17; state ^= state << 5; return state >>> 0; }
const fragments = [
  '\x1b]0;name\x07', '\x1b]1;icon\x1b\\', '\x1b]2;😀\ud800\x07', '\x1b]4;0;?;255;#123\x07', '\x1b]4;+1;?;1;rgb:1/2/3;0;bad\x07',
  '\x1b]10;?;#abc;?\x1b\\', '\x1b]10;;?;bad\x07', '\x1b]11;?;?\x07', '\x1b]12;?\x07', '\x1b]104;0;0;256;-1\x07', '\x1b]110\x07', '\x1b]111\x07', '\x1b]112\x07',
  '\x1b]8;id=a;uri;a\x07', '\x1b]8;id=;uri\x07', '\x1b]8;;uri\x07', '\x1b]8;;\x07', '\x1b]8;id=a:id=b;next\x07', '\x1b]8;bad;\x07',
  'AB', 'x', '好', '😀', '\u0301', '\xad', '\r\n', '\x1b[H', '\x1b[3;4H', '\x1b[?1049h', '\x1b[?1049l', '\x1b]999;ignored\x07', '',
];
for (let i = 0; i < 512; i++) {
  if (i % 16 === 0) await step('reset');
  const text = fragments[random() % fragments.length];
  if (text.length && random() % 4 === 0) { const split = random() % (text.length + 1); await step(hex(text.slice(0, split))); await step(hex(text.slice(split))); }
  else await step(hex(text));
}
const fixture = operations.map((operation, i) => `${operation}\t${expected[i]}`).join('\n') + '\n';
const fixturePath = 'crates/xterm-core/tests/fixtures/terminal-osc-traces.tsv';
if (process.argv.includes('--write-fixtures')) writeFileSync(fixturePath, fixture);
else assert.equal(readFileSync(fixturePath, 'utf8').replaceAll('\r\n', '\n'), fixture);
context.checkpoint({ originalCases: report.stats.tests, originalPassed: report.stats.passes, originalAssertions, phase: 'native-build' });
const artifact = await buildNativeOracle(context, root, 'terminal_osc_oracle');
const native = spawnSync(artifact.executable, [], { encoding: 'utf8', timeout: 120000, maxBuffer: 32 * 1024 * 1024, input: operations.join('\n') + '\n' });
writeFileSync(`${result}/native-stderr.log`, native.stderr ?? '');
recordSyncCommand(context, [artifact.executable], native, { stderr: `${result}/native-stderr.log` });
if (native.error) throw native.error; assert.equal(native.status, 0, native.stderr);
context.checkpoint({ phase: 'native-output-comparison' });
const actual = native.stdout.trimEnd().split(/\r?\n/); assert.equal(actual.length, expected.length);
for (let i = 0; i < actual.length; i++) if (actual[i] !== expected[i]) {
  writeFileSync(`${result}/counterexample.json`, JSON.stringify({ seed, index: i, prefix: operations.slice(0, i + 1), expected: expected[i], actual: actual[i] }, null, 2));
  assert.fail(`Terminal OSC mismatch at${i}; see counterexample.json`);
}
const hashes = {};
for (const path of [...new Set([...Object.keys(compiled.metafile.inputs), ...Object.keys(testBuild.metafile.inputs)])].sort()) if (path.startsWith('source_repo/')) hashes[path] = createHash('sha256').update(readFileSync(path)).digest('hex');
const summary = { scope: 'actual terminal OSC events/cells/row/cursor/marker/link snapshots; browser processing NOT PROVEN', command: 'node scripts/check-terminal-osc.mjs', platform: process.platform, node: process.version, rustc: artifact.rustc,
  originalCases: report.stats.tests, originalPassed: report.stats.passes, originalAssertions, seed, directedOperations, randomFragments: 512, compared: actual.length, mismatches: 0,
  fixtureSha256: createHash('sha256').update(fixture).digest('hex'), hashes, status: 'passed' };
return summary;
});
console.log(JSON.stringify(attemptReport, null, 2));
