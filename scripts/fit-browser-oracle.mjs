// Compare real FitAddon geometry with the supplied, unchanged JS distribution.
import assert from 'node:assert/strict';
import { createServer } from 'node:http';
import { createHash } from 'node:crypto';
import { mkdir, readFile, realpath, rm, statfs, writeFile } from 'node:fs/promises';
import { dirname, join, resolve } from 'node:path';
import { fileURLToPath, pathToFileURL } from 'node:url';
import { buildBootstrap } from './e2e/build-bootstrap.mjs';
import { startFixtureServer } from './e2e/fixture-server.mjs';
import { sourceIdentity, verifyBuiltPackage } from './e2e/build-identity.mjs';
import { configForRust, testCounts } from './e2e/upstream-config.mjs';
import { runCommand } from './quality-process.mjs';
import { referenceIntegrity } from './upstream-baseline-utils.mjs';

const root = resolve(dirname(fileURLToPath(import.meta.url)), '..');
const args = process.argv.slice(2);
if (args.length > 1 || args.some(arg => arg !== '--suite')) throw new Error('Usage: node scripts/fit-browser-oracle.mjs [--suite]');
const runSuite = args.includes('--suite');
const reference = await realpath(process.env.SOURCE_ROOT ?? join(root, 'source_repo'));
const output = join(root, 'results/fit-browser');
const temporary = join(root, 'tmp/fit-browser');
await mkdir(output, { recursive: true });
await mkdir(temporary, { recursive: true });
const report = { status: 'failed', platform: process.platform, node: process.version,
  scope: 'Default-font FitAddon source/Rust geometry trace only, not complete addon equivalence.', browsers: [] };
