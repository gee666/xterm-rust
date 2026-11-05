// Compare the actual SerializeAddon exports without modifying upstream tests.
import assert from 'node:assert/strict';
import { createServer } from 'node:http';
import { createHash } from 'node:crypto';
import { mkdir, readFile, realpath, rm, writeFile } from 'node:fs/promises';
import { dirname, join, resolve } from 'node:path';
import { fileURLToPath, pathToFileURL } from 'node:url';
import { buildBootstrap } from './e2e/build-bootstrap.mjs';
import { startFixtureServer } from './e2e/fixture-server.mjs';
import { artifactIdentity, sourceIdentity, verifyBuiltPackage } from './e2e/build-identity.mjs';
import { configForRust, testCounts } from './e2e/upstream-config.mjs';
import { runCommand } from './quality-process.mjs';
import { referenceIntegrity } from './upstream-baseline-utils.mjs';
import { serializationCases } from './serialize-browser-cases.mjs';
import { serializationGc } from './serialize-browser-gc.mjs';
const root = resolve(dirname(fileURLToPath(import.meta.url)), '..');
if (process.argv.slice(2).some(arg => !['--suite', '--gc'].includes(arg)) || process.argv.length > 3) throw Error('Usage: node scripts/serialize-browser-check.mjs [--suite|--gc]');
const gcOnly = process.argv.includes('--gc');
const reference = await realpath(process.env.SOURCE_ROOT ?? join(root, 'source_repo'));
const output = join(root, gcOnly ? 'results/serialize-gc' : 'results/serialize-browser');
const temporary = join(root, gcOnly ? 'tmp/serialize-gc' : 'tmp/serialize-browser');
await mkdir(output, { recursive: true }); await mkdir(temporary, { recursive: true });
const report = { status: 'failed', platform: process.platform, node: process.version, seed: gcOnly ? null : '0x05e21a11', seededCases: gcOnly ? 0 : 32,
  scope: gcOnly ? 'Forced Chromium GC for SerializeAddon host retention and unreachable cycles. Firefox/WebKit unrun; not full equivalence.'
    : 'Callable SerializeAddon, directed host behavior and seeded full public-cell restoration. Not full equivalence.', browsers: [], suites: [] };
const assets = new Map([['/test', ['demo/test.html', 'text/html']], ['/xterm.css', ['css/xterm.css', 'text/css']],
  ['/dist/client-bundle.js', ['demo/dist/client-bundle.js', 'text/javascript']]]);
