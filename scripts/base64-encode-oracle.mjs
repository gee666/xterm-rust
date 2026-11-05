// Run unchanged source Base64 methods against shared host-operation traces.
import assert from 'node:assert/strict';
import { createHash } from 'node:crypto';
import { mkdir, readFile, rm, writeFile } from 'node:fs/promises';
import { dirname, join, resolve } from 'node:path';
import { fileURLToPath, pathToFileURL } from 'node:url';
import { referenceIntegrity } from './upstream-baseline-utils.mjs';

const root = resolve(dirname(fileURLToPath(import.meta.url)), '..');
const source = resolve(process.env.SOURCE_ROOT ?? join(root, 'source_repo'));
const kind = process.argv.includes('--decode') ? 'decode' : 'encode';
const scratch = join(root, `tmp/base64-${kind}`);
const output = join(root, `results/base64-${kind}`);
const sourcePath = 'addons/addon-clipboard/src/ClipboardAddon.ts';
const fixture = join(root, `crates/xterm-web/tests/fixtures/base64-${kind}.json`);
const hash = bytes => createHash('sha256').update(bytes).digest('hex');
const report = { status: 'failed', node: process.version, platform: process.platform,
  scope: `Original Base64 ${kind}Text with shared host-method traces, not a full original terminal suite.` };
await mkdir(output, { recursive: true });
await mkdir(scratch, { recursive: true });
try {
  const manifest = JSON.parse(await readFile(join(root, 'docs/baseline/reference-manifest.json'), 'utf8'));
  report.before = referenceIntegrity(source, manifest);
  assert.equal(report.before.mismatches.length, 0, 'Reference modified');
  const esbuild = await import(pathToFileURL(join(source, 'node_modules/esbuild/lib/main.js')));
  const bundle = join(scratch, 'original.mjs');
  await esbuild.build({ entryPoints: [join(source, sourcePath)], bundle: true, platform: 'node', format: 'esm', outfile: bundle });
  const { Base64 } = await import(pathToFileURL(bundle));
  const traceSource = await readFile(join(root, `crates/xterm-web/tests/fixtures/base64-${kind}.js`), 'utf8');
  const trace = Function(`${traceSource};return base64${kind === 'decode' ? 'Decode' : 'Encode'}Cases;`)();
  const rows = trace(() => new Base64());
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
console.log(`Base64.${kind}Text source trace ${report.status}, ${report.counts?.executed ?? 0} cases`);
