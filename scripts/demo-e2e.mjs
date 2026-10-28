// Exercise the visible Rust demo through Playwright, never a reference engine.
import { mkdir, readFile, writeFile, statfs } from 'node:fs/promises';
import { dirname, join, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';
import { randomUUID } from 'node:crypto';
import { runCommand } from './quality-process.mjs';
import { startDemoServer } from './serve-demo.mjs';
import { testCounts } from './e2e/upstream-config.mjs';
import { artifactIdentity, sourceIdentity, verifyBuiltPackage } from './e2e/build-identity.mjs';

const root = resolve(dirname(fileURLToPath(import.meta.url)), '..');
const options = { case: 'demo-05', timeout: 300000, build: true, list: false };
const args = process.argv.slice(2);
for (let index = 0; index < args.length; index++) {
  const key = args[index];
  if (key === '--no-build') options.build = false;
  else if (key === '--list') options.list = true;
  else if (['--case', '--project', '--timeout-ms'].includes(key)) {
    const value = args[++index];
    if (!value || value.startsWith('--')) throw new Error(`${key} needs a value`);
    options[key === '--timeout-ms' ? 'timeout' : key.slice(2)] = key === '--timeout-ms' ? Number(value) : value;
  } else throw new Error(`Unknown argument ${key}`);
}
if (!['demo-05', 'demo-06'].includes(options.case)) throw new Error('Use --case demo-05 or demo-06');
if (options.project && !['Chromium', 'FirefoxStable', 'WebKit'].includes(options.project)) throw new Error('Unknown browser project');
if (!Number.isSafeInteger(options.timeout) || options.timeout < 1000 || options.timeout > 2147453647) throw new Error('Invalid bounded timeout');
const id = `${new Date().toISOString().replace(/[:.]/g, '-')}-${randomUUID()}`;
const output = join(root, 'results/demos/browser', id);
const temporary = join(root, 'tmp/demo-e2e', id);
await mkdir(output, { recursive: true });
await mkdir(temporary, { recursive: true });
const env = { ...process.env, TMP: temporary, TEMP: temporary, TMPDIR: temporary, CI: '1', FORCE_COLOR: '0' };
const report = { status: 'failed', case: options.case, implementation: 'Rust WASM demo', platform: process.platform,
  node: process.version, mode: options.list ? 'discovery only' : 'runtime', counts: null, errors: [], commands: [],
  scope: 'Demo control integration only. Not full source-suite or cross-platform equivalence.' };
const controller = new AbortController();
const interrupt = () => controller.abort(new Error('Demo E2E interrupted'));
process.once('SIGINT', interrupt); process.once('SIGTERM', interrupt);
let server;
async function resources() {
  const disk = await statfs(root, { bigint: true });
  if (disk.bavail * disk.bsize < 1024n ** 3n) throw new Error('Demo E2E needs 1 GiB free');
}
try {
  report.source = await sourceIdentity(root);
  const fixtureFiles = ['scripts/demo-e2e.mjs', 'scripts/serve-demo.mjs', 'scripts/demo-e2e/support.cjs',
    'scripts/demo-e2e/addon-probes.cjs', `scripts/demo-e2e/${options.case}.spec.cjs`,
    'browser/demo/index.html', 'browser/demo/app.mjs', 'browser/demo/controls.mjs', 'browser/demo/addons.mjs',
    'browser/demo/themes.mjs', 'browser/demo/demo.css'];
  report.fixture = await artifactIdentity(root, fixtureFiles);
  if (!options.list && options.build) {
    const result = await runCommand(process.execPath, ['scripts/build-demo.mjs', '--debug'], {
      cwd: root, env, log: join(output, 'build.log'), timeoutMs: options.timeout,
      resourceCheck: resources, signal: controller.signal });
    report.commands.push({ stage: 'build', ...result, log: 'build.log' });
    if (result.status !== 'passed') throw new Error('Actual WASM build failed; see build.log');
  }
  if (!options.list) {
    report.build = await verifyBuiltPackage(root, report.source);
    report.bindings = await artifactIdentity(root, ['browser/xterm.mjs', 'browser/xterm.css']);
    server = await startDemoServer({ port: 0, timeoutMs: options.timeout + 30000, webSocket: true });
  }
  const config = join(temporary, 'playwright.cjs');
  const originalConfig = join(root, 'source_repo/test/playwright/playwright.config.ts');
  await writeFile(config, `const loaded = require(${JSON.stringify(originalConfig)});
const original = loaded.default ?? loaded;
module.exports = { ...original, testDir: ${JSON.stringify(join(root, 'scripts/demo-e2e'))},
  testMatch: ${JSON.stringify(`${options.case}.spec.cjs`)}, timeout: 30000, webServer: undefined,
  use: { ...original.use, baseURL: ${JSON.stringify(server?.origin ?? 'http://127.0.0.1:1')},
    viewport: { width: 1280, height: 900 }, deviceScaleFactor: 1,
    screenshot: 'only-on-failure', trace: 'retain-on-failure' } };
`);
  const rawFile = join(output, 'playwright.json');
  const command = [join(root, 'source_repo/node_modules/@playwright/test/cli.js'), 'test', `--config=${config}`, '--reporter=json'];
  if (options.list) command.push('--list');
  else command.push('--workers=2', '--retries=0', `--global-timeout=${options.timeout - 500}`, `--output=${join(output, 'artifacts')}`);
  if (options.project) command.push(`--project=${options.project}`);
  report.command = [process.execPath, ...command];
  const result = await runCommand(process.execPath, command, { cwd: root,
    env: { ...env, PLAYWRIGHT_JSON_OUTPUT_FILE: rawFile }, log: join(output, 'playwright.log'),
    timeoutMs: options.timeout, resourceCheck: resources, signal: controller.signal });
  report.commands.push({ stage: 'playwright', ...result, log: 'playwright.log' });
  const raw = JSON.parse(await readFile(rawFile, 'utf8'));
  report.counts = testCounts(raw); report.errors = raw.errors ?? []; report.stats = raw.stats;
  if (!report.counts.discovered) throw new Error('No demo tests discovered');
  if (!options.list && !report.counts.executed) throw new Error('No demo assertions executed');
  if (result.status !== 'passed' || report.errors.length || controller.signal.aborted) throw new Error('Demo assertions failed or execution interrupted');
  if ((await sourceIdentity(root)).sha256 !== report.source.sha256
      || JSON.stringify(await artifactIdentity(root, fixtureFiles)) !== JSON.stringify(report.fixture)) {
    throw new Error('Source or fixture changed during execution');
  }
  report.status = options.list ? 'discovered, not executed' : 'passed';
} catch (error) { report.errors.push(String(error.stack ?? error)); process.exitCode = 1; }
finally {
  process.off('SIGINT', interrupt); process.off('SIGTERM', interrupt);
  if (server) {
    try { await server.close(); }
    catch (error) { report.errors.push(String(error)); report.status = 'failed'; process.exitCode = 1; }
    report.requests = server.requests; report.socketEvents = server.socketEvents;
  }
}
await writeFile(join(output, 'summary.json'), `${JSON.stringify(report, null, 2)}\n`);
console.log(`${options.case}: ${report.status}. ${JSON.stringify(report.counts)}. Results ${output}`);
if (report.errors.length) console.error(report.errors.map(error => typeof error === 'string' ? error : error.message).join('\n'));
