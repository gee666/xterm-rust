// Copyright (c) 2026 xterm-rust contributors. Licensed under MIT.
// Reuses buffer-baseline.mjs esbuild/Mocha method; source and assertions untouched.
import assert from 'node:assert/strict';
import { createHash } from 'node:crypto';
import { mkdirSync, readFileSync, writeFileSync } from 'node:fs';
import { resolve } from 'node:path';
import { pathToFileURL } from 'node:url';
import { createRequire } from 'node:module';
const root = resolve(import.meta.dirname, '..');
const dependencies = resolve(root, 'source_repo/node_modules');
const { build } = await import(pathToFileURL(resolve(dependencies, 'esbuild/lib/main.js')));
const { default: Mocha } = await import(pathToFileURL(resolve(dependencies, 'mocha/index.js')));
const temporary = resolve(root, 'tmp/buffer-service-oracle');
const output = resolve(root, 'results/buffer-service');
mkdirSync(temporary, { recursive: true }); mkdirSync(output, { recursive: true });
const sourceHashes = {};
const buildOptions = { absWorkingDir: root, bundle: true, platform: 'node', format: 'cjs',
  metafile: true, tsconfig: 'source_repo/src/common/tsconfig.json' };
function hashes(built) {
  for (const file of Object.keys(built.metafile.inputs).filter(p => p.startsWith('source_repo/src/'))) {
    sourceHashes[file] = createHash('sha256').update(readFileSync(resolve(root, file))).digest('hex');
  }
}
const suites = [];
for (const path of ['buffer/BufferSet', 'services/BufferService']) {
  const tests = `source_repo/src/common/${path}.test.ts`;
  const bundle = resolve(temporary, `${path.split('/')[1]}.test.cjs`);
  hashes(await build({ ...buildOptions, entryPoints: [tests], outfile: bundle }));
  const mocha = new Mocha({ reporter: 'dot' }); mocha.addFile(bundle); await mocha.loadFilesAsync();
  const outcomes = []; let runner;
  const failed = await new Promise(done => {
    runner = mocha.run(done);
    runner.on('pass', test => outcomes.push({ name: test.fullTitle(), status: 'passed' }));
    runner.on('pending', test => outcomes.push({ name: test.fullTitle(), status: 'skipped' }));
    runner.on('fail', (test, error) => outcomes.push({ name: test.fullTitle(), status: 'failed', error: String(error.stack) }));
  });
  suites.push({ tests, discovered: runner.total, executed: runner.stats.tests - runner.stats.pending,
    passed: runner.stats.passes, failed, skipped: runner.stats.pending, unrun: runner.total - runner.stats.tests, outcomes });
}
assert(suites.every(s => s.failed === 0 && s.skipped === 0 && s.unrun === 0));
const apiFile = resolve(temporary, 'api.cjs');
hashes(await build({ ...buildOptions, stdin: { contents: `
export { BufferService } from './source_repo/src/common/services/BufferService';
export { OptionsService } from './source_repo/src/common/services/OptionsService';
export { AttributeData } from './source_repo/src/common/buffer/AttributeData';
export { MockLogService } from './source_repo/src/common/TestUtils.test';
`, resolveDir: root, loader: 'ts' }, outfile: apiFile }));
const { BufferService, OptionsService, AttributeData, MockLogService } = createRequire(import.meta.url)(apiFile);
const directedChecks = [];
{
  const service = new BufferService(new OptionsService({cols:10,rows:5,scrollback:4}), new MockLogService());
  const marker = service.buffer.addMarker(0);
  marker.onDispose(() => { throw new Error('intentional marker failure'); });
  assert.throws(() => service.dispose(), /intentional marker failure/);
  assert.equal(marker.isDisposed, true);
  const altMarker = service.buffers.alt.addMarker(0);
  service.dispose(); assert.equal(altMarker.isDisposed, false);
  directedChecks.push('dispose failure marks owner before callbacks; repeated dispose does not resume');
}
{
  const service = new BufferService(new OptionsService({cols:10,rows:5,scrollback:4}), new MockLogService());
  const set = service.buffers; set.dispose(); const old = set.normal; const marker = old.addMarker(0);
  set.reset(); assert.equal(marker.isDisposed, false); assert.notEqual(old, set.normal);
  directedChecks.push('reset after disposal replaces raw fields without disposing old buffer again');
}
const units = text => Array.from({ length: text.length }, (_, i) => text.charCodeAt(i).toString(16)).join(',') || '-';
function bufferSnapshot(buffer) {
  const rows = [];
  for (let row = 0; row < buffer.lines.length; row++) {
    const line = buffer.lines.get(row); const cells = [];
    for (let col = 0; col < line.length; col++) {
      const cell = line.getCell(col);
      cells.push([cell.content >>> 0, cell.fg >>> 0, cell.bg >>> 0, units(cell.getChars())].join(':'));
    }
    rows.push(`${+line.isWrapped};${cells.join('/')}`);
  }
  return [buffer.x, buffer.y, buffer.ybase, buffer.ydisp, buffer.scrollTop, buffer.scrollBottom,
    buffer.lines.length, buffer.lines.maxLength, +buffer.hasScrollback,
    Object.keys(buffer.tabs).filter(key => buffer.tabs[key]).map(Number).sort((a,b) => a-b).join(','), rows.join('|')].join('~');
}
const reentrantRows = [];
function checkpoint(service, tag) {
  const set = service.buffers;
  const kind = service.buffer === set.normal ? 'n' : service.buffer === set.alt ? 'a' : 'old';
  return [tag, service.cols, service.rows, +service.isUserScrolling, kind,
    bufferSnapshot(set.normal), bufferSnapshot(set.alt), bufferSnapshot(service.buffer)].join('#');
}
function fullService() {
  const service = new BufferService(new OptionsService({cols:4,rows:3,scrollback:1}),new MockLogService());
  while (!service.buffer.lines.isFull) service.scroll(new AttributeData());
  for (let row=0;row<service.buffer.lines.length;row++) service.buffer.lines.get(row).setCellFromCodepoint(0,65+row,1,new AttributeData());
  return service;
}
{
  const service=fullService(); const buffer=service.buffer; const out=[];
  const save=tag=>out.push(checkpoint(service,tag)); const marker=buffer.addMarker(0);
  marker.onDispose(()=>{ save('marker'); buffer.x=2; buffer.ydisp=0; service.isUserScrolling=true; save('changed'); });
  service.onScroll(()=>save('scroll')); service.scroll(new AttributeData()); save('final');
  assert.equal(marker.line,-1); assert.equal(marker.isDisposed,true);
  reentrantRows.push(['trim-read-write',out.join('^')].join('\t'));
}
{
  const service=fullService(); const out=[]; const save=tag=>out.push(checkpoint(service,tag)); let nested=false;
  service.buffer.lines.onTrim(()=>{ save('trim'); if (!nested) { nested=true; service.scroll(new AttributeData(),true); save('nested-return'); } });
  service.onScroll(()=>save('scroll')); service.scroll(new AttributeData()); save('final');
  reentrantRows.push(['nested-scroll',out.join('^')].join('\t'));
}
{
  const service=new BufferService(new OptionsService({cols:4,rows:3,scrollback:1}),new MockLogService());
  service.buffers.activateAltBuffer(); const alt=service.buffers.alt; alt.x=3; alt.y=1;
  const out=[]; const save=tag=>out.push(checkpoint(service,tag)); const marker=alt.addMarker(1);
  marker.onDispose(()=>{ save('marker'); alt.x=8; service.buffers.normal.y=2;
    const added=alt.addMarker(2); added.onDispose(()=>save('added-marker')); save('changed'); });
  service.onScroll(()=>save('scroll')); service.buffers.activateNormalBuffer(); save('final');
  assert.equal(alt.markers.length,0);
  reentrantRows.push(['activation-marker',out.join('^')].join('\t'));
}
{
  const service=new BufferService(new OptionsService({cols:4,rows:3,scrollback:1}),new MockLogService());
  const old=service.buffer; const out=[]; const save=tag=>out.push(checkpoint(service,tag)); const marker=old.addMarker(0);
  marker.onDispose(()=>{ save('marker'); service.buffers.normal.x=3; old.x=2; save('changed'); });
  service.onScroll(()=>save('scroll')); service.reset(); save('final');
  reentrantRows.push(['reset-marker',out.join('^')].join('\t'));
}
{
  const service=fullService(); const out=[]; const save=tag=>out.push(checkpoint(service,tag)); let activated=false;
  service.buffer.lines.onTrim(()=>{ save('trim'); if(!activated) {activated=true; service.buffers.activateAltBuffer(); save('activated');} });
  service.onScroll(()=>save('scroll')); service.scroll(new AttributeData()); save('final');
  reentrantRows.push(['trim-activation',out.join('^')].join('\t'));
}
{
  const service=fullService(); const out=[]; const save=tag=>out.push(checkpoint(service,tag)); let resized=false;
  service.buffer.lines.onTrim(()=>{ save('trim'); if(!resized) {resized=true; service.resize(6,4); save('resized');} });
  service.onResize(()=>save('resize')); service.onScroll(()=>save('scroll')); service.scroll(new AttributeData()); save('final');
  reentrantRows.push(['trim-resize',out.join('^')].join('\t'));
}
const reentrantFixture=reentrantRows.join('\n')+'\n';
const reentrantPath=resolve(root,'crates/xterm-core/tests/fixtures/buffer-service-reentrant.tsv');
if(process.argv.includes('--generate')) writeFileSync(reentrantPath,reentrantFixture);
else assert.equal(readFileSync(reentrantPath,'utf8'),reentrantFixture,'reentry fixture drift');
const seed = 0x5e7b0ff;
let state = seed;
function random() { state ^= state << 13; state ^= state >>> 17; state ^= state << 5; return state >>> 0; }
const traces = [];
const counts = {};
for (let trace = 0; trace < 64; trace++) {
  const options = new OptionsService({ cols: 10, rows: 5, scrollback: 4 });
  const service = new BufferService(options, new MockLogService());
  let events = [];
  service.onScroll(value => events.push(`s,${value}`));
  service.onResize(e => events.push(`r,${e.cols},${e.rows},${+e.colsChanged},${+e.rowsChanged}`));
  service.buffers.onBufferActivate(e => events.push(`a,${e.activeBuffer === service.buffers.normal ? 'n' : 'a'}`));
  for (let step = 0; step < 64; step++) {
    events = [];
    const choices = ['scroll', 'display', 'margins', 'cursor', 'alt', 'normal', 'resize', 'reset', 'scrollback', 'tabs', 'write', 'user'];
    const op = choices[(trace + step) % choices.length];
    const a = random(); const b = random(); const c = random();
    let p = a; let q = b;
    const attr = new AttributeData(); attr.fg = a & 0xffffff; attr.bg = b & 0xffffff;
    const buffer = service.buffer;
    switch (op) {
      case 'scroll': p = a & 1; q = attr.fg; service.scroll(attr, !!p); break;
      case 'display': p = (a % 21) - 10; q = b & 1; service.scrollLines(p, !!q); break;
      case 'margins': p = a % service.rows; q = p + b % (service.rows - p); buffer.scrollTop = p; buffer.scrollBottom = q; break;
      case 'cursor': p = a % service.cols; q = b % service.rows; buffer.x = p; buffer.y = q; break;
      case 'alt': service.buffers.activateAltBuffer(attr); break;
      case 'normal': service.buffers.activateNormalBuffer(); break;
      case 'resize': p = 2 + a % 15; q = 1 + b % 7; service.resize(p, q); break;
      case 'reset': service.reset(); break;
      case 'scrollback': p = a % 10; options.options.scrollback = p; break;
      case 'tabs': p = 1 + a % 8; options.options.tabStopWidth = p; break;
      case 'write': p = a % service.rows; q = 65 + b % 26; buffer.lines.get(buffer.ybase + p).setCellFromCodepoint(0, q, 1, attr); break;
      case 'user': p = a & 1; service.isUserScrolling = !!p; break;
    }
    counts[op] = (counts[op] || 0) + 1;
    const snapshot = [service.cols, service.rows, +service.isUserScrolling, service.buffer === service.buffers.normal ? 'n' : 'a',
      bufferSnapshot(service.buffers.normal), bufferSnapshot(service.buffers.alt), events.join('|') || '-'].join('\t');
    traces.push([trace, step, op, p, q, b & 0xffffff, a & 0xffffff, snapshot].join('\t'));
  }
  service.dispose();
}
const fixture = traces.join('\n') + '\n';
const fixturePath = resolve(root, 'crates/xterm-core/tests/fixtures/buffer-service.tsv');
if (process.argv.includes('--generate')) writeFileSync(fixturePath, fixture);
else assert.equal(readFileSync(fixturePath, 'utf8'), fixture, 'fixture drift; review before --generate');
const report = { command: 'node scripts/buffer-service-oracle.mjs', node: process.version, platform: process.platform,
  overallEquivalence: 'NOT PROVEN', sourceHashes, suites, directedChecks, seed, traces: 64, steps: 64, operations: counts,
  fixtureSha256: createHash('sha256').update(fixture).digest('hex'),
  reentrantCases: reentrantRows.map(row=>row.split('\t')[0]),
  reentrantSha256: createHash('sha256').update(reentrantFixture).digest('hex'),
  scope: 'Original JS BufferSet/BufferService tests and deterministic fixtures. Rust replay is a separate check.' };
writeFileSync(resolve(output, 'oracle.json'), JSON.stringify(report, null, 2) + '\n');
console.log(JSON.stringify({ tests: suites.reduce((n,s) => n+s.passed, 0), traceSteps: traces.length, seed }));
