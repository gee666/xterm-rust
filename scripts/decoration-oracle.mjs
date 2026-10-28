// MIT license. Execute unchanged upstream assertions and actual-JS cache traces.
import { createHash } from 'node:crypto';
import { mkdirSync, readFileSync, writeFileSync } from 'node:fs';
import { resolve } from 'node:path';
import { pathToFileURL } from 'node:url';
const generate = process.argv.includes('--generate');
if (process.argv.slice(2).some(argument => argument !== '--generate')) throw new Error('Usage: node scripts/decoration-oracle.mjs [--generate]');
const root = resolve(import.meta.dirname, '..');
const temporary = resolve(root, 'tmp/decoration-oracle');
const results = resolve(root, 'results/decoration');
mkdirSync(temporary, { recursive: true }); mkdirSync(results, { recursive: true });
const { build } = await import(pathToFileURL(resolve(root, 'source_repo/node_modules/esbuild/lib/main.js')));
const { default: Mocha } = await import(pathToFileURL(resolve(root, 'source_repo/node_modules/mocha/index.js')));
const hashes = {};
const digest = bytes => createHash('sha256').update(bytes).digest('hex');
async function bundle(entryPoints, outfile, stdin) {
  const built = await build({ absWorkingDir: root, entryPoints, stdin, outfile, bundle: true,
    platform: 'node', format: 'cjs', metafile: true, tsconfig: 'source_repo/src/common/tsconfig.json' });
  for (const path of Object.keys(built.metafile.inputs).filter(p => p.startsWith('source_repo/src/'))) {
    hashes[path] = digest(readFileSync(resolve(root, path)));
  }
}
const baseline = resolve(temporary, 'baseline.cjs');
await bundle(['source_repo/src/common/services/DecorationService.test.ts'], baseline);
const mocha = new Mocha({ reporter: 'spec' }); mocha.addFile(baseline); await mocha.loadFilesAsync();
const outcomes = []; let runner;
await new Promise(done => {
  runner = mocha.run(done);
  runner.on('pass', t => outcomes.push({ name: t.fullTitle(), status: 'passed' }));
  runner.on('fail', (t, e) => outcomes.push({ name: t.fullTitle(), status: 'failed', error: String(e.stack) }));
  runner.on('pending', t => outcomes.push({ name: t.fullTitle(), status: 'skipped' }));
});
const runtime = resolve(temporary, 'runtime.cjs');
await bundle(undefined, runtime, { contents: `
export { DecorationService } from './source_repo/src/common/services/DecorationService';
export { Buffer } from './source_repo/src/common/buffer/Buffer';
export { OptionsService } from './source_repo/src/common/services/OptionsService';
`, resolveDir: root, loader: 'ts' });
const { DecorationService, Buffer, OptionsService } = await import(pathToFileURL(runtime));
const options = new OptionsService();
const buffer = new Buffer(true, options, { cols: 80, rows: 24 });
const log = { debug() {}, info() {}, warn() {}, error() {}, trace() {}, logLevel: 0 };
const service = new DecorationService(log, { buffer, buffers: { onBufferActivate: () => ({ dispose() {} }) } });
const decorations = [];
const indexOf = d => decorations.indexOf(d);
function snapshot() {
  const states = decorations.map(d => [d.marker.line, +d.marker.isDisposed, +d.isDisposed].join(',')).join(';') || '-';
  const ordered = [...service.decorations].map(indexOf).join(',') || '-';
  const lookups = [];
  for (let line = 0; line < 20; line++) for (const x of [0, 2, 5, 8]) for (const layer of [undefined, 'bottom', 'top']) {
    lookups.push([...service.getDecorationsAtCell(x, line, layer)].map(indexOf).join(',') || '-');
  }
  return [states, ordered, lookups.join(';')].join('|');
}
let state = 0xdec0a710;
function random() { state ^= state << 13; state ^= state >>> 17; state ^= state << 5; return state >>> 0; }
const rows = [];
for (let i = 0; i < 1024; i++) {
  const op = random() % 7;
  const a = random() % 16; const b = random() % 4;
  switch (op) {
    case 0: {
      const marker = buffer.addMarker(a);
      decorations.push(service.registerDecoration({ marker, x: b, width: b + 1, height: b, layer: b % 2 ? 'top' : 'bottom' }));
      break;
    }
    case 1: if (decorations.length) decorations[a % decorations.length].dispose(); break;
    case 2: buffer.lines.onTrimEmitter.fire(b); break;
    case 3: buffer.lines.onInsertEmitter.fire({ index: a, amount: b }); break;
    case 4: buffer.lines.onDeleteEmitter.fire({ index: a, amount: b }); break;
    case 5: service.reset(); break;
    case 6: if (decorations.length) decorations[a % decorations.length].options.layer = b % 2 ? 'top' : 'bottom'; break;
  }
  await new Promise(resolve => queueMicrotask(resolve));
  rows.push([op, a, b, snapshot()].join('\t'));
}
const fixture = resolve(root, 'crates/xterm-core/tests/fixtures/decoration.tsv');
const fixtureText = rows.join('\n') + '\n';
let fixtureDrift = false;
if (generate) writeFileSync(fixture, fixtureText);
else {
  fixtureDrift = readFileSync(fixture, 'utf8') !== fixtureText;
  if (fixtureDrift) writeFileSync(resolve(temporary, 'actual.tsv'), fixtureText);
}
const report = { command: 'node scripts/decoration-oracle.mjs', node: process.version, platform: process.platform,
  scope: 'DecorationService only, original baseline and cache/marker traces. Browser rendering unrun.', overallEquivalence: 'NOT PROVEN',
  hashes, upstream: { discovered: runner.total, executed: runner.stats.tests - runner.stats.pending, passed: runner.stats.passes,
    failed: runner.stats.failures, skipped: runner.stats.pending, unrun: runner.total - runner.stats.tests, outcomes },
  seed: '0xdec0a710', operations: rows.length, fixtureSha256: digest(readFileSync(fixture)), generatedSha256: digest(fixtureText), fixtureDrift, mode: generate ? 'generate' : 'verify' };
writeFileSync(resolve(results, 'oracle.json'), JSON.stringify(report, null, 2) + '\n');
console.log(JSON.stringify({ upstream: report.upstream, operations: report.operations }, null, 2));
if (report.upstream.failed || report.upstream.skipped || report.upstream.unrun || fixtureDrift) process.exitCode = 1;
service.dispose();
