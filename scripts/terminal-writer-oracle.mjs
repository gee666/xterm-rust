// Execute the same queue cases as wasm_terminal_writer against untouched source.
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
const temporary = await mkdtemp(resolve(root, 'tmp/terminal-writer-oracle-'));
const output = resolve(root, 'results/browser-writer/source.json');
const cases = 'crates/xterm-web/tests/browser_writer_cases.js';
const report = { status: 'failed', scope: 'Five queue cases including a real rendered root, not full equivalence',
  platform: process.platform, node: process.version, sources: [], browsers: [] };
const hash = bytes => createHash('sha256').update(bytes).digest('hex');
// Deliberately mirror Rust assertions so both independent executions validate
// full event payloads and event-time cursors, not merely final cells.
const reentry = slow => ({ text: 'PQR', events: [
  ['data', '\x1b[?1;1$y', 1], ['data', '\x1b[4;1$y', 2],
  ['cursor', 2], ['outer-complete', 2], ...(slow ? [['parsed', 2]] : []),
  ['cursor', 3], ['reentry-complete', 3], ['parsed', 3],
] });
const expected = [reentry(false), reentry(true),
  { events: [['first', 1], ['second', 2], ['parsed', 2]], text: 'AB' }, [],
  { ...reentry(false), firstRenderedCursor: 3 }];
await mkdir(temporary, { recursive: true });
await mkdir(dirname(output), { recursive: true });
try {
  const require = createRequire(resolve(reference, 'package.json'));
  const esbuild = require('esbuild');
  const sourcePath = resolve(reference, 'src/browser/public/Terminal.ts');
  const fixturePath = resolve(root, cases);
  const build = await esbuild.build({
    stdin: { contents: `export {Terminal} from ${JSON.stringify(sourcePath)}; export * from ${JSON.stringify(fixturePath)};`,
      resolveDir: root, sourcefile: 'writer-oracle-entry.ts' },
    outfile: resolve(temporary, 'source.js'), bundle: true, format: 'iife', globalName: 'WriterOracle',
    platform: 'browser', target: 'es2022', tsconfig: resolve(reference, 'src/browser/tsconfig.json'),
    metafile: true, logLevel: 'silent',
  });
  for (const file of Object.keys(build.metafile.inputs).filter(file => !file.startsWith('writer-oracle-entry'))) {
    const full = resolve(root, file);
    report.sources.push({ path: relative(root, full).replaceAll('\\', '/'), sha256: hash(await readFile(full)) });
  }
  report.esbuild = esbuild.version;
  const playwright = await import(pathToFileURL(resolve(reference, 'node_modules/playwright/index.mjs')));
  await withArtifactServer({ directory: temporary, css: await readFile(resolve(reference, 'css/xterm.css')),
    html: '<!doctype html><link rel="stylesheet" href="/xterm.css"><script src="/source.js"></script>' }, async origin => {
    for (const name of ['chromium', 'firefox', 'webkit']) {
      const browser = await playwright[name].launch({ headless: true, timeout: 30000 });
      const entry = { name, version: browser.version(), status: 'failed', captures: [] };
      report.browsers.push(entry);
      try {
        const page = await browser.newPage();
        page.setDefaultTimeout(30000);
        await page.goto(origin);
        await page.evaluate(() => {
          const make = () => new WriterOracle.Terminal({ allowProposedApi: true });
          const run = async () => [await WriterOracle.writerReentryCase(make(), false),
            await WriterOracle.writerReentryCase(make(), true),
            await WriterOracle.writerCompletionCase(make()),
            await WriterOracle.writerDisposalCase(make()),
            await WriterOracle.writerReentryCase(make(), false, true)];
          run().then(captures => { globalThis.writerResult = { captures }; },
            error => { globalThis.writerResult = { error: String(error.stack ?? error) }; });
        });
        await page.waitForFunction(() => globalThis.writerResult !== undefined);
        const result = await page.evaluate(() => globalThis.writerResult);
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
console.log(JSON.stringify(report.browsers.map(({ name, status, captures }) => ({ name, status, captures })), null, 2));
if (report.status !== 'passed') process.exitCode = 1;
