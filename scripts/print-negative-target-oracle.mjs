// Copyright (c) 2026 The xterm.js authors. All rights reserved. MIT license.
// Compare real source printing with a custom Unicode provider, not a buffer mock.
import assert from 'node:assert/strict';
import { createRequire } from 'node:module';
import { createHash } from 'node:crypto';
import { mkdirSync, readFileSync, writeFileSync, rmSync } from 'node:fs';
import { resolve } from 'node:path';
import { hex, snapshot } from './terminal-snapshot.mjs';

const root = resolve(import.meta.dirname, '..');
const args = process.argv.slice(2);
const sourceArg = args.indexOf('--source-root');
const source = resolve(sourceArg < 0 ? resolve(root, 'source_repo') : args[sourceArg + 1]);
const generate = args.includes('--generate');
const scratch = resolve(root, 'tmp/print-negative-target579');
const output = resolve(root, 'results/print-negative-target579');
const fixture = resolve(root, 'crates/xterm-core/tests/fixtures/print-negative-target579.tsv');
const manifest = JSON.parse(readFileSync(resolve(root, 'docs/baseline/reference-manifest.json'), 'utf8'));
const sha = data => createHash('sha256').update(data).digest('hex');
function verifyReference() {
  for (const entry of manifest) {
    assert.equal(sha(readFileSync(resolve(source, entry.path))), entry.sha256, entry.path);
  }
}
verifyReference();
mkdirSync(scratch, { recursive: true });
mkdirSync(output, { recursive: true });
try {
  const require = createRequire(resolve(source, 'package.json'));
  const { build } = require('esbuild');
  const bundled = await build({
    entryPoints: [resolve(source, 'src/headless/public/Terminal.ts')],
    outfile: resolve(scratch, 'upstream.cjs'), bundle: true, platform: 'node',
    format: 'cjs', target: 'node24', tsconfig: resolve(source, 'src/headless/tsconfig.json'),
    metafile: true, logLevel: 'silent'
  });
  const { Terminal } = require(resolve(scratch, 'upstream.cjs'));
  const originalTests = resolve(scratch, 'original-tests.cjs');
  await build({
    entryPoints: [resolve(source, 'src/common/InputHandler.test.ts')],
    outfile: originalTests, bundle: true, platform: 'node', format: 'cjs',
    target: 'node24', tsconfig: resolve(source, 'src/common/tsconfig.json'),
    nodePaths: [resolve(source, 'src')], logLevel: 'silent',
    plugins: [{ name: 'original-chai', setup(builder) {
      builder.onResolve({ filter: /^chai$/ }, () => ({ path: require.resolve('chai'), external: true }));
    } }]
  });
  const Mocha = require('mocha');
  const outcomes = [];
  class Reporter {
    constructor(runner) {
      runner.on('test end', test => outcomes.push({ name: test.fullTitle(), status: test.state }));
      runner.on('fail', (test, error) => outcomes.push({ name: test.fullTitle(), error: String(error) }));
    }
  }
  const mocha = new Mocha({ reporter: Reporter, grep: /^InputHandler print /, timeout: 10000 });
  mocha.addFile(originalTests);
  const failed = await new Promise(done => mocha.run(done));
  assert.equal(failed, 0, 'unchanged original print suite');
  assert.equal(outcomes.length, 8, 'original print suite selection changed');
  const rows = [];
  let cases = 0;
  for (const first of [0, 1, 2, 3]) for (const joined of [0, 1, 2, 3]) {
    for (const split of [false, true]) for (const insert of [false, true]) {
      const id = `w${first}-j${joined}-split${+split}-insert${+insert}`;
      const t = new Terminal({ cols: 6, rows: 3, scrollback: 4, allowProposedApi: true });
      try {
        if (insert) await new Promise(done => t.write('\x1b[4h', done));
        t.unicode.register({ version: 'negative-target579', wcwidth: () => 1,
          charProperties(code) {
            const width = code === 65 ? first : code === 66 ? joined : 1;
            return (1 << 3) | (width << 1) | +(code === 66);
          }
        });
        t.unicode.activeVersion = 'negative-target579';
        const events = [];
        t.onCursorMove(() => events.push('C'));
        t.onBell(() => events.push('B'));
        t._core.onRender(e => events.push(`R${e.start}:${e.end}`));
        for (const text of [...(split ? ['A', 'B'] : ['AB']), 'C\x07']) {
          events.length = 0;
          await new Promise(done => t.write(text, done));
          rows.push([id, first, joined, +insert, hex(text), snapshot(t), events.join('/'),
            t._core._inputHandler._parser.precedingJoinState].join('\t'));
        }
        cases++;
      } finally { t.dispose(); }
    }
  }
  const text = rows.join('\n') + '\n';
  if (generate) writeFileSync(fixture, text);
  else assert.equal(readFileSync(fixture, 'utf8'), text, 'unchanged source fixture drift');
  verifyReference();
  const sourceHashes = Object.fromEntries(Object.keys(bundled.metafile.inputs).sort().map(path =>
    [path, sha(readFileSync(path))]));
  const report = { cases, snapshots: rows.length, node: process.version,
    platform: `${process.platform}-${process.arch}`, sourceFilesVerified: manifest.length,
    fixtureSha256: sha(text), sourceHashes, originalPrint: { discovered: 8,
      executed: outcomes.length, passed: outcomes.filter(test => test.status === 'passed').length,
      failed, skipped: outcomes.filter(test => test.status === 'pending').length, outcomes },
    scope: 'Custom Unicode widths 0..3, joining at negative and valid targets, chunk splits, insert mode, complete buffers and ordered cursor/render/Bell events. Not full equivalence.' };
  writeFileSync(resolve(output, 'source.json'), JSON.stringify(report, null, 2) + '\n');
  console.log(JSON.stringify({ cases, snapshots: rows.length, sourceFilesVerified: manifest.length }));
} finally {
  try { verifyReference(); }
  finally { rmSync(scratch, { recursive: true }); }
}
