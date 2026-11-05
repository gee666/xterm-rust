// Run the original legacy Node assertions against source and Rust/WASM.
// Only TypeScript/module resolution is adapted; test bodies stay unchanged.
import fs from 'node:fs/promises';
import path from 'node:path';
import Module, { createRequire } from 'node:module';
import { pathToFileURL } from 'node:url';
import { createHash } from 'node:crypto';
import { spawnSync } from 'node:child_process';
const root = process.cwd();
const require = createRequire(process.env.SOURCE_PACKAGE_JSON ?? path.join(root, 'source_repo/package.json'));
const rustModule = process.argv[2];
if (!rustModule) throw new Error('Usage: node scripts/ligature-legacy-check.mjs PATH_TO_NODEJS_WASM_BINDINGS');
const output = path.join(root, 'results/font-legacy');
await fs.mkdir(output, { recursive: true });
const baseline = spawnSync(process.execPath, ['scripts/ligature-addon-oracle.mjs'], {
  cwd: root, env: process.env, encoding: 'utf8', timeout: 120_000
});
await fs.writeFile(path.join(output, 'source-preparation.log'), baseline.stdout + baseline.stderr);
if (baseline.error || baseline.status !== 0) throw baseline.error ?? new Error('Source oracle preparation failed');
const source = await import(pathToFileURL(path.join(root, 'tmp/ligature-addon-oracle/index.js')));
const bindingPath = path.resolve(rustModule);
const wasmPath = bindingPath.replace(/\.js$/, '_bg.wasm');
const wasm = await fs.readFile(wasmPath);
if (!wasm.subarray(0, 4).equals(Buffer.from([0, 97, 115, 109]))) throw new Error('Expected a real WASM module');
const rust = require(bindingPath);
const file = path.join(root, 'source_repo/addons/addon-ligatures/test/LigaturesAddon.test.ts');
const bytes = await fs.readFile(file);
const ts = require('typescript');
const compiled = ts.transpileModule(bytes.toString('utf8'), {
  compilerOptions: { target: ts.ScriptTarget.ES2022, module: ts.ModuleKind.CommonJS }
}).outputText;
const Mocha = require('mocha');
const report = { node: process.version, platform: process.platform,
  scope: 'Unchanged legacy Node assertions; font-finder mock is upstream legacy setup, not real font loading proof',
  sourceTest: path.relative(root, file),
  sourceSha256: createHash('sha256').update(bytes).digest('hex'),
  rustArtifact: { bindings: path.relative(root, bindingPath),
    bindingsSha256: createHash('sha256').update(await fs.readFile(bindingPath)).digest('hex'),
    wasm: path.relative(root, wasmPath), wasmBytes: wasm.length,
    wasmSha256: createHash('sha256').update(wasm).digest('hex') }, runs: [] };
for (const [implementation, exports] of [['source', source], ['rust-wasm', rust]]) {
  const mocha = new Mocha({ reporter: 'spec', timeout: 10_000 });
  mocha.suite.emit('pre-require', globalThis, file, mocha);
  const test = new Module(file);
  test.filename = file;
  test.require = name => name === '../out-esbuild/index' ? exports : require(name);
  test._compile(compiled, file);
  const outcomes = [];
  let runner;
  await new Promise(resolve => {
    runner = mocha.run(resolve);
    runner.on('pass', test => outcomes.push({ name: test.fullTitle(), status: 'passed' }));
    runner.on('fail', (test, error) => outcomes.push({ name: test.fullTitle(), status: 'failed', error: error.stack }));
    runner.on('pending', test => outcomes.push({ name: test.fullTitle(), status: 'skipped' }));
  });
  const stats = runner.stats;
  report.runs.push({ implementation, discovered: runner.total, executed: stats.tests - stats.pending,
    passed: stats.passes, failed: stats.failures, skipped: stats.pending,
    unrun: runner.total - stats.tests, outcomes });
}
await fs.writeFile(path.join(output, 'legacy-results.json'), JSON.stringify(report, null, 2) + '\n');
if (report.runs.some(run => run.failed || run.skipped || run.unrun || run.discovered !== 8)) process.exitCode = 1;
console.log(JSON.stringify(report));
