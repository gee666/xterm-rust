// Copyright (c) 2026 xterm-rust contributors. Licensed under MIT.
import assert from 'node:assert/strict';
import {createHash} from 'node:crypto';
import {createRequire} from 'node:module';
import {mkdirSync, mkdtempSync, readFileSync, writeFileSync, rmSync} from 'node:fs';
import {resolve, join} from 'node:path';
import {hex} from './terminal-snapshot.mjs';

const root = resolve(import.meta.dirname, '..');
const source = resolve(root, 'source_repo');
const require = createRequire(join(process.env.XTERM_SOURCE_DEPENDENCIES ?? source, 'package.json'));
const esbuild = require('esbuild'), Mocha = require('mocha');
const manifest = JSON.parse(readFileSync(join(root, 'docs/baseline/reference-manifest.json')));
function integrity() {
  for (const entry of manifest) {
    const bytes = readFileSync(join(source, entry.path));
    assert.equal(bytes.length, entry.bytes, entry.path);
    assert.equal(createHash('sha256').update(bytes).digest('hex'), entry.sha256, entry.path);
  }
  return manifest.length;
}
// The shared dense projection requires every row to exist. This variant retains
// absent slots explicitly, rather than filling them or changing source state.
function buffer(b) {
  const rows = [];
  for (let y = 0; y < b.lines.length; y++) {
    const line = b.lines.get(y);
    if (!line) {rows.push('-'); continue;}
    const cells = [];
    for (let x = 0; x < line.length; x++) {
      const c = line.getCell(x);
      cells.push([c.content >>> 0, c.fg >>> 0, c.bg >>> 0, hex(c.getChars()), c.extended.ext >>> 0, c.extended.urlId].join(','));
    }
    rows.push(`${+line.isWrapped}:${cells.join(';')}`);
  }
  return [b.x, b.y, b.ybase, b.ydisp, b.scrollTop, b.scrollBottom].join(',') + '/' + rows.join('/');
}
function snapshot(t) {
  const c = t._core, a = c._inputHandler.getAttrData(), m = c.coreService.decPrivateModes;
  return [c.cols, c.rows, +(c.buffers.active === c.buffers.alt), a.fg >>> 0, a.bg >>> 0,
    a.extended.ext >>> 0, a.extended.urlId, +m.origin, +m.reverseWraparound].join(',') +
    '|' + buffer(c.buffers.normal) + '|' + buffer(c.buffers.alt);
}
const before = integrity();
mkdirSync(join(root, 'tmp'), {recursive: true});
const scratch = mkdtempSync(join(root, 'tmp/backspace-sparse-'));
try {
  const common = {absWorkingDir: root, bundle: true, platform: 'node', format: 'cjs', logLevel: 'silent',
    nodePaths: [join(process.env.XTERM_SOURCE_DEPENDENCIES ?? source, 'node_modules')],
    tsconfig: join(source, 'src/headless/tsconfig.json')};
  const suite = join(scratch, 'original.cjs');
  await esbuild.build({...common, entryPoints: [join(source, 'src/common/InputHandler.test.ts')], outfile: suite});
  const upstream = [], mocha = new Mocha({timeout: 10000, grep: /BS with reverseWraparound set\/unset/, reporter: 'dot'});
  mocha.addFile(suite);
  const failed = await new Promise(done => {
    const runner = mocha.run(done);
    runner.on('pass', t => upstream.push({name: t.fullTitle(), status: 'passed'}));
    runner.on('fail', (t, e) => upstream.push({name: t.fullTitle(), status: 'failed', error: String(e.stack)}));
    runner.on('pending', t => upstream.push({name: t.fullTitle(), status: 'pending'}));
  });
  assert.equal(failed, 0);
  assert.equal(upstream.length, 8, 'Unchanged backspace suite inventory');
  assert.ok(upstream.every(t => t.status === 'passed'));
  const bundle = join(scratch, 'Terminal.cjs');
  const built = await esbuild.build({...common, entryPoints: [join(source, 'src/headless/public/Terminal.ts')], outfile: bundle, metafile: true});
  const {Terminal} = require(bundle), records = [];
  for (const alt of [false, true]) for (const origin of [false, true]) for (const reverse of [false, true]) {
    for (const x of [0, 5, 9]) for (const y of [0, 2, 4, 9]) {
      for (const pattern of ['missing-current', 'hard-current', 'wrapped-valid', 'wrapped-previous-missing', 'hard-previous-missing']) {
        const t = new Terminal({cols: 5, rows: 5, scrollback: 3, allowProposedApi: true});
        const c = t._core, parse = text => c._inputHandler.parse(text);
        parse((alt ? '\x1b[?1049h' : '') + '\x1b[31;44mAAAAA\r\nBBBBB\r\nCCCCC\r\nDDDDD\r\nEEEEE\r\nFFFFF\x1b[2;4r' +
          (origin ? '\x1b[?6h' : '') + (reverse ? '\x1b[?45h' : ''));
        const b = c.buffer, restrictedY = origin ? Math.min(Math.max(y, 1), 3) : Math.min(y, 4);
        const current = b.ybase + restrictedY;
        b.setWrapped(current, pattern.startsWith('wrapped'));
        if (pattern === 'missing-current') b.lines.set(current, undefined);
        if (pattern.endsWith('previous-missing') && current > 0) b.lines.set(current - 1, undefined);
        b.x = x; b.y = y;
        const events = [];
        c.onBell(() => events.push('B')); c.onCursorMove(() => events.push('C'));
        c.onRender(({start, end}) => events.push(`R${start}:${end}`));
        let error;
        try {parse('\x08\x07');} catch (e) {error = e;}
        if (error) {
          assert.equal(pattern, 'wrapped-previous-missing');
          assert.ok(error instanceof TypeError, String(error));
          assert.ok(/hasWidth/.test(error.message), error.message);
        }
        records.push([+alt, +origin, +reverse, x, y, pattern, +!!error, snapshot(t), events.join('/')].join('\t'));
        t.dispose();
      }
    }
  }
  assert.equal(records.length, 480);
  const contents = records.join('\n') + '\n';
  const fixture = join(root, 'crates/xterm-core/tests/fixtures/backspace-sparse.tsv');
  if (process.argv.includes('--generate')) writeFileSync(fixture, contents);
  else assert.equal(readFileSync(fixture, 'utf8').replaceAll('\r\n', '\n'), contents, 'Sparse backspace fixture drift');
  const after = integrity();
  const report = {command: process.argv.slice(1), node: process.version, platform: process.platform,
    sourceIntegrity: {before, after}, cases: records.length, expectedPreviousRowErrors: records.filter(r => r.split('\t')[6] === '1').length,
    upstream: {discovered: upstream.length, executed: upstream.length, passed: upstream.length, failed: 0, skipped: 0, unrun: 0, outcomes: upstream},
    fixtureSha256: createHash('sha256').update(contents).digest('hex'),
    sourceHashes: Object.fromEntries(Object.keys(built.metafile.inputs).map(p => [p, createHash('sha256').update(readFileSync(resolve(p))).digest('hex')])),
    scope: 'Direct internal sparse-row boundary inputs; complete packed cells/attributes and cursor/margins of both buffers, ordered cursor/render/BEL, previous-row error controls. Native Windows source runtime only. Full equivalence NOT PROVEN.'};
  const output = join(root, 'results/backspace-sparse');
  mkdirSync(output, {recursive: true});
  writeFileSync(join(output, 'source.json'), JSON.stringify(report, null, 2) + '\n');
  console.log(`Sparse BS ${records.length} source cases PASS; unchanged BS ${upstream.length} PASS; reference ${before}/${after} PASS`);
} finally {esbuild.stop(); rmSync(scratch, {recursive: true, force: true});}
