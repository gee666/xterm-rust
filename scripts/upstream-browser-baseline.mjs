// Discover or run original Playwright suites. Never substitutes Rust evidence.
import fs from 'node:fs';
import path from 'node:path';
import net from 'node:net';
import { fileURLToPath } from 'node:url';
import { referenceIntegrity, sha256, createEvidenceRun, persistEvidence } from './upstream-baseline-utils.mjs';

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..');
const source = path.join(root, 'source_repo');
const evidence = createEvidenceRun(path.join(root, 'results/upstream-browser'), { kind: 'original-browser-baseline' });
const output = evidence.directory;
const relativeArtifact = file => path.relative(root, file).replaceAll('\\', '/');
const manifest = JSON.parse(fs.readFileSync(path.join(root, 'docs/baseline/reference-manifest.json'), 'utf8'));
const options = new Map(process.argv.slice(2).map(argument => {
  const [key, ...value] = argument.split('=');
  return [key, value.join('=') || true];
}));
for (const key of options.keys()) {
  if (!['--run', '--suite', '--project', '--timeout-ms', '--original-server'].includes(key)) throw new Error(`Unknown option: ${key}`);
}
const execute = options.has('--run');
const timeoutMs = Number(options.get('--timeout-ms') ?? 300_000);
const MAX_TIMEOUT_MS = 2_147_483_647;
if (!Number.isSafeInteger(timeoutMs) || timeoutMs <= 0 || timeoutMs > MAX_TIMEOUT_MS) throw new Error('Invalid process timeout');
const manageServer = execute && !options.has('--original-server');
const project = options.get('--project');
if (project && !['Chromium', 'FirefoxStable', 'WebKit'].includes(project)) throw new Error('Unknown original project');
const wrapper = fs.readFileSync(path.join(source, 'bin/test_integration.js'), 'utf8');
const addonArray = wrapper.match(/const addons = \[([\s\S]*?)\];/);
if (!addonArray) throw new Error('Original integration suite list was not found');
const addons = [...addonArray[1].matchAll(/'([^']+)'/g)].map(match => match[1]);
let configs = [
  { name: 'core', source: 'test/playwright/playwright.config.ts', generated: 'out-esbuild-test/playwright/playwright.config.js' },
  ...addons.map(name => ({ name: `addon-${name}`, source: `addons/addon-${name}/test/playwright.config.ts`,
    generated: `addons/addon-${name}/out-esbuild-test/playwright.config.js` }))
];
for (const addon of fs.readdirSync(path.join(source, 'addons'))) {
  const config = `addons/${addon}/test/playwright.config.ts`;
  if (fs.existsSync(path.join(source, config)) && !configs.some(item => item.source === config)) {
    throw new Error(`Original wrapper omits an existing browser config: ${config}`);
  }
}
const allConfigs = configs.map(config => config.name);
if (options.has('--suite')) {
  configs = configs.filter(config => config.name === options.get('--suite'));
  if (!configs.length) throw new Error('Unknown original suite');
}
const before = referenceIntegrity(source, manifest);
if (before.mismatches.length) throw new Error('Reference changed before browser baseline');
const report = {
  schemaVersion: 1, mode: execute ? 'runtime' : 'discovery-only',
  environment: { node: process.version, platform: process.platform, arch: process.arch,
    playwright: JSON.parse(fs.readFileSync(path.join(source, 'node_modules/@playwright/test/package.json'), 'utf8')).version },
  sourceWrapper: { path: 'source_repo/bin/test_integration.js', sha256: sha256(wrapper) },
  configInventory: allConfigs, selectedProject: project ?? 'all original projects',
  serverAdapter: manageServer ? 'Owned node demo/start.js; original webServer disabled in temporary config; same original testDir, assertions and projects.' : 'Original webServer command unchanged',
  referenceBefore: before, suites: [], status: 'passed',
  scope: 'Original JavaScript only. Discovery alone executes no browser tests. Rust-backed and other native-platform evidence remain separate.'
};
const MIN_FREE_BYTES = 256 * 1024 * 1024;
function diskCheck() {
  const disk = fs.statfsSync(root);
  if (disk.bavail * disk.bsize < MIN_FREE_BYTES) throw new Error('Browser baseline stopped below 256 MiB free');
}
function configPath(config) {
  if (!manageServer) return config.generated;
  const directory = path.join(root, 'tmp/upstream-browser-config', evidence.id);
  fs.mkdirSync(directory, { recursive: true });
  const file = path.join(directory, `${config.name}.cjs`);
  const original = path.join(source, config.generated);
  fs.writeFileSync(file, `const loaded = require(${JSON.stringify(original)});\n` +
    'const config = loaded.default ?? loaded;\n' +
    `const path = require('node:path');\n` +
    `if (!config.webServer || Array.isArray(config.webServer) || config.webServer.port !== 3000) throw new Error('Expected one original webServer on port 3000');\n` +
    `module.exports = { ...config, testDir: path.resolve(${JSON.stringify(path.dirname(original))}, config.testDir ?? '.'),\n` +
    "use: { ...config.use, baseURL: config.use?.baseURL ?? 'http://localhost:3000' }, webServer: undefined };\n");
  return path.relative(source, file);
}

