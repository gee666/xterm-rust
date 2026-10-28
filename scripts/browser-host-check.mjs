import { createHash, randomUUID } from 'node:crypto';
import { access, mkdir, readFile, writeFile, statfs } from 'node:fs/promises';
import { resolve, dirname } from 'node:path';
import { fileURLToPath, pathToFileURL } from 'node:url';
import { runCommand } from './quality-process.mjs';
import { pinArtifact } from './acceptance-artifacts.mjs';
import { withArtifactServer } from './browser-artifact-server.mjs';

// Rust wasm-bindgen-test assertions, not the reference JavaScript terminal.
const root = resolve(dirname(fileURLToPath(import.meta.url)), '..');
const packageName = process.env.WASM_PACKAGE ?? 'xterm-web';
if (!['xterm-web', 'xterm-wasm-support'].includes(packageName)) throw new Error('Invalid WASM_PACKAGE');
const testName = process.env.WASM_TEST ?? 'wasm_host';
if (!/^[a-z0-9_]+$/.test(testName)) throw new Error('Invalid WASM_TEST');
const features = process.env.CARGO_FEATURES ?? '';
if (features && !/^[a-zA-Z0-9_-]+(?:,[a-zA-Z0-9_-]+)*$/.test(features)) throw new Error('Invalid CARGO_FEATURES');
const key = packageName === 'xterm-web' ? testName : `${packageName}__${testName}`;
const resultRoot = resolve(root, 'results/browser-host');
const runId = `${new Date().toISOString().replace(/[:.]/g, '-')}-${randomUUID()}`;
const results = resolve(resultRoot, key, runId);
const work = resolve(results, 'artifacts');
await mkdir(work, { recursive: true });
const commands = [];
const report = { package: packageName, testName, features: features.split(',').filter(Boolean),
  platform: process.platform, node: process.version, date: new Date().toISOString(),
  componentOnly: true, status: 'preparing', commands, browsers: [] };
