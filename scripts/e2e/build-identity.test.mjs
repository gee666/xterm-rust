import test from 'node:test';
import assert from 'node:assert/strict';
import { mkdir, mkdtemp, writeFile, rm } from 'node:fs/promises';
import { execFile } from 'node:child_process';
import { promisify } from 'node:util';
import { dirname, join, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';
import { artifactIdentity, sourceIdentity, verifyBuiltPackage } from './build-identity.mjs';

const execute = promisify(execFile);
const root = resolve(dirname(fileURLToPath(import.meta.url)), '../..');
test('no-build refuses stale source, modified package and absent provenance', async () => {
  await mkdir(join(root, 'tmp'), { recursive: true });
  const directory = await mkdtemp(join(root, 'tmp/e2e-identity-'));
  async function put(path, content) {
    await mkdir(dirname(join(directory, path)), { recursive: true });
    await writeFile(join(directory, path), content);
  }
  try {
    await execute('git', ['init', '--quiet'], { cwd: directory, timeout: 10000 });
    await execute('git', ['-c', 'user.name=Fixture', '-c', 'user.email=fixture@example.invalid',
      'commit', '--quiet', '--allow-empty', '-m', 'Identity fixture'], { cwd: directory, timeout: 10000 });
    for (const path of ['Cargo.toml', 'Cargo.lock', '.cargo/config.toml', 'crates/example/src/lib.rs']) await put(path, 'input');
    // Identity-test bytes only, not executable terminal behavior evidence.
    const outputs = ['browser/pkg/xterm_web.js', 'browser/pkg/xterm_web_bg.wasm'];
    for (const path of outputs) await put(path, 'build-output');
    const source = await sourceIdentity(directory);
    const report = 'results/demos/build/test-run/report.json';
    await put('results/demos/build/latest.json', JSON.stringify({ status: 'passed', report }));
    await put(report, JSON.stringify({ status: 'passed', artifacts: await artifactIdentity(directory, outputs) }));
    await assert.rejects(verifyBuiltPackage(directory, source), /no matching source identity/);
    await put(report, JSON.stringify({ status: 'passed', source, artifacts: await artifactIdentity(directory, outputs) }));
    assert.equal((await verifyBuiltPackage(directory, source)).artifacts.length, 2);
    await put(outputs[1], 'tampered');
    await assert.rejects(verifyBuiltPackage(directory, source), /artifact changed/);
    await put(outputs[1], 'build-output');
    await put('crates/example/src/lib.rs', 'uncommitted change');
    const changed = await sourceIdentity(directory);
    assert.equal(changed.revision, source.revision);
    assert.notEqual(changed.sha256, source.sha256);
    await assert.rejects(verifyBuiltPackage(directory, changed), /stale/);
    await put('results/demos/build/latest.json', JSON.stringify({ status: 'passed', report: '../outside.json' }));
    await assert.rejects(verifyBuiltPackage(directory, changed), /Invalid Rust build report path/);
  } finally { await rm(directory, { recursive: true, force: true }); }
});
