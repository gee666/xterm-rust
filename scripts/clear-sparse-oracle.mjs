// Copyright (c) 2026 xterm-rust contributors. Licensed under MIT.
import assert from 'node:assert/strict';
import {createHash} from 'node:crypto';
import {createRequire} from 'node:module';
import {mkdirSync, mkdtempSync, readFileSync, writeFileSync, rmSync} from 'node:fs';
import {resolve, join} from 'node:path';
import {snapshot} from './terminal-snapshot.mjs';

const root = resolve(import.meta.dirname, '..');
const source = resolve(process.env.XTERM_SOURCE_ROOT ?? join(root, 'source_repo'));
const dependencies = resolve(process.env.XTERM_SOURCE_DEPENDENCIES ?? source);
const require = createRequire(join(dependencies, 'package.json'));
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
const before = integrity();
mkdirSync(join(root, 'tmp'), {recursive: true});
const scratch = mkdtempSync(join(root, 'tmp/clear-sparse-'));
try {
  const common = {absWorkingDir: root, bundle: true, platform: 'node', format: 'cjs', logLevel: 'silent',
    nodePaths: [join(dependencies, 'node_modules')], tsconfig: join(source, 'src/headless/tsconfig.json')};
  const suite = join(scratch, 'original.cjs');
  await esbuild.build({...common, entryPoints: [join(source, 'src/headless/public/Terminal.test.ts')], outfile: suite});
  const upstream = [], mocha = new Mocha({timeout: 10000, grep: /^Headless API Tests clear(?: disposes markers)?$/, reporter: 'dot'});
  mocha.addFile(suite);
  const failed = await new Promise(done => {
    const runner = mocha.run(done);
    runner.on('pass', t => upstream.push({name: t.fullTitle(), status: 'passed'}));
    runner.on('fail', (t, e) => upstream.push({name: t.fullTitle(), status: 'failed', error: String(e.stack)}));
    runner.on('pending', t => upstream.push({name: t.fullTitle(), status: 'pending'}));
  });
  assert.equal(failed, 0);
  assert.equal(upstream.length, 2, 'Unchanged public clear suite inventory');
  assert.ok(upstream.every(t => t.status === 'passed'));
  const bundle = join(scratch, 'Terminal.cjs');
  const built = await esbuild.build({...common, entryPoints: [join(source, 'src/headless/public/Terminal.ts')], outfile: bundle, metafile: true});
  const {Terminal} = require(bundle), records = [];
  for (const alternate of [false, true]) for (const rows of [1, 4]) for (const x of [0, 5]) {
    for (const pattern of ['dense', 'hole', 'truncated', 'empty', 'outside']) {
      for (const action of ['direct', 'callback', 'reset', 'resize', 'activate', 'marker-error', 'scroll-error']) {
        const t = new Terminal({cols: 5, rows, scrollback: 3, allowProposedApi: true});
        const c = t._core;
        c._inputHandler.parse((alternate ? '\x1b[?1049h' : '') + '\x1b[31;44mAAAAA\r\nBBBBB\r\nCCCCC\r\nDDDDD\r\nEEEEE');
        const marker = t.registerMarker(), second = t.registerMarker(-1), events = [];
        const sentinel = new Error('sparse clear sentinel');
        let selected, ready = false;
        const capture = tag => events.push(`${tag}#${snapshot(t)}`);
        function mutate() {
          const b = c.buffer;
          b.x = x;
          b.y = pattern === 'outside' ? c.rows + 5 : c.rows - 1;
          const current = b.ybase + b.y;
          if (pattern !== 'dense') b.lines.set(current, undefined);
          if (pattern === 'truncated') b.lines.length = current;
          if (pattern === 'empty') b.lines.length = 0;
          selected = b.lines.get(current);
          ready = true;
          capture('ready');
        }
        const scroll = t.onScroll(position => {
          capture(`S${position}`);
          if (action === 'scroll-error') throw sentinel;
        });
        const resize = t.onResize(({cols, rows}) => capture(`Z${cols}:${rows}`));
        const first = marker.onDispose(() => {
          capture('M1');
          if (!ready) {
            if (action === 'reset') t.reset();
            if (action === 'resize') t.resize(7, 6);
            if (action === 'activate') {
              if (alternate) c.buffers.activateNormalBuffer();
              else c.buffers.activateAltBuffer();
            }
            mutate();
          }
          if (action === 'marker-error') throw sentinel;
        });
        const later = second.onDispose(() => capture('M2'));
        if (action === 'direct' || action === 'scroll-error') mutate();
        let failure;
        try {t.clear();} catch (error) {failure = error;}
        assert.equal(failure, action.endsWith('error') ? sentinel : undefined);
        capture('final');
        const same = c.buffer.lines.get(0) === selected;
        records.push([+alternate, rows, x, pattern, action, +!!failure, +same,
          +marker.isDisposed, +second.isDisposed, events.join('^')].join('\t'));
        first.dispose(); later.dispose(); scroll.dispose(); resize.dispose();
        t.dispose();
      }
    }
  }
  assert.equal(records.length, 280);
  const contents = records.join('\n') + '\n';
  const fixture = join(root, 'crates/xterm-core/tests/fixtures/clear-sparse.tsv');
  if (process.argv.includes('--generate')) writeFileSync(fixture, contents);
  else assert.equal(readFileSync(fixture, 'utf8').replaceAll('\r\n', '\n'), contents, 'Sparse clear fixture drift');
  const after = integrity();
  const report = {command: process.argv.slice(1), node: process.version, platform: process.platform,
    sourceIntegrity: {before, after}, cases: records.length,
    upstream: {discovered: upstream.length, executed: upstream.length, passed: upstream.length, failed: 0, skipped: 0, unrun: 0, outcomes: upstream},
    fixtureSha256: createHash('sha256').update(contents).digest('hex'),
    sourceHashes: Object.fromEntries(Object.keys(built.metafile.inputs).map(p => [p, createHash('sha256').update(readFileSync(resolve(root, p))).digest('hex')])),
    scope: 'Public headless clear with direct/internal sparse slots and marker-callback reset/resize/activation, packed full-state and ordered callback snapshots, row identity, exact thrown sentinel. Native execution separate. Browser runtime, complete families and full equivalence NOT PROVEN.'};
  const output = join(root, 'results/clear-sparse');
  mkdirSync(output, {recursive: true});
  writeFileSync(join(output, 'source.json'), JSON.stringify(report, null, 2) + '\n');
  console.log(`Sparse clear ${records.length} source cases PASS; unchanged clear ${upstream.length} PASS; reference ${before}/${after} PASS`);
} finally {esbuild.stop(); rmSync(scratch, {recursive: true, force: true});}
