// Run marker public-API cases against the unchanged supplied browser Terminal.
import assert from 'node:assert/strict';
import { createRequire } from 'node:module';
import { createHash } from 'node:crypto';
import { mkdir, mkdtemp, readFile, writeFile, rm } from 'node:fs/promises';
import { resolve, dirname, relative } from 'node:path';
import { fileURLToPath, pathToFileURL } from 'node:url';
import { withArtifactServer } from './browser-artifact-server.mjs';

const root = resolve(dirname(fileURLToPath(import.meta.url)), '..');
const reference = resolve(process.env.REFERENCE_ROOT ?? resolve(root, 'source_repo'));
await mkdir(resolve(root, 'tmp'), { recursive: true });
const temporary = await mkdtemp(resolve(root, 'tmp/terminal-marker-oracle-'));
const output = resolve(root, process.env.MARKER_ORACLE_REPORT ?? 'results/browser-markers/source.json');
const report = { status: 'failed', scope: 'Marker registration/handles/resources, not full array/decorations parity',
  platform: process.platform, node: process.version, sources: [], browsers: [] };
const hash = bytes => createHash('sha256').update(bytes).digest('hex');
// Independent expectations also appear in the Rust assertions. Source and Rust
// cannot agree silently on a changed event payload or incorrect coercion order.
const expected = [
  { lines: [0, 0, 0, 1, 2, -2, '02', '0', '0 2 ', 9007199254740992],
    errors: Array(6).fill('This API only accepts integers'),
    hints: ['number', 'number', 'default'], objectLine: '07' },
  { seen: [[true, true, true, -1]], disposed: [true, true, true], lines: [-1, -1, -1], remaining: 0 },
  { before: 1, alternateDisposed: true, capturedLine: 3, capturedInCurrentBuffer: false,
    hints: ['number', 'number', 'default'] },
  { log: ['event', 'resource', 'resource', 'event-added', 'appended', 'appended', 'changed'],
    gets: 2, errors: [true, true, true, true] },
];
await mkdir(dirname(output), { recursive: true });
try {
  const require = createRequire(resolve(reference, 'package.json'));
  const esbuild = require('esbuild');
  const fixture = resolve(root, 'crates/xterm-web/tests/browser_marker_cases.js');
  const source = resolve(reference, 'src/browser/public/Terminal.ts');
  const build = await esbuild.build({
    stdin: { contents: `export {Terminal} from ${JSON.stringify(source)}; export * from ${JSON.stringify(fixture)};`,
      resolveDir: root, sourcefile: 'marker-oracle-entry.ts' },
    outfile: resolve(temporary, 'source.js'), bundle: true, format: 'iife', globalName: 'MarkerOracle',
    platform: 'browser', target: 'es2022', tsconfig: resolve(reference, 'src/browser/tsconfig.json'),
    metafile: true, logLevel: 'silent',
  });
  for (const file of Object.keys(build.metafile.inputs).filter(file => !file.startsWith('marker-oracle-entry'))) {
    const full = resolve(root, file);
    report.sources.push({ path: relative(root, full).replaceAll('\\', '/'), sha256: hash(await readFile(full)) });
  }
  report.esbuild = esbuild.version;
  const playwright = await import(pathToFileURL(resolve(reference, 'node_modules/playwright/index.mjs')));
  await withArtifactServer({ directory: temporary, css: await readFile(resolve(reference, 'css/xterm.css')),
    html: '<!doctype html><script src="/source.js"></script>' }, async origin => {
    for (const name of ['chromium', 'firefox', 'webkit']) {
      const browser = await playwright[name].launch({ headless: true, timeout: 30000 });
      const entry = { name, version: browser.version(), status: 'failed' };
      report.browsers.push(entry);
      try {
        const page = await browser.newPage();
        page.setDefaultTimeout(30000);
        await page.goto(origin);
        await page.evaluate(() => {
          const make = () => new MarkerOracle.Terminal({ rows: 3, scrollback: 2 });
          const run = async () => [MarkerOracle.markerValidationCase(make()),
            await MarkerOracle.markerLifecycleCase(make()), await MarkerOracle.markerBufferCase(make()),
            MarkerOracle.markerResourcesCase(make())];
          run().then(captures => { globalThis.markerResult = { captures }; },
            error => { globalThis.markerResult = { error: String(error.stack ?? error) }; });
        });
        await page.waitForFunction(() => globalThis.markerResult !== undefined);
        const result = await page.evaluate(() => globalThis.markerResult);
        if (result.error) throw new Error(result.error);
        entry.captures = result.captures;
        assert.deepEqual(entry.captures, expected);
        entry.status = 'passed';
      } catch (error) { entry.error = String(error.stack ?? error); }
      finally { await browser.close(); }
    }
  });
  if (report.browsers.every(entry => entry.status === 'passed')) report.status = 'passed';
} catch (error) { report.error = String(error.stack ?? error); }
finally {
  await writeFile(output, JSON.stringify(report, null, 2) + '\n');
  await rm(temporary, { recursive: true, force: true });
}
console.log(JSON.stringify(report.browsers, null, 2));
if (report.status !== 'passed') process.exitCode = 1;