const assets = new Map([
  ['/test', ['demo/test.html', 'text/html']],
  ['/xterm.css', ['css/xterm.css', 'text/css']],
  ['/dist/client-bundle.js', ['demo/dist/client-bundle.js', 'text/javascript']]
]);
let source, rust, lifetime;
try {
  const manifest = JSON.parse(await readFile(join(root, 'docs/baseline/reference-manifest.json'), 'utf8'));
  report.referenceBefore = referenceIntegrity(reference, manifest);
  assert.equal(report.referenceBefore.mismatches.length, 0, 'Original source changed');
  report.referenceBundle = 'Existing generated original demo bundle; exact asset hash recorded, not rebuilt by this command.';
  report.source = await sourceIdentity(root);
  report.build = await verifyBuiltPackage(root, report.source);
  report.reference = [];
  for (const file of [...assets.values()].map(([file]) => file).concat(
    'addons/addon-fit/src/FitAddon.ts', 'addons/addon-fit/test/FitAddon.test.ts')) {
    const bytes = await readFile(join(reference, file));
    report.reference.push({ path: file, bytes: bytes.length,
      sha256: createHash('sha256').update(bytes).digest('hex') });
  }
  const bootstrapFile = join(temporary, 'bootstrap.js');
  await buildBootstrap({ root, referenceRoot: reference, outputFile: bootstrapFile });
  rust = await startFixtureServer({ root, referenceRoot: reference,
    packageRoot: join(root, 'browser/pkg'), bootstrapFile, timeoutMs: 90000 });
  source = createServer(async (request, response) => {
    try {
      const asset = assets.get(request.url);
      if (!asset) { response.writeHead(404).end(); return; }
      response.setHeader('Content-Type', asset[1]);
      response.end(await readFile(join(reference, asset[0])));
    } catch (error) { response.writeHead(500).end(String(error)); }
  });
  await new Promise((ready, reject) => {
    source.once('error', reject); source.listen(0, '127.0.0.1', ready);
  });
  lifetime = setTimeout(() => source.closeAllConnections(), 180000);
  const playwright = await import(pathToFileURL(process.env.PLAYWRIGHT_MODULE
    ?? join(reference, 'node_modules/playwright/index.mjs')));
  for (const name of ['chromium', 'firefox', 'webkit']) {
    const entry = { name, status: 'failed', traces: [] }; report.browsers.push(entry);
    let browser;
    try {
      browser = await playwright[name].launch({ headless: true, timeout: 30000 });
      entry.version = browser.version();
      for (const [engine, origin] of [['source', `http://127.0.0.1:${source.address().port}`], ['rust', rust.origin]]) {
        const page = await browser.newPage({ viewport: { width: 1024, height: 768 } });
        page.setDefaultTimeout(30000);
        await page.goto(`${origin}/test`);
        const snapshots = await page.evaluate(async () => {
          const container = document.querySelector('#terminal-container');
          container.style.width = '800px'; container.style.height = '450px';
          const terminal = new Terminal({ allowProposedApi: true });
          terminal.open(container);
          const fit = new FitAddon(); terminal.loadAddon(fit);
          terminal.loadAddon(new UnicodeGraphemesAddon());
          terminal.unicode.activeVersion = '15-graphemes';
          const snapshots = [];
          function record(stage) {
            snapshots.push({ stage, cols: terminal.cols, rows: terminal.rows,
              proposal: fit.proposeDimensions(), dimensions: terminal.dimensions,
              fontFamily: terminal.options.fontFamily, fontSize: terminal.options.fontSize,
              screenStyle: terminal.element.querySelector('.xterm-screen').getAttribute('style') });
          }
          try {
            await new Promise(resolve => requestAnimationFrame(resolve));
            for (const [label, width, height] of [['default', 800, 450], ['wide', 1008, 450],
              ['small', 1, 1], ['repeat', 800, 450]]) {
              container.style.width = `${width}px`; container.style.height = `${height}px`;
              terminal.reset(); record(`${label}:before`);
              fit.fit(); record(`${label}:first`);
              fit.fit(); record(`${label}:second`);
            }
            return snapshots;
          } finally { terminal.dispose(); }
        });
        entry.traces.push({ engine, snapshots });
        await page.close();
      }
      assert.deepEqual(entry.traces[1].snapshots, entry.traces[0].snapshots,
        `${name} live Fit geometry differs from original JS`);
      entry.comparedSnapshots = entry.traces[0].snapshots.length;
      entry.status = 'passed';
    } catch (error) { entry.error = String(error.stack ?? error); }
    finally { if (browser) await browser.close(); }
  }
  assert.ok(report.browsers.every(entry => entry.status === 'passed'), 'Fit source/Rust mismatch');
  report.geometryStatus = 'passed';
  if (runSuite) {
    const suiteOutput = join(output, 'source-suite');
    await mkdir(suiteOutput, { recursive: true });
    const configuration = join(temporary, 'playwright.cjs');
    await writeFile(configuration, configForRust(join(reference, 'addons/addon-fit/test/playwright.config.ts'),
      `http://127.0.0.1:${source.address().port}`));
    const command = [join(reference, 'node_modules/@playwright/test/cli.js'), 'test', `--config=${configuration}`,
      '--reporter=json', '--workers=2', '--retries=0', '--global-timeout=90000', `--output=${join(temporary, 'artifacts')}`];
    const result = await runCommand(process.execPath, command, { cwd: reference,
      log: join(suiteOutput, 'playwright.log'), timeoutMs: 100000,
      resourceCheck: async () => {
        const disk = await statfs(root);
        if (disk.bavail * disk.bsize < 1024 ** 3) throw new Error('Fit needs1GiB free');
      }, env: { ...process.env, CI: '1', FORCE_COLOR: '0', TMP: temporary, TEMP: temporary, TMPDIR: temporary,
        PLAYWRIGHT_JSON_OUTPUT_FILE: join(suiteOutput, 'playwright.json') } });
    const raw = JSON.parse(await readFile(join(suiteOutput, 'playwright.json'), 'utf8'));
    report.sourceSuite = { engine: 'unchanged original JavaScript', counts: testCounts(raw),
      errors: raw.errors ?? [], result, command: [process.execPath, ...command] };
    await writeFile(join(suiteOutput, 'summary.json'), `${JSON.stringify(report.sourceSuite, null, 2)}\n`);
    if (result.status !== 'passed' || report.sourceSuite.errors.length) process.exitCode = 1;
  }
  assert.equal((await sourceIdentity(root)).sha256, report.source.sha256, 'Rust inputs changed');
  report.referenceAfter = referenceIntegrity(reference, manifest);
  assert.equal(report.referenceAfter.mismatches.length, 0, 'Original source changed during run');
  report.status = process.exitCode ? 'source suite failed; geometry passed' : 'passed';
} catch (error) { report.error = String(error.stack ?? error); process.exitCode = 1; }
finally {
  clearTimeout(lifetime);
  if (rust) await rust.close();
  if (source) await new Promise((done, reject) => {
    source.closeAllConnections(); source.close(error => error ? reject(error) : done());
  });
  await rm(temporary, { recursive: true, force: true });
  await writeFile(join(output, 'oracle.json'), `${JSON.stringify(report, null, 2)}\n`);
}
console.log(`Fit geometry ${report.status}; ${report.browsers.map(entry =>
  `${entry.name}:${entry.comparedSnapshots ?? 0} snapshots`).join(', ')}`);
