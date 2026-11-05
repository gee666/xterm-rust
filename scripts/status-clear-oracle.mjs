// Bundle unchanged reference methods and retain raw clear() coercion traces.
import assert from 'node:assert/strict';
import { createHash } from 'node:crypto';
import { mkdir, readFile, rm, writeFile } from 'node:fs/promises';
import { dirname, join, resolve } from 'node:path';
import { fileURLToPath, pathToFileURL } from 'node:url';
import { referenceIntegrity } from './upstream-baseline-utils.mjs';

const root = resolve(dirname(fileURLToPath(import.meta.url)), '..');
const source = resolve(process.env.SOURCE_ROOT ?? join(root, 'source_repo'));
const scratch = join(root, 'tmp/status-clear');
const output = join(root, 'results/status-clear');
const sourceFiles = ['addons/addon-program-status/src/ProgramStatusAddon.ts',
  'addons/addon-program-status/src/ProgramStatus.ts', 'src/common/Event.ts', 'src/common/Lifecycle.ts'];
const fixture = join(root, 'crates/xterm-web/tests/fixtures/status-clear.json');
const hash = bytes => createHash('sha256').update(bytes).digest('hex');
const report = { status: 'failed', node: process.version, platform: process.platform,
  scope: 'Original addon with registration fixture. Shared operation trace also runs through Rust Terminal in browsers.' };
await mkdir(output, { recursive: true });
await mkdir(scratch, { recursive: true });
try {
  const manifest = JSON.parse(await readFile(join(root, 'docs/baseline/reference-manifest.json'), 'utf8'));
  report.before = referenceIntegrity(source, manifest);
  assert.equal(report.before.mismatches.length, 0, 'Reference modified');
  const esbuild = await import(pathToFileURL(join(source, 'node_modules/esbuild/lib/main.js')));
  const bundle = join(scratch, 'original.mjs');
  await esbuild.build({ entryPoints: [join(source, sourceFiles[0])], bundle: true, platform: 'node',
    format: 'esm', alias: { common: join(source, 'src/common') }, outfile: bundle });
  const { ProgramStatusAddon } = await import(pathToFileURL(bundle));
  const traceSource = await readFile(join(root, 'crates/xterm-web/tests/fixtures/status-clear.js'), 'utf8');
  const trace = Function(`${traceSource};return statusClearCases;`)();
  const callbacks = new WeakMap();
  const rows = await trace(() => {
    const addon = new ProgramStatusAddon();
    addon.activate({ parser: {
      registerOscHandler(id, callback) { assert.equal(id, 7501); callbacks.set(addon, callback); return { dispose() {} }; },
      registerEscHandler() { return { dispose() {} }; }
    } });
    return { addon, close() { addon.dispose(); } };
  }, (addon, data) => callbacks.get(addon)(data));
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
console.log(`ProgramStatus.clear source trace ${report.status}, ${report.counts?.executed ?? 0} cases`);