async function persist() {
  const json = JSON.stringify(report, null, 2) + '\n';
  await writeFile(resolve(results, `${testName}-results.json`), json);
  await writeFile(resolve(resultRoot, `${key}-results.json`), json);
}
await persist(); // A failed build must not leave an older pass labelled latest.
const sha256 = bytes => createHash('sha256').update(bytes).digest('hex');
async function headroom() {
  for (const location of [root, resolve(root, 'target')]) {
    const disk = await statfs(location, { bigint: true });
    if (disk.bavail * disk.bsize < 1024n ** 3n) throw new Error(`Build disk headroom below 1GiB: ${location}`);
  }
}
async function command(executable, args, log) {
  await headroom();
  const record = await runCommand(executable, args, {
    cwd: root, env: process.env, log: resolve(results, log),
    timeoutMs: 180_000, resourceCheck: headroom,
  });
  commands.push(record);
  await writeFile(resolve(results, 'commands.json'), JSON.stringify(commands, null, 2) + '\n');
  await persist();
  if (record.status !== 'passed') throw new Error(`Command failed; see ${results}/${log}`);
}
async function bindgenPath() {
  if (process.env.WASM_BINDGEN) return process.env.WASM_BINDGEN;
  if (process.platform === 'win32') {
    const local = resolve(root, 'tmp/wasm-tools/wasm-bindgen-0.2.100-x86_64-pc-windows-msvc/wasm-bindgen.exe');
    try { await access(local); return local; } catch (error) { if (error.code !== 'ENOENT') throw error; }
  }
  return 'wasm-bindgen';
}
try {
  const isLib = testName === 'lib';
  await command(process.env.CARGO ?? 'cargo', ['test', '-p', packageName, '--target', 'wasm32-unknown-unknown',
    ...(isLib ? ['--lib'] : ['--test', testName]), ...(features ? ['--features', features] : []),
    '--no-run', '--message-format=json'], `${testName}-build.log`);
  const messages = (await readFile(resolve(results, `${testName}-build.log`), 'utf8')).split('\n')
    .filter(line => line.startsWith('{')).map(line => JSON.parse(line));
  const artifact = messages.find(message => message.reason === 'compiler-artifact'
    && message.target.name === (isLib ? packageName.replaceAll('-', '_') : testName) && message.profile.test);
  const compilerPath = artifact?.filenames.find(name => name.endsWith('.wasm'));
  if (!compilerPath) throw new Error('No WASM test artifact in cargo output');
  // The shared target may be a junction. Reuse the strict cache-aware pinning
  // helper rather than relaxing source/result symlink guards.
  const compilerPin = await pinArtifact(root, compilerPath);
  const bindgen = await bindgenPath();
  await command(bindgen, ['--version'], `${testName}-bindgen-version.log`);
  if ((await readFile(resolve(results, `${testName}-bindgen-version.log`), 'utf8')).trim() !== 'wasm-bindgen 0.2.100') {
    throw new Error('wasm-bindgen CLI must match the locked 0.2.100 runtime');
  }
  await command(bindgen, [resolve(root, compilerPin.path), '--target', 'web', '--out-dir', work,
    '--out-name', 'suite'], `${testName}-bindgen.log`);
  const playwright = await import(pathToFileURL(process.env.PLAYWRIGHT_MODULE ?? resolve(root,
    'source_repo/node_modules/playwright/index.mjs')));
  const css = await readFile(resolve(root, 'crates/xterm-web/assets/xterm.css'));
  const wasmPath = resolve(work, 'suite_bg.wasm');
  const bindingsPath = resolve(work, 'suite.js');
  const wasmBytes = await readFile(wasmPath);
  const bindingsBytes = await readFile(bindingsPath);
  report.artifacts = { compiler: { ...compilerPin, originalPath: compilerPath },
    wasm: { path: wasmPath, bytes: wasmBytes.length, sha256: sha256(wasmBytes) },
    bindings: { path: bindingsPath, bytes: bindingsBytes.length, sha256: sha256(bindingsBytes) },
    css: { path: 'crates/xterm-web/assets/xterm.css', bytes: css.length, sha256: sha256(css) },
    cargoLockSha256: sha256(await readFile(resolve(root, 'Cargo.lock'))),
    testSourceSha256: sha256(await readFile(isLib ? resolve(root, 'crates', packageName, 'src/lib.rs')
      : resolve(root, 'crates', packageName, 'tests', `${testName}.rs`))) };
  await persist();
  const html = '<!doctype html><meta charset="utf-8"><link rel="stylesheet" href="/xterm.css"><title>Rust WASM assertions</title><pre id="output"></pre>';
  // The helper's finally owns the socket across all body/close failures.
  await withArtifactServer({ directory: work, css, html }, async baseUrl => {
  report.status = 'running';
  await persist();
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
      await page.goto(baseUrl);
      await page.evaluate(() => {
        globalThis.__wbg_test_invoke = callback => callback();
        globalThis.testRun = (async () => {
          const bindings = await import('/suite.js');
          const exports = await bindings.default();
          const names = Object.keys(exports).filter(name => name.startsWith('__wbgt_')).sort();
          globalThis.testNames = names;
          if (!names.length) throw new Error('No exported wasm-bindgen-test assertions found');
          const context = new bindings.WasmBindgenTestContext();
          try {
            const passed = await context.run(names.map(name => exports[name]));
            return { names, passed, output: document.querySelector('#output').textContent };
          } finally { context.free(); }
        })();
        globalThis.testRun.then(value => { globalThis.testResult = value; }, error => {
          globalThis.testResult = { names: globalThis.testNames ?? [], error: error.stack ?? String(error) };
        });
      });
      await page.waitForFunction(() => globalThis.testResult !== undefined);
      const result = await page.evaluate(() => globalThis.testResult);
      entry.tests = result.names ?? [];
      entry.output = result.output;
      entry.status = result.passed && entry.errors.length === 0 ? 'passed' : 'failed';
      if (result.error) entry.errors.push(result.error);
      const summary = result.output?.match(/test result: (?:ok|FAILED)\. (\d+) passed; (\d+) failed; (\d+) ignored; (\d+) filtered out/);
      if (summary) entry.counts = Object.fromEntries(['passed', 'failed', 'ignored', 'filtered'].map((field, index) => [field, Number(summary[index + 1])]));
      await writeFile(resolve(results, `${testName}-${name}.log`), result.output ?? JSON.stringify(result, null, 2));
      console.log(`${name} ${entry.version}: ${entry.status}, ${entry.tests.length} Rust assertions`);
    } catch (error) {
      if (browser) entry.status = 'failed';
      entry.errors.push(error.stack ?? error.message);
      console.error(`${name}: ${error.message}`);
    } finally { if (browser) await browser.close(); }
  }
  });
  report.status = report.browsers.every(browser => browser.status === 'passed') ? 'passed' : 'failed';
  if (report.status !== 'passed') process.exitCode = 1;
} catch (error) {
  report.status = 'failed';
  report.error = error.stack ?? error.message;
  process.exitCode = 1;
  console.error(report.error);
} finally {
  report.finishedAt = new Date().toISOString();
  await persist();
}
