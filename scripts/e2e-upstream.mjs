// Run unchanged upstream Playwright assertions with the Rust browser engine.
import { readFile, writeFile, mkdir, realpath, statfs } from 'node:fs/promises';
import { resolve, dirname, join } from 'node:path';
import { fileURLToPath } from 'node:url';
import { randomUUID } from 'node:crypto';
import { runCommand } from './quality-process.mjs';
import { startFixtureServer } from './e2e/fixture-server.mjs';
import { configForRust, testCounts } from './e2e/upstream-config.mjs';
import { buildBootstrap } from './e2e/build-bootstrap.mjs';

const root = resolve(dirname(fileURLToPath(import.meta.url)), '..');
const options = { suite: 'core', timeout: 300000, list: false, build: true };
const args = process.argv.slice(2);
for (let index = 0; index < args.length; index++) {
  const key = args[index];
  if (key === '--list') options.list = true;
  else if (key === '--no-build') options.build = false;
  else if (['--suite', '--project', '--grep', '--timeout-ms'].includes(key)) {
    const value = args[++index];
    if (!value || value.startsWith('--')) throw new Error(`${key} requires a value`);
    options[key === '--timeout-ms' ? 'timeout' : key.slice(2)] = key === '--timeout-ms' ? Number(value) : value;
  } else throw new Error(`Unknown argument ${key}`);
}
if (!Number.isSafeInteger(options.timeout) || options.timeout < 1000 || options.timeout > 2147453647) throw new Error('Invalid bounded timeout');
if (options.project && !['Chromium', 'FirefoxStable', 'WebKit'].includes(options.project)) throw new Error('Use an original Playwright project name');
const reference = await realpath(join(root, 'source_repo'));
const wrapper = await readFile(join(reference, 'bin/test_integration.js'), 'utf8');
const addonArray = wrapper.match(/const addons = \[([\s\S]*?)\];/);
if (!addonArray) throw new Error('Original addon suite inventory is missing');
const addons = [...addonArray[1].matchAll(/'([^']+)'/g)].map(match => match[1]);
if (options.suite !== 'core' && !addons.map(name => `addon-${name}`).includes(options.suite)) throw new Error('Unknown original suite');
const configuration = options.suite === 'core' ? 'test/playwright/playwright.config.ts'
  : `addons/${options.suite}/test/playwright.config.ts`;
const originalConfig = join(reference, configuration);
await readFile(originalConfig); // Playwright transforms original TypeScript without changing its assertions.
const id = `${new Date().toISOString().replace(/[:.]/g, '-')}-${randomUUID()}`;
const output = join(root, 'results/e2e/upstream', id);
const temporary = join(root, 'tmp/e2e', id);
await mkdir(output, { recursive: true }); await mkdir(temporary, { recursive: true });
const environment = { ...process.env, TMP: temporary, TEMP: temporary, TMPDIR: temporary };
const report = { status: 'failed', engine: 'Rust WASM', suite: options.suite, project: options.project ?? 'all original projects',
  mode: options.list ? 'discovery only' : 'runtime', platform: process.platform, node: process.version,
  counts: null, errors: [], scope: 'This run does not establish complete terminal or cross-platform equivalence.' };
async function diskCheck() {
  const disk = await statfs(root, { bigint: true });
  if (disk.bavail * disk.bsize < 1024n ** 3n) throw new Error('E2E needs 1 GiB free');
}
let server;
const controller = new AbortController();
const interrupt = () => controller.abort(new Error('E2E interrupted'));
process.once('SIGINT', interrupt); process.once('SIGTERM', interrupt);
try {
  if (!options.list && options.build) {
    const result = await runCommand(process.execPath, ['scripts/build-demo.mjs', '--debug'], {
      cwd: root, env: environment, log: join(output, 'build.log'), timeoutMs: options.timeout, resourceCheck: diskCheck, signal: controller.signal });
    if (result.status !== 'passed') throw new Error(`Rust debug build failed. See ${join(output, 'build.log')}`);
  }
  if (!options.list) {
    const bootstrapFile = join(temporary, 'rust-test-page.js');
    await buildBootstrap({ root, referenceRoot: reference, outputFile: bootstrapFile });
    server = await startFixtureServer({ root, packageRoot: join(root, 'browser/pkg'), bootstrapFile, referenceRoot: reference,
      timeoutMs: options.timeout + 30000 });
  }
  const config = join(temporary, 'playwright.cjs');
  await writeFile(config, configForRust(originalConfig, server?.origin ?? 'http://127.0.0.1:1'));
  const rawFile = join(output, 'playwright.json');
  const command = [join(reference, 'node_modules/@playwright/test/cli.js'), 'test', `--config=${config}`, '--reporter=json'];
  if (options.list) command.push('--list');
  else command.push('--workers=2', '--retries=0', `--global-timeout=${options.timeout - 500}`, `--output=${join(output, 'artifacts')}`);
  if (options.project) command.push(`--project=${options.project}`);
  if (options.grep) command.push(`--grep=${options.grep}`);
  report.command = [process.execPath, ...command];
  const result = await runCommand(process.execPath, command, { cwd: reference,
    env: { ...environment, CI: '1', FORCE_COLOR: '0', PLAYWRIGHT_JSON_OUTPUT_FILE: rawFile },
    log: join(output, 'playwright.log'), timeoutMs: options.timeout, resourceCheck: diskCheck, signal: controller.signal });
  const raw = JSON.parse(await readFile(rawFile, 'utf8'));
  report.counts = testCounts(raw);
  report.errors = raw.errors ?? [];
  report.playwrightStats = raw.stats;
  if (!report.counts.discovered) throw new Error('No original tests discovered');
  if (!options.list && !report.counts.executed) throw new Error('No Rust-backed browser assertions executed');
  if (controller.signal.aborted) throw controller.signal.reason;
  if (result.status !== 'passed' || report.errors.length) throw new Error('Original Playwright assertions failed against Rust');
  report.status = options.list ? 'discovered, not executed' : 'passed';
} catch (error) { report.errors.push(String(error.stack ?? error)); process.exitCode = 1; }
finally {
  process.off('SIGINT', interrupt); process.off('SIGTERM', interrupt);
  if (server) {
    try { await server.close(); } catch (error) { report.errors.push(String(error)); process.exitCode = 1; report.status = 'failed'; }
    report.requests = server.requests;
    report.serverErrors = server.failures;
    report.runtimes = server.runtimes;
    if (server.failures.length) { report.status = 'failed'; process.exitCode = 1; }
  }
}
await writeFile(join(output, 'summary.json'), `${JSON.stringify(report, null, 2)}\n`);
console.log(`${options.suite}: ${report.status}. ${JSON.stringify(report.counts)}. Results ${output}`);
if (report.errors.length) console.error(report.errors.map(error => typeof error === 'string' ? error : error.message).join('\n'));
