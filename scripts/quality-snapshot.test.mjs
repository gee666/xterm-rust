import { test } from 'node:test';
import assert from 'node:assert/strict';
import { mkdir, mkdtemp, writeFile, rm } from 'node:fs/promises';
import { join } from 'node:path';
import { fileURLToPath } from 'node:url';
import { snapshot } from './quality-snapshot.mjs';
import { diskHeadroom, requireHeadroom } from './quality-resources.mjs';
const root = fileURLToPath(new URL('../', import.meta.url));
async function fixture(fn) {
  await mkdir(join(root, 'tmp'), { recursive: true });
  const directory = await mkdtemp(join(root, 'tmp', 'quality-snapshot-'));
  try {
    for (const path of ['crates/core', 'scripts', '.cargo', '.github', 'crates/core/target']) await mkdir(join(directory, path), { recursive: true });
    for (const path of ['Cargo.toml', 'Cargo.lock', 'crates/core/lib.rs', 'crates/core/fixture.tsv', 'scripts/check.mjs', '.cargo/config.toml', '.github/workflow.yml']) await writeFile(join(directory, path), 'initial\n');
    await fn(directory);
  } finally { await rm(directory, { recursive: true, force: true }); }
}
test('snapshot changes for Rust, fixture, JS, workflow and profile edits', () => fixture(async directory => {
  let previous = await snapshot(directory);
  assert.equal(previous.files, 7);
  for (const path of ['crates/core/lib.rs', 'crates/core/fixture.tsv', 'scripts/check.mjs', '.cargo/config.toml', '.github/workflow.yml']) {
    await writeFile(join(directory, path), 'mutated\n');
    const current = await snapshot(directory);
    assert.notEqual(current.sha256, previous.sha256, path);
    previous = current;
  }
  await writeFile(join(directory, 'crates/core/target/ignored.rs'), 'generated');
  assert.equal((await snapshot(directory)).sha256, previous.sha256);
}));
test('unsafe scan retains exact file and line but does not claim semantic safety', () => fixture(async directory => {
  await writeFile(join(directory, 'crates/core/lib.rs'), '// unsafe review candidate\nfn safe() {}\n');
  const current = await snapshot(directory);
  assert.deepEqual(current.unsafeCandidates, [{ path: 'crates/core/lib.rs', line: 1, text: '// unsafe review candidate' }]);
}));
test('disk preflight records byte counts and rejects insufficient headroom without cleanup', async () => {
  const observed = await diskHeadroom(root, 1024);
  assert.match(observed.availableBytes, /^\d+$/);
  assert.equal(observed.minimumBytes, '1073741824');
  assert.doesNotThrow(() => requireHeadroom({ sufficient: true }));
  assert.throws(() => requireHeadroom({ sufficient: false, availableBytes: '10', minimumBytes: '100' }), /No cleanup was attempted/);
});
