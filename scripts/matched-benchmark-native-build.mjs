// Run only with an explicit compiler allocation. No shared targets are borrowed.
import assert from 'node:assert/strict';
import fs from 'node:fs';
import path from 'node:path';
import os from 'node:os';
import { fileURLToPath } from 'node:url';
import { spawnSync } from 'node:child_process';
import { runCommand } from './quality-process.mjs';
import { jsonHash } from './benchmark-contract.mjs';
import { sha256, verify, runtimePins } from './matched-benchmark-adapter.mjs';
const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..');
const result = path.join(root, 'results/matched-performance329');
const temporary = path.join(root, 'tmp');
const target = path.join(root, 'target');
export function sources() {
  const names = ['Cargo.toml', 'Cargo.lock', '.cargo/config.toml',
    'crates/xterm-addons/examples/matched_benchmark.rs',
    'crates/xterm-core/tests/support/terminal_snapshot.rs', 'crates/xterm-core/tests/support/full_trace_snapshot.rs'];
  function walk(relative) {
    for (const file of fs.readdirSync(path.join(root, relative), { withFileTypes: true })) {
      assert.ok(!file.isSymbolicLink(), `Source link: ${relative}/${file.name}`);
      const name = `${relative}/${file.name}`;
      if (file.isDirectory()) walk(name); else if (file.isFile()) names.push(name);
    }
  }
  for (const name of ['xterm-core', 'xterm-addons']) {
    names.push(`crates/${name}/Cargo.toml`); walk(`crates/${name}/src`);
  }
  for (const name of ['xterm-web', 'xterm-font-addons', 'xterm-wasm-support']) names.push(`crates/${name}/Cargo.toml`);
  return names.sort().map(name => ({ name, path: path.join(root, name), sha256: sha256(path.join(root, name)) }));
}
function resources() {
  const memory = os.freemem(), disk = fs.statfsSync(temporary);
  const free = disk.bavail * disk.bsize;
  assert.ok(memory >= 8 * 1024 ** 3, 'Less than 8 GiB available RAM');
  assert.ok(free >= 10 * 1024 ** 3, 'Less than 10 GiB temporary disk');
  return { availableMemory: memory, temporaryDiskFree: free, at: new Date().toISOString() };
}
function version(program) {
  const command = spawnSync(program, ['--version', '--verbose'], { encoding: 'utf8', timeout: 30000, windowsHide: true });
  assert.equal(command.status, 0, command.stderr); return command.stdout.trim();
}
if (process.argv[1] && path.resolve(process.argv[1]) === fileURLToPath(import.meta.url)) {
  assert.equal(process.argv.length, 2, 'No arguments expected');
  fs.mkdirSync(temporary, { recursive: true }); fs.mkdirSync(result, { recursive: true });
  assert.ok(!fs.lstatSync(temporary).isSymbolicLink() && !fs.lstatSync(target).isSymbolicLink(), 'Private physical paths required');
  for (const key of ['RUSTFLAGS', 'CARGO_ENCODED_RUSTFLAGS', 'CARGO_PROFILE_RELEASE_OPT_LEVEL', 'CARGO_PROFILE_RELEASE_LTO',
    'CARGO_PROFILE_RELEASE_DEBUG', 'CARGO_PROFILE_RELEASE_STRIP', 'CARGO_PROFILE_RELEASE_PANIC', 'CARGO_BUILD_TARGET'])
    assert.ok(!process.env[key], `Undeclared build override: ${key}`);
  const pins = sources(), runtime = runtimePins(root);
  fs.writeFileSync(path.join(result, 'rust-provenance.json'), JSON.stringify({ status: 'invalidated until release build succeeds' }));
  const before = resources();
  const env = { ...process.env, CARGO_TARGET_DIR: target, CARGO_BUILD_JOBS: '2', TEMP: temporary, TMP: temporary, TMPDIR: temporary };
  const cargo = process.env.CARGO ?? 'cargo';
  const args = ['build', '--release', '--locked', '-p', 'xterm-addons', '--example', 'matched_benchmark', '-j', '2'];
  const outcome = await runCommand(cargo, args, { cwd: root, env, log: path.join(result, 'native-release-build.log'), timeoutMs: 600000,
    resourceCheck: resources });
  let after;
  try { after = resources(); } catch (error) { after = { error: String(error) }; }
  fs.writeFileSync(path.join(result, 'native-release-build.json'), JSON.stringify({ before, after, outcome, sources: pins }, null, 2));
  assert.equal(outcome.status, 'passed', 'Native release build failed');
  const artifact = path.join(target, 'release/examples', `matched_benchmark${process.platform === 'win32' ? '.exe' : ''}`);
  const artifacts = { native: { path: artifact, sha256: sha256(artifact) } };
  const build = { commands: [outcome.command.join(' ')], sourceSha256: jsonHash(pins.map(({ name, sha256 }) => ({ name, sha256 }))),
    artifactSha256: jsonHash(Object.entries(artifacts).map(([name, pin]) => ({ name, sha256: pin.sha256 }))),
    toolchain: `${version(cargo)}; ${version(process.env.RUSTC ?? 'rustc')}`,
    flags: 'Cargo default release opt-level=3, debug=false, debug-assertions=false, overflow-checks=false, lto=false, strip=none, panic=unwind; default features; --locked; -j2; no RUSTFLAGS; private target' };
  build.flags += `; adapter-runtime-sha256=${jsonHash(runtime.map(({ name, sha256 }) => ({ name, sha256 })))}`;
  const provenance = { sources: pins, artifacts, runtime, build };
  verify(provenance);
  fs.writeFileSync(path.join(result, 'rust-provenance.json'), JSON.stringify(provenance, null, 2));
  console.log(JSON.stringify({ artifact: artifacts.native, sourceFiles: pins.length, outcome: outcome.status }));
}
