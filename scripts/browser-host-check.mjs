import { createServer } from 'node:http';
import { mkdir, readFile, writeFile, statfs } from 'node:fs/promises';
import { resolve, dirname, sep } from 'node:path';
import { fileURLToPath, pathToFileURL } from 'node:url';
import { runCommand } from './quality-process.mjs';

// Runs Rust wasm-bindgen-test assertions with Playwright, without WebDriver or
// the upstream JavaScript terminal engine. The runtime remains wasm-bindgen's.
const root = resolve(dirname(fileURLToPath(import.meta.url)), '..');
const packageName = process.env.WASM_PACKAGE ?? 'xterm-web';
if (!['xterm-web', 'xterm-font-addons'].includes(packageName)) throw new Error('Invalid WASM_PACKAGE');
const testName = process.env.WASM_TEST ?? 'wasm_host';
if (!/^[a-z0-9_]+$/.test(testName)) throw new Error('Invalid WASM_TEST name');
const work = resolve(root, 'tmp/browser-host', testName);
const resultRoot = resolve(root, 'results/browser-host');
const runId = new Date().toISOString().replace(/[:.]/g, '-');
const results = resolve(resultRoot, testName, runId);
await mkdir(work, { recursive: true });
await mkdir(results, { recursive: true });
const bindgen = process.env.WASM_BINDGEN ?? resolve(root,
  'tmp/wasm-tools/wasm-bindgen-0.2.100-x86_64-pc-windows-msvc/wasm-bindgen.exe');
const cargo = process.env.CARGO ?? 'cargo';
const commands = [];
async function headroom() {
  const disk = await statfs(root);
  const bytes = Number(disk.bavail) * Number(disk.bsize);
  if (bytes < 512 * 1024 * 1024) throw new Error(`Build disk headroom below 512MiB: ${bytes}`);
}
async function command(executable, args, log) {
  await headroom();
  const record = await runCommand(executable, args, {
    cwd: root, env: process.env, log: resolve(results, log),
    timeoutMs: 180_000, resourceCheck: headroom,
  });
  commands.push(record);
  await writeFile(resolve(results, 'commands.json'), JSON.stringify(commands, null, 2) + '\n');
  if (record.status !== 'passed') throw new Error(`Command failed; see ${results}/${log}`);
}
await command(cargo, ['test', '-p', packageName, '--target', 'wasm32-unknown-unknown',
  '--test', testName, '--no-run', '--message-format=json'], `${testName}-build.log`);
const messages = (await readFile(resolve(results, `${testName}-build.log`), 'utf8')).split('\n')
  .filter(line => line.startsWith('{')).map(line => JSON.parse(line));
const artifact = messages.find(message => message.reason === 'compiler-artifact'
  && message.target.name === testName && message.profile.test);
const wasm = artifact?.filenames.find(name => name.endsWith('.wasm'));
if (!wasm) throw new Error('No WASM test artifact in cargo output');
await command(bindgen, [wasm, '--target', 'web', '--out-dir', work,
  '--out-name', 'suite'], `${testName}-bindgen.log`);
const playwrightPath = process.env.PLAYWRIGHT_MODULE ?? resolve(root,
  'source_repo/node_modules/playwright/index.mjs');
const playwright = await import(pathToFileURL(playwrightPath));
const html = '<!doctype html><meta charset="utf-8"><title>Rust WASM assertions</title><pre id="output"></pre>';
const server = createServer(async (request, response) => {
  try {
    const pathname = new URL(request.url, 'http://localhost').pathname;
    if (pathname === '/') {
      response.setHeader('Content-Type', 'text/html; charset=utf-8');
      response.end(html);
      return;
    }
    const path = resolve(work, '.' + decodeURIComponent(pathname));
    if (!path.startsWith(work + sep)) {
      response.writeHead(403).end();
      return;
    }
    const data = await readFile(path);
    response.setHeader('Content-Type', path.endsWith('.wasm') ? 'application/wasm' : 'text/javascript');
    response.end(data);
  } catch (error) {
    response.writeHead(error.code === 'ENOENT' ? 404 : 500).end(error.message);
  }
});
await new Promise((ready, reject) => {
  server.once('error', reject);
  server.listen(0, '127.0.0.1', ready);
});
const report = { testName, platform: process.platform, node: process.version,
  date: new Date().toISOString(), componentOnly: true, commands, browsers: [] };
try {
  const address = server.address();
  for (const name of (process.env.BROWSERS ?? 'chromium,firefox,webkit').split(',')) {
    const entry = { name, status: 'unrun', tests: [], errors: [], console: [] };
    report.browsers.push(entry);
    let browser;
    try {
      if (!['chromium', 'firefox', 'webkit'].includes(name)) throw new Error(`Unknown browser ${name}`);
      browser = await playwright[name].launch({ headless: true, timeout: 30_000 });
      entry.version = browser.version();
      const page = await browser.newPage();
      page.setDefaultTimeout(60_000);
      page.on('pageerror', error => entry.errors.push(error.stack ?? error.message));
      page.on('console', event => entry.console.push({ type: event.type(), text: event.text() }));
      await page.goto(`http://127.0.0.1:${address.port}/`);
      await page.evaluate(() => {
        globalThis.__wbg_test_invoke = callback => callback();
        globalThis.testRun = (async () => {
          const bindings = await import('/suite.js');
          const exports = await bindings.default();
          const names = Object.keys(exports).filter(name => name.startsWith('__wbgt_')).sort();
          if (!names.length) throw new Error('No exported wasm-bindgen-test assertions found');
          const context = new bindings.WasmBindgenTestContext();
          try {
            const passed = await context.run(names.map(name => exports[name]));
            return { names, passed, output: document.querySelector('#output').textContent };
          } finally { context.free(); }
        })();
        globalThis.testRun.then(value => { globalThis.testResult = value; }, error => {
          globalThis.testResult = { error: error.stack ?? String(error) };
        });
      });
      await page.waitForFunction(() => globalThis.testResult !== undefined);
      const result = await page.evaluate(() => globalThis.testResult);
      entry.tests = result.names ?? [];
      entry.output = result.output;
      entry.status = result.passed && entry.errors.length === 0 ? 'passed' : 'failed';
      if (result.error) entry.errors.push(result.error);
      const summary = result.output?.match(/test result: (?:ok|FAILED)\. (\d+) passed; (\d+) failed; (\d+) ignored; (\d+) filtered out/);
      if (summary) entry.counts = Object.fromEntries(['passed', 'failed', 'ignored', 'filtered'].map((key, index) => [key, Number(summary[index + 1])]));
      await writeFile(resolve(results, `${testName}-${name}.log`), result.output ?? JSON.stringify(result, null, 2));
      console.log(`${name} ${entry.version}: ${entry.status}, ${entry.tests.length} Rust assertions`);
    } catch (error) {
      entry.errors.push(error.stack ?? error.message);
      console.error(`${name}: ${error.message}`);
    } finally {
      if (browser) await browser.close();
    }
  }
} finally {
  await new Promise((done, reject) => server.close(error => error ? reject(error) : done()));
  const json = JSON.stringify(report, null, 2) + '\n';
  await writeFile(resolve(results, `${testName}-results.json`), json);
  await writeFile(resolve(resultRoot, `${testName}-results.json`), json);
}
if (report.browsers.some(browser => browser.status !== 'passed')) process.exitCode = 1;
