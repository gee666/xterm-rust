// Unchanged source ProgressAddon setter, with shared Rust-browser assertions.
import assert from 'node:assert/strict';
import { createHash } from 'node:crypto';
import { mkdir, readFile, rm, writeFile } from 'node:fs/promises';
import { dirname, join, resolve } from 'node:path';
import { fileURLToPath, pathToFileURL } from 'node:url';
import { referenceIntegrity } from './upstream-baseline-utils.mjs';

const root = resolve(dirname(fileURLToPath(import.meta.url)), '..');
const source = resolve(process.env.SOURCE_ROOT ?? join(root, 'source_repo'));
const modules = resolve(process.env.REFERENCE_MODULES ?? join(source, 'node_modules'));
const scratch = join(root, 'tmp/progress-math');
const output = join(root, 'results/progress-math');
const fixture = join(root, 'crates/xterm-web/tests/fixtures/progress-math.json');
const tracePath = join(root, 'crates/xterm-web/tests/fixtures/progress-math.js');
const sourceFiles = ['addons/addon-progress/src/ProgressAddon.ts', 'src/common/Event.ts', 'src/common/Lifecycle.ts'];
const hash = bytes => createHash('sha256').update(bytes).digest('hex');
const report = { status: 'failed', node: process.version, platform: process.platform,
  scope: 'Public Progress setter only. Shared assertions use original addon and actual Rust/WASM addon.' };
await mkdir(output, { recursive: true });
await mkdir(scratch, { recursive: true });
try {
  const manifest = JSON.parse(await readFile(join(root, 'docs/baseline/reference-manifest.json'), 'utf8'));
  report.before = referenceIntegrity(source, manifest);
  assert.equal(report.before.mismatches.length, 0, 'Reference modified');
  const esbuild = await import(pathToFileURL(join(modules, 'esbuild/lib/main.js')));
  const bundle = join(scratch, 'original.mjs');
  const exports = `export {ProgressAddon} from ${JSON.stringify(join(source, sourceFiles[0]))};\nexport {Emitter} from ${JSON.stringify(join(source, sourceFiles[1]))};`;
  await esbuild.build({ stdin: { contents: exports, resolveDir: source, loader: 'ts' },
    bundle: true, platform: 'node', format: 'esm', alias: { common: join(source, 'src/common') }, outfile: bundle });
  const { ProgressAddon, Emitter } = await import(pathToFileURL(bundle));
  const traceSource = await readFile(tracePath, 'utf8');
  const trace = Function(`${traceSource};return progressMathCases;`)();
  const rows = await trace(() => {
    const addon = new ProgressAddon();
    addon.activate({ parser: { registerOscHandler(id) {
      assert.equal(id, 9); return { dispose() {} };
    } }, _core: { _onData: new Emitter() } });
    return { addon, close() { addon.dispose(); } };
  });
  if (process.argv.includes('--generate')) await writeFile(fixture, `${JSON.stringify(rows, null, 2)}\n`);
  else assert.deepEqual(rows, JSON.parse(await readFile(fixture, 'utf8')), 'Reference trace changed');
  report.cases = rows;
  report.counts = { discovered: rows.length, executed: rows.length, passed: rows.length, failed: 0, skipped: 0, unrun: 0 };
  report.fixtureSha256 = hash(await readFile(fixture));
  report.traceSha256 = hash(Buffer.from(traceSource));
  report.sources = await Promise.all(sourceFiles.map(async path => ({ path, sha256: hash(await readFile(join(source, path))) })));
  report.after = referenceIntegrity(source, manifest);
  assert.equal(report.after.mismatches.length, 0, 'Reference modified during oracle');
  report.status = 'passed';
} catch (error) {
  report.error = String(error.stack ?? error);
  process.exitCode = 1;
} finally {
  await rm(scratch, { recursive: true, force: true });
  await writeFile(join(output, 'source.json'), `${JSON.stringify(report, null, 2)}\n`);
}
console.log(`Progress public Math source ${report.status}, ${report.counts?.executed ?? 0} cases`);
