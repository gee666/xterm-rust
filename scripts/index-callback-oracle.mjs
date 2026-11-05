// Copyright (c) 2026 xterm-rust contributors. Licensed under MIT.
// Direct InputHandler.parse keeps synchronous callback exceptions observable.
import assert from 'node:assert/strict';
import {createRequire} from 'node:module';
import {createHash} from 'node:crypto';
import {mkdirSync, readFileSync, writeFileSync, rmSync} from 'node:fs';
import {resolve} from 'node:path';
import {hex, snapshot} from './terminal-snapshot.mjs';
process.chdir(resolve(import.meta.dirname, '..'));
const require = createRequire(resolve(process.env.XTERM_SOURCE_ROOT ?? 'source_repo', 'package.json'));
const esbuild = require('esbuild');
const scratch = 'tmp/index-callback-oracle';
mkdirSync(scratch, {recursive: true});
const output = [];
try {
  const built = await esbuild.build({entryPoints: ['source_repo/src/headless/public/Terminal.ts'],
    outfile: `${scratch}/source.cjs`, bundle: true, platform: 'node', format: 'cjs',
    tsconfig: 'source_repo/src/headless/tsconfig.json', metafile: true, logLevel: 'silent'});
  const hashes = Object.fromEntries(Object.keys(built.metafile.inputs).sort().map(path =>
    [path, createHash('sha256').update(readFileSync(path)).digest('hex')]));
  const {Terminal} = require(resolve(`${scratch}/source.cjs`));
  for (const command of ['\x1bD', '\x1bE', '\x84', '\x85']) {
    for (const trigger of ['scroll', 'marker']) {
      for (const action of ['reset', 'inspect', 'error']) {
        const terminal = new Terminal({cols: 5, rows: 3, scrollback: 1, allowProposedApi: true});
        const parse = text => terminal._core._inputHandler.parse(text);
        parse('AAAAA\r\nBBBBB\r\nCCCCC\r\nDDDDD');
        const marker = terminal.registerMarker(-3);
        const records = [];
        const original = new Error('index callback sentinel');
        let called = false;
        const callback = () => {
          if (called) return;
          called = true;
          records.push(snapshot(terminal));
          if (action === 'error') throw original;
          if (action === 'reset') terminal.reset();
          else {
            terminal.options.convertEol = true;
            terminal.registerMarker(0);
          }
          records.push(snapshot(terminal));
        };
        if (trigger === 'scroll') terminal.onScroll(callback);
        else marker.onDispose(callback);
        const renders = [];
        terminal._core.onRender(({start, end}) => renders.push(`${start}:${end}`));
        let failure;
        try {parse(command + 'Z');} catch (error) {failure = error;}
        assert.equal(failure, action === 'error' ? original : undefined);
        assert.equal(records.length, action === 'error' ? 1 : 2);
        output.push([hex(command), trigger, action, +marker.isDisposed,
          records.join('~'), snapshot(terminal), renders.join(',')].join('\t'));
        terminal.dispose();
      }
    }
  }
  for (const [path, hash] of Object.entries(hashes)) {
    assert.equal(createHash('sha256').update(readFileSync(path)).digest('hex'), hash, path);
  }
  const fixture = 'crates/xterm-core/tests/fixtures/index-callbacks.tsv';
  const text = output.join('\n') + '\n';
  if (process.argv.includes('--generate')) writeFileSync(fixture, text);
  else assert.equal(readFileSync(fixture, 'utf8'), text, 'index callback fixture drift');
  mkdirSync('results/index-callbacks', {recursive: true});
  writeFileSync('results/index-callbacks/source.json', JSON.stringify({command: process.argv.slice(1),
    node: process.version, platform: process.platform, cases: output.length, sourceHashes: hashes,
    fixture, fixtureSha256: createHash('sha256').update(text).digest('hex'),
    scope: '24 real headless IND/NEL callback cases. Not full parser or browser equivalence.'}, null, 2) + '\n');
  console.log(`index callback source cases=${output.length} passed=${output.length}`);
} finally {
  esbuild.stop();
  rmSync(scratch, {recursive: true, force: true});
}
