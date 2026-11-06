import test from 'node:test';
import assert from 'node:assert/strict';
import { mkdir, mkdtemp, readFile, rm } from 'node:fs/promises';
import { join } from 'node:path';
import { fileURLToPath } from 'node:url';
import { ownedCommand } from './owned-command.mjs';
import { runCommand } from './quality-process.mjs';

const root = fileURLToPath(new URL('../', import.meta.url));
test('containment leaves ordinary and non-Windows commands unchanged', () => {
  const args = ['one', 'two']; const env = { marker: 'retained' };
  for (const [platform, containDescendants] of [['win32', false], ['linux', true], ['darwin', true]]) {
    assert.deepEqual(ownedCommand('command', args, { cwd: root, env, containDescendants }, platform),
      { command: 'command', args, env });
  }
});
test('Windows containment passes a private data plan, never evaluated command text', () => {
  const env = { SystemRoot: 'C:\\Windows', marker: 'retained' };
  const launch = ownedCommand('program.exe', ['a "quote"', ''], { cwd: root, env, containDescendants: true }, 'win32');
  assert.match(launch.command, /WindowsPowerShell[\\/]v1.0[\\/]powershell.exe$/);
  assert.ok(launch.args.includes('-File'));
  assert.deepEqual(JSON.parse(launch.env.XTERM_OWNED_JOB_COMMAND), {
    executable: 'program.exe', arguments: ['a "quote"', ''], cwd: root
  });
  assert.deepEqual(env, { SystemRoot: 'C:\\Windows', marker: 'retained' });
});
test('both Playwright runners contain descendants and pin the native boundary', async () => {
  for (const name of ['demo-e2e.mjs', 'e2e-upstream.mjs']) {
    const source = await readFile(join(root, 'scripts', name), 'utf8');
    assert.match(source, /log: join\(output, 'playwright.log'\)[\s\S]*?containDescendants: true/);
    for (const boundary of ['owned-command.mjs', 'windows-owned-job.ps1', 'windows-owned-job.cs']) {
      assert.ok(source.includes(`'scripts/${boundary}'`));
    }
    assert.ok(source.includes('`--global-timeout=${options.timeout - 500}`'));
  }
});
async function fixture(fn) {
  await mkdir(join(root, 'tmp'), { recursive: true });
  const directory = await mkdtemp(join(root, 'tmp', 'owned-job-'));
  try { await fn(directory); }
  finally { await rm(directory, { recursive: true }); }
}
if (process.platform === 'win32') {
  test('Windows Job retains stdout, stderr, exit code, cwd and exact argument values', () => fixture(async directory => {
    const values = ['', 'white space', '"quoted"', 'trailing\\', 'back\\\"quote', '字😀', '$not; a shell | command'];
    const code = 'console.log(JSON.stringify({args:process.argv.slice(1),cwd:process.cwd(),plan:process.env.XTERM_OWNED_JOB_COMMAND}));console.error("stderr retained");process.exit(7)';
    const result = await runCommand(process.execPath, ['-e', code, ...values], {
      cwd: directory, env: process.env, log: join(directory, 'command.log'), timeoutMs: 20_000, containDescendants: true
    });
    assert.equal(result.status, 'failed'); assert.equal(result.exitCode, 7);
    const log = await readFile(join(directory, 'command.log'), 'utf8');
    const observed = JSON.parse(log.split(/\r?\n/).find(line => line.startsWith('{')));
    assert.deepEqual(observed.args, values); assert.equal(observed.cwd.toLowerCase(), directory.toLowerCase());
    assert.equal(observed.plan, undefined); assert.match(log, /stderr retained/);
  }));
}
