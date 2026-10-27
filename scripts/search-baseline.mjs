// Copyright (c) 2026 xterm-rust contributors. MIT license.
// Execute original search assertions unchanged against the original terminal.
import { createHash } from 'node:crypto';
import { mkdirSync, readFileSync, writeFileSync } from 'node:fs';
import { resolve } from 'node:path';
import { pathToFileURL } from 'node:url';

const root = resolve(import.meta.dirname, '..');
const dependencies = resolve(root, 'source_repo/node_modules');
const { build } = await import(pathToFileURL(resolve(dependencies, 'esbuild/lib/main.js')));
const { default: Mocha } = await import(pathToFileURL(resolve(dependencies, 'mocha/index.js')));
const temporary = resolve(root, 'tmp/search-baseline');
const output = resolve(root, 'results/search');
mkdirSync(temporary, { recursive: true });
mkdirSync(output, { recursive: true });
const results = [];
for (const suite of ['SearchEngine', 'SearchLineCache']) {
  const tests = `source_repo/addons/addon-search/src/${suite}.test.ts`;
  const bundle = resolve(temporary, `${suite}.test.cjs`);
  const built = await build({
    absWorkingDir: root, entryPoints: [tests], outfile: bundle,
    bundle: true, platform: 'node', format: 'cjs', metafile: true,
    tsconfig: 'source_repo/src/browser/tsconfig.json',
    alias: { common: resolve(root, 'source_repo/src/common'), browser: resolve(root, 'source_repo/src/browser') },
  });
  const hashes = Object.fromEntries(Object.keys(built.metafile.inputs)
    .filter(p => p.startsWith('source_repo/') && !p.includes('/node_modules/'))
    .map(file => [file, createHash('sha256').update(readFileSync(resolve(root, file))).digest('hex')]));
  const mocha = new Mocha({ reporter: 'spec' });
  mocha.addFile(bundle);
  await mocha.loadFilesAsync();
  const outcomes = [];
  let runner;
  const failed = await new Promise(done => {
    runner = mocha.run(done);
    runner.on('pass', test => outcomes.push({ name: test.fullTitle(), status: 'passed' }));
    runner.on('pending', test => outcomes.push({ name: test.fullTitle(), status: 'pending' }));
    runner.on('fail', (test, error) => outcomes.push({ name: test.fullTitle(), status: 'failed', error: String(error.stack) }));
  });
  results.push({ tests, hashes, discovered: runner.total,
    executed: runner.stats.tests - runner.stats.pending, passed: runner.stats.passes,
    failed, pending: runner.stats.pending, unrun: runner.total - runner.stats.tests, outcomes });
}
const report = { node: process.version, platform: process.platform,
  command: 'node scripts/search-baseline.mjs', overallEquivalence: 'NOT PROVEN',
  scope: 'Unchanged original SearchEngine and SearchLineCache tests over original browser Terminal. Not Rust execution.',
  suites: results };
writeFileSync(resolve(output, 'original.json'), JSON.stringify(report, null, 2) + '\n');
console.log(JSON.stringify(results.map(({ outcomes, hashes, ...summary }) => summary), null, 2));
if (results.some(s => s.failed || s.pending || s.unrun)) process.exitCode = 1;