let source, rust, lifetime;
try {
  const manifest = JSON.parse(await readFile(join(root, 'docs/baseline/reference-manifest.json'), 'utf8'));
  report.referenceBefore = referenceIntegrity(reference, manifest);
  assert.equal(report.referenceBefore.mismatches.length, 0);
  report.source = await sourceIdentity(root); report.build = await verifyBuiltPackage(root, report.source);
  const runnerFiles = ['scripts/serialize-browser-check.mjs', 'scripts/serialize-browser-cases.mjs', 'scripts/serialize-browser-gc.mjs', 'browser/xterm.mjs',
    'scripts/e2e/build-bootstrap.mjs', 'scripts/e2e/fixture-server.mjs', 'scripts/e2e/test-page.mjs'];
  report.runner = await artifactIdentity(root, runnerFiles);
  report.declarations = await artifactIdentity(root, ['browser/pkg/xterm_web.d.ts']);
  const declarations = await readFile(join(root, 'browser/pkg/xterm_web.d.ts'), 'utf8');
  for (const contract of [/serialize\(options\?: any/, /serializeAsHTML\(options\?: any/,
    /getNullCell\(\): BufferCell;/, /clone\(\): ExtendedCellAttributes;/]) {
    assert.match(declarations, contract, 'Generated callable declarations regressed');
  }
  report.reference = [];
  for (const file of [...assets.values()].map(([file]) => file).concat('addons/addon-serialize/src/SerializeAddon.ts', 'addons/addon-serialize/test/SerializeAddon.test.ts')) {
    const bytes = await readFile(join(reference, file)); report.reference.push({ path: file, bytes: bytes.length, sha256: createHash('sha256').update(bytes).digest('hex') });
  }
  const bootstrapFile = join(temporary, 'bootstrap.js');
  await buildBootstrap({ root, referenceRoot: reference, outputFile: bootstrapFile });
  rust = await startFixtureServer({ root, referenceRoot: reference, packageRoot: join(root, 'browser/pkg'), bootstrapFile, timeoutMs: 360000 });
  source = createServer(async (request, response) => {
    try { const asset = assets.get(request.url); if (!asset) { response.writeHead(404).end(); return; }
      response.setHeader('Content-Type', asset[1]); response.end(await readFile(join(reference, asset[0])));
    } catch (error) { response.writeHead(500).end(String(error)); }
  });
  await new Promise((ready, reject) => { source.once('error', reject); source.listen(0, '127.0.0.1', ready); });
  lifetime = setTimeout(() => source.closeAllConnections(), 360000);
  const origins = [['source', `http://127.0.0.1:${source.address().port}`], ['rust', rust.origin]];
  const playwright = await import(pathToFileURL(process.env.PLAYWRIGHT_MODULE ?? join(reference, 'node_modules/playwright/index.mjs')));
  if (gcOnly) report.gc = await serializationGc(playwright, origins);
  for (const name of gcOnly ? [] : ['chromium', 'firefox', 'webkit']) {
    let browser; const entry = { name, status: 'failed', mismatches: [] }; report.browsers.push(entry);
    try {
      browser = await playwright[name].launch({ headless: true, timeout: 30000 }); entry.version = browser.version();
      const traces = {};
      for (const [engine, origin] of origins) {
        const page = await browser.newPage(); page.setDefaultTimeout(45000);
        await page.goto(`${origin}/test`);
        let deadline;
        try { traces[engine] = await Promise.race([page.evaluate(serializationCases), new Promise((_, reject) => {
          deadline = setTimeout(() => reject(Error(`${engine} serialization probes exceeded45s`)), 45000);
        })]); } finally { clearTimeout(deadline); }
        await writeFile(join(output, `${name}-${engine}.json`), JSON.stringify(traces[engine]) + '\n');
        if (engine === 'rust') {
          await page.evaluate(async () => {
            const container = document.querySelector('#terminal-container');
            const term = new Terminal({ cols: 30, rows: 5 }); term.open(container);
            await new Promise(resolve => term.write('\x1b[31mRestored Rust serialization\x1b[0m\r\n中 e\u0301', resolve));
            const addon = new SerializeAddon(); term.loadAddon(addon); const ansi = addon.serialize();
            term.reset(); await new Promise(resolve => term.write(ansi, resolve));
            await new Promise(resolve => requestAnimationFrame(resolve));
            await new Promise(resolve => requestAnimationFrame(resolve));
          });
          assert.match(await page.locator('.xterm-rows').innerText(), /Restored Rust serialization/);
          await page.screenshot({ path: join(output, `${name}-terminal.png`) });
        }
        await page.close();
      }
      assert.equal(traces.source.length, 90, 'Expected58 directed and32 seeded cases');
      assert.equal(traces.source.filter(entry => entry.name.startsWith('seeded-')).length, 32);
      assert.equal(traces.source.length, traces.rust.length);
      entry.discovered = traces.source.length; entry.executed = traces.rust.length;
      for (let index = 0; index < traces.source.length; index++) {
        try {
          assert.equal(traces.source[index].thrown, undefined, 'Original operation unexpectedly threw');
          assert.deepEqual(traces.rust[index], traces.source[index]);
        }
        catch (error) { entry.mismatches.push({ name: traces.source[index].name, error: error.message }); }
      }
      entry.passed = entry.executed - entry.mismatches.length; entry.failed = entry.mismatches.length;
      entry.status = entry.failed ? 'failed' : 'passed';
    } catch (error) { entry.error = String(error.stack ?? error); }
    finally { if (browser) await browser.close(); }
  }
  if (process.argv.includes('--suite')) {
    for (const [engine, origin] of origins) {
      const directory = join(output, `${engine}-suite`); await mkdir(directory, { recursive: true });
      const configuration = join(temporary, `${engine}-playwright.cjs`);
      await writeFile(configuration, configForRust(join(reference, 'addons/addon-serialize/test/playwright.config.ts'), origin));
      const command = [join(reference, 'node_modules/@playwright/test/cli.js'), 'test', `--config=${configuration}`, '--reporter=json',
        '--workers=2', '--retries=0', '--global-timeout=120000', `--output=${join(temporary, `${engine}-artifacts`)}`];
      const result = await runCommand(process.execPath, command, { cwd: reference, log: join(directory, 'playwright.log'), timeoutMs: 130000,
        env: { ...process.env, CI: '1', FORCE_COLOR: '0', TMP: temporary, TEMP: temporary, TMPDIR: temporary,
          PLAYWRIGHT_JSON_OUTPUT_FILE: join(directory, 'playwright.json') } });
      const raw = JSON.parse(await readFile(join(directory, 'playwright.json'), 'utf8'));
      await writeFile(join(directory, 'configuration.cjs.txt'), await readFile(configuration));
      report.suites.push({ engine, counts: testCounts(raw), errors: raw.errors ?? [], result, command: [process.execPath, ...command] });
    }
  }
  assert.equal((await sourceIdentity(root)).sha256, report.source.sha256);
  assert.deepEqual(await artifactIdentity(root, runnerFiles), report.runner);
  report.referenceAfter = referenceIntegrity(reference, manifest); assert.equal(report.referenceAfter.mismatches.length, 0);
  report.status = report.browsers.every(entry => entry.status === 'passed') && report.suites.every(entry => entry.result.status === 'passed')
    && (!gcOnly || report.gc?.status === 'passed') ? 'passed' : 'failed';
  if (report.status !== 'passed') process.exitCode = 1;
} catch (error) { report.error = String(error.stack ?? error); process.exitCode = 1; }
finally {
  clearTimeout(lifetime);
  report.cleanupErrors = [];
  if (rust) {
    report.requests = rust.requests; report.runtimes = rust.runtimes; report.serverErrors = rust.failures;
    try { await rust.close(); } catch (error) { report.cleanupErrors.push(String(error)); }
    if (rust.failures.length) { report.status = 'failed'; process.exitCode = 1; }
  }
  if (source) try {
    await new Promise((done, reject) => { source.closeAllConnections(); source.close(error => error ? reject(error) : done()); });
  } catch (error) { report.cleanupErrors.push(String(error)); }
  try { await rm(temporary, { recursive: true, force: true }); } catch (error) { report.cleanupErrors.push(String(error)); }
  if (report.cleanupErrors.length) { report.status = 'failed'; process.exitCode = 1; }
  await writeFile(join(output, 'verification.json'), JSON.stringify(report, null, 2) + '\n');
}
console.log(gcOnly ? `Serialize Chromium GC ${report.status}`
  : `Serialize browser ${report.status}; ${report.browsers.map(entry => `${entry.name}:${entry.passed ?? 0}/${entry.executed ?? 0}`).join(', ')}`);