function collectTests(suites, parents = []) {
  return suites.flatMap(suite => {
    const names = [...parents, suite.title];
    return [
      ...(suite.specs ?? []).flatMap(spec => (spec.tests ?? []).map(test => ({
        source: spec.file ?? suite.file, title: [...names, spec.title].join(' / '),
        project: test.projectName, expectedStatus: test.expectedStatus,
        results: test.results ?? [], outcome: test.status ?? null
      }))),
      ...collectTests(suite.suites ?? [], names)
    ];
  });
}
async function requireUnusedPort() {
  await new Promise((resolve, reject) => {
    const socket = net.createConnection({ host: '127.0.0.1', port: 3000 });
    socket.setTimeout(1000);
    socket.once('connect', () => { socket.destroy(); reject(new Error('Port 3000 is already in use; refusing to reuse or kill another server')); });
    socket.once('error', error => {
      socket.destroy();
      if (error.code === 'ECONNREFUSED') resolve();
      else reject(error);
    });
    socket.once('timeout', () => { socket.destroy(); reject(new Error('Port 3000 availability check timed out')); });
  });
}

let ownedServer;
try {
  const { runCommand, startManagedProcess } = await import('./quality-process.mjs');
  if (manageServer) {
    await requireUnusedPort();
    diskCheck();
    ownedServer = startManagedProcess(process.execPath, ['demo/start.js'], {
      cwd: source, env: { ...process.env, FORCE_COLOR: '0' }, log: path.join(output, 'server.log'),
      timeoutMs: Math.min(MAX_TIMEOUT_MS, timeoutMs * configs.length + 30_000),
      startupTimeoutMs: 15_000, resourceCheck: diskCheck,
      ready: async ({ signal }) => {
        try {
          const response = await fetch('http://127.0.0.1:3000/test', { signal: AbortSignal.any([signal, AbortSignal.timeout(1000)]) });
          return response.ok;
        } catch (error) {
          if (error.cause?.code === 'ECONNREFUSED') return false;
          throw error;
        }
      }
    });
    await ownedServer.ready;
  }
  for (const config of configs) {
    diskCheck();
    if (!fs.existsSync(path.join(source, config.generated))) throw new Error(`Build original first; missing ${config.generated}`);
    const name = `${config.name}-${project ?? 'all'}-${execute ? 'runtime' : 'list'}`;
    const rawFile = path.join(output, `${name}.json`);
    const logFile = path.join(output, `${name}.log`);
    const args = ['node_modules/@playwright/test/cli.js', 'test', `--config=${configPath(config)}`, '--reporter=json'];
    if (!execute) args.push('--list');
    else args.push('--workers=2', '--retries=0', `--global-timeout=${Math.max(1, timeoutMs - 15_000)}`,
      `--output=../${relativeArtifact(path.join(output, `${name}-artifacts`))}`);
    if (project) args.push(`--project=${project}`);
    const command = await runCommand(process.execPath, args, {
      cwd: source, log: logFile, timeoutMs, resourceCheck: diskCheck,
      env: { ...process.env, CI: '1', FORCE_COLOR: '0', PLAYWRIGHT_JSON_OUTPUT_FILE: rawFile }
    });
    const entry = { name: config.name, sourceConfig: config.source, generatedConfig: config.generated,
      command: { ...command, command: ['node', ...args], cwd: 'source_repo' },
      log: path.relative(root, logFile).replaceAll('\\', '/'),
      raw: fs.existsSync(rawFile) ? path.relative(root, rawFile).replaceAll('\\', '/') : null };
    if (entry.raw) {
      const raw = JSON.parse(fs.readFileSync(rawFile, 'utf8'));
      const tests = collectTests(raw.suites ?? []);
      entry.tests = tests;
      entry.rawStats = raw.stats;
      entry.errors = raw.errors ?? [];
      entry.projects = Object.fromEntries([...new Set(tests.map(test => test.project))].map(name => {
        const selected = tests.filter(test => test.project === name);
        const counts = { discovered: selected.length, executed: 0, passed: 0, failed: 0, skipped: 0, unrun: 0 };
        for (const test of selected) {
          const result = test.results.at(-1);
          if (!result) counts.unrun++;
          else if (result.status === 'skipped') counts.skipped++;
          else {
            counts.executed++;
            if (result.status === 'passed') counts.passed++;
            else counts.failed++;
          }
        }
        return [name, counts];
      }));
      if (entry.errors.length) report.status = 'failed';
    } else report.status = 'failed';
    if (command.status !== 'passed') report.status = 'failed';
    report.suites.push(entry);
  }
} catch (error) {
  report.status = 'failed';
  report.error = String(error.stack);
} finally {
  if (ownedServer) {
    report.server = await ownedServer.stop();
    if (report.server.status !== 'stopped') report.status = 'failed';
    try { await requireUnusedPort(); }
    catch (error) { report.status = 'failed'; report.shutdownError = String(error.stack); }
  }
  report.referenceAfter = referenceIntegrity(source, manifest);
  if (report.referenceAfter.mismatches.length) report.status = 'failed';
  persistEvidence(evidence, report);
}
console.log(JSON.stringify({ mode: report.mode, status: report.status, summary: relativeArtifact(path.join(output, 'summary.json')),
  suites: report.suites.map(suite => ({ name: suite.name, projects: suite.projects, process: suite.command.status })) }));
if (report.status !== 'passed') process.exitCode = 1;
