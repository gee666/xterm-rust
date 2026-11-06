import assert from 'node:assert/strict';
import { readFile, writeFile, mkdir } from 'node:fs/promises';
import { execFileSync } from 'node:child_process';
import { join } from 'node:path';
import { createHash } from 'node:crypto';
import { startDemoServer } from '../scripts/serve-demo.mjs';
import { runCommand } from '../scripts/quality-process.mjs';
import { sourceIdentity, verifyBuiltPackage } from '../scripts/e2e/build-identity.mjs';
import { testCounts } from '../scripts/e2e/upstream-config.mjs';
const root = 'O:/xterm-rust-swarm-16/tmp/worktrees/Chagatai566';
const output = join(root, 'results/link-demo-reload');
const probePath = join(root, 'scripts/demo-e2e/addon-probes.cjs');
const fixed = await readFile(probePath);
const old = execFileSync('C:/Program Files/Git/cmd/git.exe', ['-C', root, 'show', 'main:scripts/demo-e2e/addon-probes.cjs']);
const hash = value => createHash('sha256').update(value).digest('hex');
const source = await sourceIdentity(root);
assert.equal(source.sha256, '19ffb731fe479eb0630bf8b01c74464c7be91ae039ab4641ab11e56d18b7cf95');
const report = { source, build: await verifyBuiltPackage(root, source), grant: '409#5033',
  scope: 'Three original custom demo web-links cases only, unchanged assertions, old-pointer negative then fixed physical path. Not full demo or addon parity.',
  probeOldSha256: hash(old), probeFixedSha256: hash(fixed), attempts: [], status: 'running' };
const host = await startDemoServer({ port: 0, timeoutMs: 300000 });
try {
  for (const [name, bytes] of [['old', old], ['fixed', fixed]]) {
    await writeFile(probePath, bytes);
    const stage = join(output, `rust-${name}`);
    await mkdir(stage, { recursive: true });
    const rawFile = join(stage, 'playwright.json');
    const args = [join(root, 'source_repo/node_modules/@playwright/test/cli.js'), 'test',
      '--config=tmp/link-playwright.cjs', '--grep=web-links load, reload, behavior and unload', '--reporter=json',
      '--workers=2', '--retries=0', '--global-timeout=119500', `--output=${join(stage, 'artifacts')}`];
    const command = await runCommand(process.execPath, args, { cwd: root, timeoutMs: 120000,
      log: join(stage, 'driver.log'), env: { ...process.env, DEMO_ORIGIN: host.origin,
        PLAYWRIGHT_JSON_OUTPUT_FILE: rawFile, CI: '1', FORCE_COLOR: '0' } });
    const raw = JSON.parse(await readFile(rawFile, 'utf8'));
    const counts = testCounts(raw);
    report.attempts.push({ name, command, counts, errors: raw.errors ?? [], rawFile });
    console.log(name, JSON.stringify(counts));
    assert.equal(counts.discovered, 3);
    assert.equal(counts.skipped, 0);
    assert.equal(counts.unrun, 0);
    assert.equal((raw.errors ?? []).length, 0);
    if (name === 'old') {
      assert.equal(counts.passed, 0);
      assert.equal(counts.failed, 3);
      const errors = [];
      const walk = suite => {
        for (const spec of suite.specs ?? []) for (const test of spec.tests) for (const result of test.results) errors.push(...result.errors ?? []);
        for (const child of suite.suites ?? []) walk(child);
      };
      for (const suite of raw.suites) walk(suite);
      assert.equal(errors.length, 3);
      assert(errors.every(error => error.message.includes(".xterm-cursor-pointer")), 'Old failures must be the intended hover assertion');
    } else {
      assert.equal(command.status, 'passed');
      assert.equal(counts.passed, 3);
      assert.equal(counts.failed, 0);
    }
  }
  report.status = 'passed scoped negative and fixed controls';
} catch (error) {
  report.status = 'failed'; report.error = String(error.stack ?? error); throw error;
} finally {
  await writeFile(probePath, fixed);
  assert.equal(hash(await readFile(probePath)), hash(fixed));
  await host.close();
  report.serverClosed = true;
  report.requests = host.requests;
  assert.equal((await sourceIdentity(root)).sha256, source.sha256);
  await writeFile(join(output, 'rust-summary.json'), JSON.stringify(report, null, 2) + '\n');
}
