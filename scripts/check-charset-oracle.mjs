// Checks deterministic fixture generation and real Rust assertion sensitivity.
// All source mutations are isolated under tmp/, never applied to the workspace.
import { readFile, writeFile, mkdir, copyFile, rm } from 'node:fs/promises';
import { createHash } from 'node:crypto';
import { spawnSync } from 'node:child_process';
import { fileURLToPath } from 'node:url';
import path from 'node:path';
import { strict as assert } from 'node:assert';

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..');
const temp = path.join(root, 'tmp/charset-negative-controls');
const results = path.join(root, 'results/charset');
await mkdir(path.join(temp, 'tests/fixtures'), { recursive: true });
await mkdir(results, { recursive: true });
const digest = bytes => createHash('sha256').update(bytes).digest('hex');
const fixtureNames = ['charset-maps.tsv', 'charset-service.tsv'];
const hashes = async () => Promise.all(fixtureNames.map(async file =>
  digest(await readFile(path.join(root, 'crates/xterm-core/tests/fixtures', file)))));
const report = {
  node: process.version, platform: process.platform, arch: process.arch,
  purpose: 'Deterministic regeneration and mutation rejection, not terminal equivalence.',
  commands: [], controls: []
};
function command(program, args, expectFailure = false) {
  const run = spawnSync(program, args, { cwd: root, encoding: 'utf8', timeout: 60000, maxBuffer: 2 ** 20 });
  report.commands.push({ program, args, status: run.status, signal: run.signal,
    stdout: run.stdout, stderr: run.stderr, error: run.error?.message });
  if (run.error) throw run.error;
  assert.equal(run.signal, null, 'Command terminated by a signal');
  if (expectFailure) {
    assert.notEqual(run.status, 0, 'Mutated Rust implementation unexpectedly passed');
    assert.match(run.stdout, /test result: FAILED\./, 'Failure must come from test assertions');
  } else assert.equal(run.status, 0, `${program} failed: ${run.stderr}`);
  return run;
}
function replaceExactlyOnce(text, oldValue, newValue) {
  assert.equal(text.split(oldValue).length, 2, `Mutation must have one target: ${oldValue}`);
  return text.replace(oldValue, newValue);
}
const service = await readFile(path.join(root, 'crates/xterm-core/src/charset.rs'), 'utf8');
const tables = await readFile(path.join(root, 'crates/xterm-core/src/charset_tables.rs'), 'utf8');
const library = path.join(temp, 'libxterm_core.rlib');
const executable = path.join(temp, process.platform === 'win32' ? 'charset-tests.exe' : 'charset-tests');
try {
  command(process.execPath, ['scripts/charset-oracle.mjs']);
  const first = await hashes();
  command(process.execPath, ['scripts/charset-oracle.mjs']);
  const second = await hashes();
  assert.deepEqual(second, first, 'Regenerated fixtures differ');
  report.fixtureSha256 = Object.fromEntries(fixtureNames.map((name, i) => [name, first[i]]));
  report.deterministic = true;
  await writeFile(path.join(temp, 'lib.rs'), 'pub mod charset;\n');
  await copyFile(path.join(root, 'crates/xterm-core/tests/charset.rs'), path.join(temp, 'tests/upstream.rs'));
  await copyFile(path.join(root, 'crates/xterm-core/tests/charset_edges.rs'), path.join(temp, 'tests/edges.rs'));
  await writeFile(path.join(temp, 'tests/charset.rs'), '#[path = "upstream.rs"] mod upstream;\n#[path = "edges.rs"] mod edges;\n');
  for (const file of fixtureNames) {
    await copyFile(path.join(root, 'crates/xterm-core/tests/fixtures', file), path.join(temp, 'tests/fixtures', file));
  }
  const rustc = process.env.RUSTC || 'rustc';
  report.rustc = command(rustc, ['--version']).stdout.trim();
  const cases = [
    { name: 'unmodified baseline' },
    { name: 'wrong map entry', tables: ["('a', \"▒\")", "('a', \"A\")"] },
    { name: 'lost alias identity', tables: ['Arc::clone(&map)', '{ let original: Charset = Arc::clone(&map); Arc::new((*original).clone()) }'] },
    { name: 'wrong active level', service: ['self.glevel = g;', 'self.glevel = g.wrapping_add(1);'] },
    { name: 'wrong slot length', service: ['self.charsets.resize_with(len, || None);', 'self.charsets.resize_with(len + 1, || None);'] },
    { name: 'wrong slot value', service: ['self.charsets[g] = charset.clone();', 'self.charsets[g] = None;'] }
  ];
  for (const mutation of cases) {
    await writeFile(path.join(temp, 'charset.rs'), mutation.service
      ? replaceExactlyOnce(service, ...mutation.service) : service);
    await writeFile(path.join(temp, 'charset_tables.rs'), mutation.tables
      ? replaceExactlyOnce(tables, ...mutation.tables) : tables);
    // One tiny std-only test library is rebuilt in place. Debug symbols and
    // incremental caches are not generated, and artifacts are removed below.
    command(rustc, ['--crate-name', 'xterm_core', '--crate-type', 'lib', '--edition=2021',
      path.join(temp, 'lib.rs'), '-o', library]);
    command(rustc, ['--test', '--edition=2021', path.join(temp, 'tests/charset.rs'),
      '--extern', `xterm_core=${library}`, '-o', executable]);
    const rejected = mutation.name !== 'unmodified baseline';
    const run = command(executable, [], rejected);
    if (!rejected) assert.match(run.stdout, /16 passed; 0 failed;/);
    report.controls.push({ name: mutation.name, expectedFailure: rejected, status: run.status,
      result: run.stdout.match(/test result: .*$/m)?.[0] });
  }
  report.passed = true;
} catch (error) {
  report.passed = false;
  report.failure = String(error.stack);
  process.exitCode = 1;
} finally {
  // Preserve source copies for diagnosing a failed negative control, but free
  // executables immediately to respect the resumed swarm's disk budget.
  await Promise.all([library, executable].map(file => rm(file, { force: true })));
  await writeFile(path.join(results, 'oracle-controls.json'), JSON.stringify(report, null, 2) + '\n');
}
console.log(JSON.stringify({ deterministic: report.deterministic, controls: report.controls, passed: report.passed }));
