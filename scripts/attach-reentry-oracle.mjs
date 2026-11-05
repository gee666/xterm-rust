// Compare unchanged AttachAddon methods with shared synchronous callback traces.
import assert from 'node:assert/strict';
import { createHash } from 'node:crypto';
import { mkdir, readFile, rm, writeFile } from 'node:fs/promises';
import { dirname, join, resolve } from 'node:path';
import { fileURLToPath, pathToFileURL } from 'node:url';
import { referenceIntegrity } from './upstream-baseline-utils.mjs';

const root = resolve(dirname(fileURLToPath(import.meta.url)), '..');
const source = resolve(process.env.SOURCE_ROOT ?? join(root, 'source_repo'));
const scratch = join(root, 'tmp/attach-reentry');
const output = join(root, 'results/attach-reentry');
const sourcePath = 'addons/addon-attach/src/AttachAddon.ts';
const fixture = join(root, 'crates/xterm-web/tests/fixtures/attach-reentry.json');
const hash = bytes => createHash('sha256').update(bytes).digest('hex');
const report = { status: 'failed', node: process.version, platform: process.platform,
  scope: 'Unchanged AttachAddon methods with shared socket traces and a recording terminal, not a full original suite.' };
await mkdir(output, { recursive: true });
await mkdir(scratch, { recursive: true });
try {
  const manifest = JSON.parse(await readFile(join(root, 'docs/baseline/reference-manifest.json'), 'utf8'));
  report.before = referenceIntegrity(source, manifest);
  assert.equal(report.before.mismatches.length, 0, 'Reference modified');
  const esbuild = await import(pathToFileURL(join(source, 'node_modules/esbuild/lib/main.js')));
  const bundle = join(scratch, 'original.mjs');
  await esbuild.build({ entryPoints: [join(source, sourcePath)], bundle: true, platform: 'node', format: 'esm', outfile: bundle });
  const { AttachAddon } = await import(pathToFileURL(bundle));
  const traceSource = await readFile(join(root, 'crates/xterm-web/tests/fixtures/attach-reentry.js'), 'utf8');
  const trace = Function(`${traceSource};return attachReentryCases;`)();
  const rows = await trace(socket => {
    const writes = [];
    const addon = new AttachAddon(socket, { bidirectional: false });
    addon.activate({ write(data) { writes.push(typeof data === 'string' ? data : new TextDecoder().decode(data)); } });
    return { dispose: () => addon.dispose(), flush: async () => writes.join('') };
  });
  if (process.argv.includes('--generate')) await writeFile(fixture, `${JSON.stringify(rows, null, 2)}\n`);
  else assert.deepEqual(rows, JSON.parse(await readFile(fixture, 'utf8')), 'Reference trace changed');
  report.cases = rows;
  report.counts = { discovered: rows.length, executed: rows.length, passed: rows.length, failed: 0, skipped: 0, unrun: 0 };
  report.fixtureSha256 = hash(await readFile(fixture));
  report.traceSha256 = hash(Buffer.from(traceSource));
  report.source = { path: sourcePath, sha256: hash(await readFile(join(source, sourcePath))) };
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
console.log(`Attach reentry source trace ${report.status}, ${report.counts?.executed ?? 0} cases`);
