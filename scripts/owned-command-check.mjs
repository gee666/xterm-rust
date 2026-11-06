// Positive and old-policy negative controls run inside a bounded outer Job.
// The negative orphan is never selected/killed by PID; closing that Job owns it.
import assert from 'node:assert/strict';
import { mkdir, mkdtemp, readFile, writeFile, rm, lstat } from 'node:fs/promises';
import { join } from 'node:path';
import { fileURLToPath } from 'node:url';
import { randomUUID, createHash } from 'node:crypto';
import { runCommand } from './quality-process.mjs';

if (process.platform !== 'win32') throw new Error('Real Windows Job controls require native Windows');
const root = fileURLToPath(new URL('../', import.meta.url));
const id = `${new Date().toISOString().replace(/[:.]/g, '-')}-${randomUUID()}`;
const output = join(root, 'results/owned-process562', id);
await mkdir(output, { recursive: true });
await mkdir(join(root, 'tmp'), { recursive: true });
const temporary = await mkdtemp(join(root, 'tmp', 'owned-command-check-'));
const controller = new AbortController();
const interrupt = () => controller.abort(new Error('Owned command controls interrupted'));
process.once('SIGINT', interrupt); process.once('SIGTERM', interrupt);
const report = { platform: process.platform, node: process.version, output, temporary, runs: [], status: 'failed' };
const inputs = ['scripts/windows-owned-job.cs', 'scripts/windows-owned-job.ps1', 'scripts/owned-command.mjs',
  'scripts/quality-process.mjs', 'scripts/owned-command.test.mjs', 'scripts/windows-owned-job.test.mjs',
  'scripts/owned-command-check.mjs', 'scripts/demo-e2e.mjs', 'scripts/e2e-upstream.mjs'];
async function pins() {
  return Object.fromEntries(await Promise.all(inputs.map(async name => [name,
    createHash('sha256').update(await readFile(join(root, name))).digest('hex')])));
}
try {
  report.inputsBefore = await pins();
  for (const negative of [false, true]) {
    const label = negative ? 'negative' : 'positive';
    const fixtures = join(temporary, label);
    await mkdir(fixtures);
    const log = join(output, `${label}.log`);
    const env = { ...process.env, TEMP: temporary, TMP: temporary, TMPDIR: temporary,
      XTERM_OWNED_JOB_FIXTURE_ROOT: fixtures, XTERM_OWNED_JOB_NEGATIVE: negative ? '1' : '0',
      XTERM_OWNED_JOB_NEGATIVE_CONTAINED: '1' };
    const result = await runCommand(process.execPath, ['--test', '--test-reporter=tap',
      'scripts/owned-command.test.mjs', 'scripts/windows-owned-job.test.mjs'], {
      cwd: root, env, log, timeoutMs: 90_000, signal: controller.signal, containDescendants: true
    });
    const raw = await readFile(log, 'utf8');
    const counts = Object.fromEntries(['tests', 'pass', 'fail', 'cancelled', 'skipped', 'todo'].map(key => {
      const value = raw.match(new RegExp(`^# ${key} (\\d+)$`, 'm'));
      assert.ok(value, `Missing ${key} summary in ${label}`); return [key, Number(value[1])];
    }));
    report.runs.push({ label, ...result, log, counts });
    assert.deepEqual(counts, { tests: 7, pass: negative ? 4 : 7, fail: negative ? 3 : 0, cancelled: 0, skipped: 0, todo: 0 });
    assert.equal(result.exitCode, negative ? 1 : 0);
    assert.equal(result.timedOut, false); assert.equal(result.aborted, false);
    assert.equal(result.terminationError, null);
    if (negative) assert.equal((raw.match(/detached orphan must not outlive the command/g) ?? []).length, 3);
    await rm(fixtures, { recursive: true });
  }
  report.inputsAfter = await pins();
  assert.deepEqual(report.inputsAfter, report.inputsBefore);
  report.status = 'passed positive7 and intended old-policy negative3 controls';
} catch (error) { report.error = error.stack ?? String(error); process.exitCode = 1; }
finally {
  process.off('SIGINT', interrupt); process.off('SIGTERM', interrupt);
  try {
    await rm(temporary, { recursive: true });
    try { await lstat(temporary); throw new Error('Owned fixture root still exists'); }
    catch (error) { if (error.code !== 'ENOENT') throw error; }
    report.temporaryRemoved = true;
  } catch (error) { report.cleanupError = String(error); process.exitCode = 1; }
  await writeFile(join(output, 'summary.json'), JSON.stringify(report, null, 2) + '\n');
}
console.log(JSON.stringify(report, null, 2));
