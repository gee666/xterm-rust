// Copyright (c) 2026 xterm-rust contributors. Licensed under MIT.
// Run original assertions without rewriting source, imports, or test bodies.
import { createHash } from 'node:crypto';
import { mkdirSync, readFileSync, writeFileSync } from 'node:fs';
import { resolve } from 'node:path';
import { pathToFileURL } from 'node:url';

const root = resolve(import.meta.dirname, '..');
const dependencies = resolve(root, 'source_repo/node_modules');
const { build } = await import(pathToFileURL(resolve(dependencies, 'esbuild/lib/main.js')));
const { default: Mocha } = await import(pathToFileURL(resolve(dependencies, 'mocha/index.js')));
const temporary = resolve(root, 'tmp/buffer-baseline');
const output = resolve(root, 'results/buffer');
mkdirSync(temporary, { recursive: true });
mkdirSync(output, { recursive: true });
const suites = ['Buffer', 'BufferSet', 'BufferReflow'];
const results = [];
for (const suite of suites) {
  const source = `source_repo/src/common/buffer/${suite}.ts`;
  const tests = `source_repo/src/common/buffer/${suite}.test.ts`;
  const bundle = resolve(temporary, `${suite}.test.cjs`);
  const built = await build({
    absWorkingDir: root, entryPoints: [tests], outfile: bundle,
    bundle: true, platform: 'node', format: 'cjs', metafile: true,
    tsconfig: 'source_repo/src/common/tsconfig.json',
  });
  const hashes = {};
  for (const file of Object.keys(built.metafile.inputs).filter(p => p.startsWith('source_repo/src/'))) {
    hashes[file] = createHash('sha256').update(readFileSync(resolve(root, file))).digest('hex');
  }
  const mocha = new Mocha({ reporter: 'spec' });
  mocha.addFile(bundle);
  await mocha.loadFilesAsync();
  const outcomes = [];
  let runner;
  const failed = await new Promise(done => {
    runner = mocha.run(done);
    runner.on('pass', test => outcomes.push({ name: test.fullTitle(), status: 'passed' }));
    runner.on('pending', test => outcomes.push({ name: test.fullTitle(), status: 'skipped' }));
    runner.on('fail', (test, error) => outcomes.push({ name: test.fullTitle(), status: 'failed', error: String(error.stack) }));
  });
  results.push({ source, tests, hashes, discovered: runner.total,
    executed: runner.stats.tests - runner.stats.pending, passed: runner.stats.passes,
    failed, skipped: runner.stats.pending, unrun: runner.total - runner.stats.tests, outcomes });
}
const report = { node: process.version, platform: process.platform,
  command: 'node scripts/buffer-baseline.mjs',
  overallEquivalence: 'NOT PROVEN',
  note: 'Original JS baseline only. Rust assertion migration and differential execution are separate evidence.',
  suites: results };
writeFileSync(resolve(output, 'original.json'), JSON.stringify(report, null, 2) + '\n');
console.log(JSON.stringify(results.map(({ outcomes, hashes, ...summary }) => summary), null, 2));
if (results.some(s => s.failed || s.skipped || s.unrun)) process.exitCode = 1;
