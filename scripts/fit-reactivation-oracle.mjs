// Compare unchanged FitAddon.ts with the exact host-callback regression corpus.
import assert from 'node:assert/strict';
import { createHash } from 'node:crypto';
import { mkdir, readFile, rm, writeFile } from 'node:fs/promises';
import { dirname, join, resolve } from 'node:path';
import { fileURLToPath, pathToFileURL } from 'node:url';
import { referenceIntegrity } from './upstream-baseline-utils.mjs';

const root = resolve(dirname(fileURLToPath(import.meta.url)), '..');
const source = resolve(process.env.SOURCE_ROOT ?? join(root, 'source_repo'));
const scratch = join(root, 'tmp/fit-reactivation');
const output = join(root, 'results/fit-reactivation');
const input = 'addons/addon-fit/src/FitAddon.ts';
const fixture = join(root, 'crates/xterm-web/tests/fixtures/fit-reactivation.json');
const hash = bytes => createHash('sha256').update(bytes).digest('hex');
const report = { status: 'failed', node: process.version, platform: process.platform,
  scope: 'Unchanged FitAddon with DOM host fixtures, not the original browser suite.' };
await mkdir(output, { recursive: true });
await mkdir(scratch, { recursive: true });
try {
  const manifest = JSON.parse(await readFile(join(root, 'docs/baseline/reference-manifest.json'), 'utf8'));
  report.before = referenceIntegrity(source, manifest);
  assert.equal(report.before.mismatches.length, 0, 'Reference modified');
  const esbuild = await import(pathToFileURL(join(source, 'node_modules/esbuild/lib/main.js')));
  const bundle = join(scratch, 'original.mjs');
  await esbuild.build({ entryPoints: [join(source, input)], bundle: true, platform: 'node', format: 'esm', outfile: bundle,
    nodePaths: [join(source, 'src')] });
  const { FitAddon } = await import(pathToFileURL(bundle));
  const traceSource = await readFile(join(root, 'crates/xterm-web/tests/fixtures/fit-reactivation.js'), 'utf8');
  const trace = Function(`${traceSource};return fitReactivationCases;`)();
  const host = { getComputedStyle(element) { return { getPropertyValue: name => element.style[name] ?? '0px' }; } };
  function terminal(cols, rows, width, height) {
    const ownerDocument = { defaultView: host };
    const parentElement = { ownerDocument, style: { width: `${width}px`, height: `${height}px` } };
    return { cols, rows, options: { scrollback: 0 }, dimensions: { css: { cell: { width: 10, height: 20 } } },
      element: { ownerDocument, parentElement, style: {} },
      resize(cols, rows) { this.cols = cols; this.rows = rows; }, dispose() {} };
  }
  const rows = trace(() => new FitAddon(), () => [terminal(13, 7, 800, 450), terminal(17, 8, 600, 350)], host);
  if (process.argv.includes('--generate')) await writeFile(fixture, `${JSON.stringify(rows, null, 2)}\n`);
  else assert.deepEqual(rows, JSON.parse(await readFile(fixture, 'utf8')), 'Source trace changed');
  report.cases = rows;
  report.counts = { discovered: rows.length, executed: rows.length, passed: rows.length, failed: 0, skipped: 0, unrun: 0 };
  report.fixtureSha256 = hash(await readFile(fixture));
  report.traceSha256 = hash(Buffer.from(traceSource));
  report.source = { path: input, sha256: hash(await readFile(join(source, input))) };
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
console.log(`Fit reactivation source trace ${report.status}, ${report.counts?.executed ?? 0} cases`);
