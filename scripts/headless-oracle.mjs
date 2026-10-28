// Copyright (c) 2019 The xterm.js authors. All rights reserved.
// Licensed under MIT. Runs unchanged headless public API assertions.
import assert from 'node:assert/strict';
import { createRequire } from 'node:module';
import { mkdir, readFile, writeFile } from 'node:fs/promises';
import { createHash } from 'node:crypto';
import path from 'node:path';

const require = createRequire(path.resolve('source_repo/package.json'));
const { build } = require('esbuild');
const Mocha = require('mocha');
const root = process.cwd();
const source = 'source_repo/src/headless/public/Terminal.test.ts';
const output = path.resolve('tmp/headless-oracle/upstream.cjs');
await mkdir(path.dirname(output), { recursive: true });
await mkdir('results/headless', { recursive: true });
const bundled = await build({
  absWorkingDir: root,
  entryPoints: [source], outfile: output, bundle: true,
  platform: 'node', format: 'cjs', target: 'node24',
  tsconfig: 'source_repo/src/headless/tsconfig.json', metafile: true,
  logLevel: 'silent'
});
const hashes = {};
for (const file of Object.keys(bundled.metafile.inputs).sort()) {
  hashes[file] = createHash('sha256').update(await readFile(file)).digest('hex');
}
const mocha = new Mocha({ timeout: 10000, reporter: 'spec' });
mocha.addFile(output);
const outcomes = [];
let failures;
await new Promise(resolve => {
  const runner = mocha.run(count => { failures = count; resolve(); });
  runner.on('pass', test => outcomes.push({ name: test.fullTitle(), status: 'passed' }));
  runner.on('fail', (test, error) => outcomes.push({
    name: test.fullTitle(), status: 'failed', error: error.stack ?? String(error)
  }));
  runner.on('pending', test => outcomes.push({ name: test.fullTitle(), status: 'skipped' }));
});
const discovered = [...(await readFile(source, 'utf8')).matchAll(/\bit\('/g)].length;
const count = status => outcomes.filter(test => test.status === status).length;
const report = {
  command: 'node scripts/headless-oracle.mjs', node: process.version,
  platform: process.platform, arch: process.arch,
  versions: { esbuild: require('esbuild/package.json').version, mocha: require('mocha/package.json').version },
  implementation: 'Unchanged source_repo TypeScript bundled in tmp/. This is JavaScript baseline, not Rust coverage.',
  sourceHashes: hashes,
  upstream: { discovered, executed: count('passed') + count('failed'), passed: count('passed'),
    failed: count('failed'), skipped: count('skipped'), unrun: discovered - outcomes.length },
  outcomes
};
await writeFile('results/headless/upstream.json', `${JSON.stringify(report, null, 2)}\n`);
assert.equal(failures, 0, 'headless upstream assertions failed');
assert.equal(report.upstream.unrun, 0, 'headless test discovery mismatch');
console.log(JSON.stringify(report.upstream));
