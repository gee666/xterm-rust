// Fault injection executes the real entrypoints, not browsers or terminal logic.
import test from 'node:test';
import assert from 'node:assert/strict';
import { readFile } from 'node:fs/promises';
import vm from 'node:vm';
import { spawnSync } from 'node:child_process';
import * as path from 'node:path';
import * as url from 'node:url';
import * as crypto from 'node:crypto';
import * as upstream from './upstream-config.mjs';

const passed = { expectedStatus: 'passed', results: [{ status: 'passed' }] };
const rawReport = (...tests) => ({ errors: [], suites: [{ specs: [{ tests }] }] });

async function run(name, raw, { list = false, commandStatus = 'passed' } = {}) {
  const file = new URL(`../${name}`, import.meta.url);
  const writes = new Map();
  let closes = 0;
  const processStub = { argv: ['node', url.fileURLToPath(file), '--no-build', ...(list ? ['--list'] : [])],
    env: {}, platform: process.platform, version: process.version, execPath: process.execPath,
    once() {}, off() {} };
  const context = vm.createContext({ process: processStub, AbortController, console: { log() {}, error() {} } });
  const server = { origin: 'http://127.0.0.1:1', requests: [], failures: [], runtimes: [], socketEvents: [],
    async close() { closes++; } };
  const mocks = {
    'node:fs/promises': {
      async readFile(file) {
        if (String(file).replaceAll('\\', '/').endsWith('bin/test_integration.js')) return "const addons = ['fit'];";
        if (String(file).endsWith('playwright.json')) return JSON.stringify(raw);
        return '';
      },
      async writeFile(file, text) { writes.set(file, text); }, async mkdir() {},
      async realpath(file) { return file; }, async statfs() { return { bavail: 10n ** 12n, bsize: 4096n }; }
    },
    'node:path': path, 'node:url': url, 'node:crypto': crypto,
    './quality-process.mjs': { async runCommand() { return { status: commandStatus, code: commandStatus === 'passed' ? 0 : 1 }; } },
    './serve-demo.mjs': { async startDemoServer() { return server; } },
    './e2e/fixture-server.mjs': { async startFixtureServer() { return server; } },
    './e2e/build-bootstrap.mjs': { async buildBootstrap() {} },
    './e2e/upstream-config.mjs': upstream,
    './e2e/build-identity.mjs': {
      async artifactIdentity() { return []; }, async sourceIdentity() { return { sha256: 'fixed-unit-identity' }; },
      async verifyBuiltPackage() { return { report: 'unit-only' }; }
    }
  };
  const module = new vm.SourceTextModule(await readFile(file, 'utf8'), {
    context, identifier: file.href, initializeImportMeta(meta) { meta.url = file.href; }
  });
  await module.link(specifier => {
    const exports = mocks[specifier];
    assert.ok(exports, `Unexpected runner dependency ${specifier}`);
    return new vm.SyntheticModule(Object.keys(exports), function () {
      for (const [name, value] of Object.entries(exports)) this.setExport(name, value);
    }, { context });
  });
  await module.evaluate({ timeout: 10000 });
  const summary = [...writes].find(([file]) => file.endsWith('summary.json'));
  assert.ok(summary, 'The actual runner must retain a summary');
  return { summary: JSON.parse(summary[1]), exitCode: processStub.exitCode ?? 0, closes };
}

// Keep the documented `node --test scripts/e2e/*.test.mjs` command usable.
// Only the isolated fault-injection child needs Node's VM module flag.
if (!vm.SourceTextModule) {
  test('entrypoint report controls in bounded VM child', () => {
    const child = spawnSync(process.execPath, ['--experimental-vm-modules', '--test', url.fileURLToPath(import.meta.url)],
      { encoding: 'utf8', timeout: 30000, windowsHide: true });
    assert.ifError(child.error);
    assert.equal(child.status, 0, `${child.stdout}\n${child.stderr}`);
  });
} else for (const name of ['e2e-upstream.mjs', 'demo-e2e.mjs']) {
  test(`${name} keeps complete runtime and discovery outcomes distinct`, async () => {
    const complete = await run(name, rawReport(passed));
    assert.equal(complete.summary.status, 'passed');
    assert.equal(complete.exitCode, 0);
    assert.equal(complete.closes, 1);
    const discovery = await run(name, rawReport({ results: [] }), { list: true });
    assert.equal(discovery.summary.status, 'discovered, not executed');
    assert.equal(discovery.summary.counts.unrun, 1);
    assert.equal(discovery.exitCode, 0);
    assert.equal(discovery.closes, 0);
  });
  for (const [label, other] of [
    ['missing result', {}], ['beforeAll fallout', { expectedStatus: 'passed', results: [{ status: 'skipped' }] }],
    ['unknown status', { results: [{ status: 'success' }] }],
    ['unexpected failure', { expectedStatus: 'passed', results: [{ status: 'failed' }] }]
  ]) {
    test(`${name} rejects exit0 with ${label} and retains cleanup/accounting`, async () => {
      const outcome = await run(name, rawReport(passed, other));
      assert.equal(outcome.summary.status, 'failed');
      assert.equal(outcome.exitCode, 1);
      assert.equal(outcome.summary.counts.discovered, 2);
      assert.ok(outcome.summary.errors.length > 0);
      assert.equal(outcome.closes, 1);
    });
  }
  test(`${name} retains a failure summary for malformed suite errors`, async () => {
    for (const errors of ['corrupt errors', null]) {
      const outcome = await run(name, { ...rawReport(passed), errors });
      assert.equal(outcome.summary.status, 'failed');
      assert.equal(outcome.exitCode, 1);
      assert.ok(Array.isArray(outcome.summary.errors));
      assert.ok(outcome.summary.errors.length > 0);
      assert.equal(outcome.closes, 1);
    }
  });
  test(`${name} still rejects a failed command beside a complete passing report`, async () => {
    const outcome = await run(name, rawReport(passed), { commandStatus: 'failed' });
    assert.equal(outcome.summary.status, 'failed');
    assert.equal(outcome.exitCode, 1);
    assert.equal(outcome.closes, 1);
  });
}
