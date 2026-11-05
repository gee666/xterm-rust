import test from 'node:test';
import assert from 'node:assert/strict';
import { spawnSync } from 'node:child_process';
import { mkdir, mkdtemp, readdir, rm } from 'node:fs/promises';
import { join } from 'node:path';
import { fileURLToPath } from 'node:url';

const root = fileURLToPath(new URL('../', import.meta.url));
test('retired queue refuses commands without starting work or creating lease data', async () => {
  await mkdir(join(root, 'tmp'), { recursive: true });
  const directory = await mkdtemp(join(root, 'tmp', 'queue-retired-'));
  try {
    const marker = join(directory, 'must-not-exist');
    const result = spawnSync(process.execPath, [join(root, 'scripts/build-slot.mjs'),
      '--owner', 'retired-test', '--lane', 'native', '--', process.execPath, '-e',
      `require('node:fs').writeFileSync(${JSON.stringify(marker)}, 'launched')`],
    { cwd: directory, encoding: 'utf8', timeout: 5000 });
    assert.equal(result.error, undefined);
    assert.equal(result.status, 1);
    assert.match(result.stderr, /global build queue is retired/);
    assert.match(result.stderr, /worktree-create\.mjs/);
    assert.deepEqual(await readdir(directory), []);
  } finally {
    await rm(directory, { recursive: true, force: true });
  }
});
