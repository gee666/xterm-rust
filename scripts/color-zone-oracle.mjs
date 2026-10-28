// MIT license. Separate process/bundle scope keeps the three original assertions intact.
import { createHash } from 'node:crypto';
import { mkdirSync, readFileSync, writeFileSync } from 'node:fs';
import { resolve } from 'node:path';
import { pathToFileURL } from 'node:url';
const generate = process.argv.includes('--generate');
if (process.argv.slice(2).some(arg => arg !== '--generate')) throw new Error('Usage: node scripts/color-zone-oracle.mjs [--generate]');
const root = resolve(import.meta.dirname, '..');
const temporary = resolve(root, 'tmp/color-zone-oracle'); const output = resolve(root, 'results/color-zone');
mkdirSync(temporary, { recursive: true }); mkdirSync(output, { recursive: true });
const { build } = await import(pathToFileURL(resolve(root, 'source_repo/node_modules/esbuild/lib/main.js')));
const { default: Mocha } = await import(pathToFileURL(resolve(root, 'source_repo/node_modules/mocha/index.js')));
const sha = value => createHash('sha256').update(value).digest('hex'); const hashes = {};
async function bundle(entry, output) {
  const result = await build({ absWorkingDir: root, entryPoints: [entry], outfile: output, bundle: true, platform: 'node', format: 'cjs', metafile: true, tsconfig: 'source_repo/src/browser/tsconfig.json' });
  for (const path of Object.keys(result.metafile.inputs).filter(p => p.startsWith('source_repo/src/'))) hashes[path] = sha(readFileSync(resolve(root, path)));
}
const baseline = resolve(temporary, 'baseline.cjs');
await bundle('source_repo/src/browser/decorations/ColorZoneStore.test.ts', baseline);
const mocha = new Mocha({ reporter: 'spec' }); mocha.addFile(baseline); await mocha.loadFilesAsync();
let runner; const outcomes = [];
await new Promise(done => { runner = mocha.run(done);
  runner.on('pass', t => outcomes.push({ name: t.fullTitle(), status: 'passed' }));
  runner.on('fail', (t, e) => outcomes.push({ name: t.fullTitle(), status: 'failed', error: String(e.stack) }));
  runner.on('pending', t => outcomes.push({ name: t.fullTitle(), status: 'skipped' }));
});
const runtime = resolve(temporary, 'runtime.cjs'); await bundle('source_repo/src/browser/decorations/ColorZoneStore.ts', runtime);
const { ColorZoneStore } = await import(pathToFileURL(runtime));
const store = new ColorZoneStore(); const retained = [];
const positions = [undefined, 'full', 'left', 'center', 'right']; const colors = ['red', 'blue', '\ud800', 'red'];
const ids = []; const identify = zone => { let id = ids.indexOf(zone); if (id < 0) { id = ids.length; ids.push(zone); } return id; };
const units = text => Array.from({ length: text.length }, (_, i) => text.charCodeAt(i).toString(16)).join(',') || '-';
function state() {
  const zones = store.zones;
  const active = zones.map(z => [identify(z), units(z.color), positions.indexOf(z.position), z.startBufferLine, z.endBufferLine].join(':')).join(';') || '-';
  const handles = retained.map(z => [identify(z), units(z.color), positions.indexOf(z.position), z.startBufferLine, z.endBufferLine].join(':')).join(';') || '-';
  return `${active}|${handles}`;
}
let seed = 0xc01020ae; const random = () => { seed ^= seed << 13; seed ^= seed >>> 17; seed ^= seed << 5; return seed >>> 0; };
const rows = [];
for (let i = 0; i < 2048; i++) {
  const op = random() % 5; const line = (random() % 40) - 10; const color = random() % colors.length; const position = random() % positions.length; const padding = random() % 5;
  switch (op) {
    case 0: store.addDecoration({ marker: { line }, options: { overviewRulerOptions: { color: colors[color], position: positions[position] } } }); break;
    case 1: store.clear(); store.addDecoration({ marker: { line }, options: { overviewRulerOptions: { color: colors[color], position: positions[position] } } }); break;
    case 2: store.setPadding({ full: padding, left: padding + 1, center: padding + 2, right: padding - 1 }); break;
    case 3: { const zone = store.zones[0]; if (zone) { if (retained.length === 16) retained.shift(); retained.push(zone); } break; }
    case 4: store.addDecoration({ marker: { line }, options: {} }); break;
  }
  rows.push([op, line, color, position, padding, state()].join('\t'));
}
const fixture = resolve(root, 'crates/xterm-web/tests/fixtures/color-zone.tsv'); const bytes = rows.join('\n') + '\n';
let fixtureDrift = false;
if (generate) writeFileSync(fixture, bytes);
else { fixtureDrift = readFileSync(fixture, 'utf8') !== bytes; if (fixtureDrift) writeFileSync(resolve(temporary, 'actual.tsv'), bytes); }
const report = { command: 'node scripts/color-zone-oracle.mjs', node: process.version, platform: process.platform, seed: '0xc01020ae', hashes,
  upstream: { discovered: runner.total, executed: runner.stats.tests - runner.stats.pending, passed: runner.stats.passes, failed: runner.stats.failures, skipped: runner.stats.pending, unrun: runner.total - runner.stats.tests, outcomes },
  operations: rows.length, fixtureSha256: sha(readFileSync(fixture)), generatedSha256: sha(bytes), fixtureDrift, mode: generate ? 'generate' : 'verify',
  scope: 'ColorZoneStore only; actual overview canvas/browser rendering separate', overallEquivalence: 'NOT PROVEN' };
writeFileSync(resolve(output, 'oracle.json'), JSON.stringify(report, null, 2) + '\n');
console.log(JSON.stringify(report, null, 2));
if (report.upstream.failed || report.upstream.skipped || report.upstream.unrun || fixtureDrift) process.exitCode = 1;
