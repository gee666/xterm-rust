// Task498 scoped source-restoring negative control. Run from an owned O: worktree.
import assert from 'node:assert/strict';
import { cp, mkdir, readFile, readdir, rm, statfs, writeFile } from 'node:fs/promises';
import { execFileSync } from 'node:child_process';
import { createHash } from 'node:crypto';
import { freemem } from 'node:os';
import { join, resolve } from 'node:path';
import { runCommand } from '../../scripts/quality-process.mjs';
import { artifactIdentity, sourceIdentity, verifyBuiltPackage } from '../../scripts/e2e/build-identity.mjs';
import { referenceIntegrity } from '../../scripts/upstream-baseline-utils.mjs';
const root = process.cwd(), output = join(root, 'results/serialize-scrollback-array');
const scratch = join(root, 'tmp/scrollback-array'), mode = process.argv[2];
assert(['proof', 'quality', 'audit'].includes(mode), 'Usage: node results/serialize-scrollback-array/reproduce.mjs proof|quality|audit');
await mkdir(scratch, { recursive: true });
await mkdir(output, { recursive: true });
const sourcePath = 'crates/xterm-web/src/serialize_addon/mod.rs';
const digest = bytes => createHash('sha256').update(bytes).digest('hex');
const json = async file => JSON.parse(await readFile(file, 'utf8'));
const save = (file, data) => writeFile(file, JSON.stringify(data, null, 2) + '\n');
const git = args => execFileSync('git', args, { cwd: root, timeout: 30000, encoding: 'utf8' }).trim();
async function optional(file) { try { return await readFile(file); } catch (error) { if (error.code !== 'ENOENT') throw error; return null; } }
async function headroom() {
  const disk = await statfs(root), free = Number(disk.bavail) * Number(disk.bsize);
  assert(freemem() >= 8 * 1024 ** 3 && free >= 10 * 1024 ** 3, 'Insufficient RAM/O: headroom');
}
async function command(name, executable, args, directory = output) {
  await mkdir(directory, { recursive: true });
  await headroom();
  return runCommand(executable, args, { cwd: root, env: process.env,
    log: join(directory, `${name}.log`), timeoutMs: 300000, resourceCheck: headroom });
}
async function snapshotEntry() {
  const backup = join(scratch, 'entry-package');
  await rm(backup, { recursive: true, force: true });
  let packagePresent = false;
  try { await cp(join(root, 'browser/pkg'), backup, { recursive: true, errorOnExist: true }); packagePresent = true; }
  catch (error) { if (error.code !== 'ENOENT') throw error; }
  return { backup, packagePresent, pointer: await optional(join(root, 'results/demos/build/latest.json')),
    source: await readFile(join(root, sourcePath)) };
}
async function restoreEntry(entry) {
  // Remove only this worktree's generated package, never reference/dependency trees.
  await rm(join(root, 'browser/pkg'), { recursive: true, force: true });
  if (entry.packagePresent) await cp(entry.backup, join(root, 'browser/pkg'), { recursive: true });
  const pointer = join(root, 'results/demos/build/latest.json');
  if (entry.pointer === null) await rm(pointer, { force: true }); else await writeFile(pointer, entry.pointer);
  assert.deepEqual(await optional(pointer), entry.pointer);
  if (entry.packagePresent) {
    assert.deepEqual(await artifactIdentity(root, ['browser/pkg/xterm_web.js', 'browser/pkg/xterm_web_bg.wasm']),
      await artifactIdentity(scratch, ['entry-package/xterm_web.js', 'entry-package/xterm_web_bg.wasm'])
        .then(files => files.map((file, index) => ({ ...file, path: ['browser/pkg/xterm_web.js', 'browser/pkg/xterm_web_bg.wasm'][index] }))));
  } else assert.equal(await optional(join(root, 'browser/pkg/xterm_web_bg.wasm')), null);
}
async function variant(name, bytes, retainPackage) {
  const directory = join(output, name), entry = await snapshotEntry();
  await mkdir(directory, { recursive: true });
  const report = { name, status: 'failed', sourceBytes: bytes.length, sourceHash: digest(bytes), commands: [] };
  let complete = false;
  try {
    // writeFile is deliberate. Windows copyFile can retain an old mtime and let
    // Cargo reuse stale WASM despite a changed sourceIdentity hash.
    await writeFile(join(root, sourcePath), bytes);
    report.input = await sourceIdentity(root);
    const build = await command('build', process.execPath, ['scripts/build-demo.mjs', '--debug'], directory);
    report.commands.push(build); assert.equal(build.status, 'passed');
    report.build = await verifyBuiltPackage(root, report.input);
    const pointer = await json(join(root, 'results/demos/build/latest.json'));
    await cp(join(root, pointer.report, '..'), join(directory, 'build'), { recursive: true });
    const buildReport = await json(join(directory, 'build/report.json'));
    const buildLog = buildReport.commands.find(command => command.log.endsWith('debug-wasm.log'));
    const cargo = (await readFile(join(root, buildLog.log), 'utf8')).split(/\r?\n/)
      .filter(line => line.startsWith('{')).map(line => JSON.parse(line));
    const web = cargo.filter(message => message.reason === 'compiler-artifact' && message.target.name === 'xterm_web');
    assert(web.length > 0 && web.every(message => message.fresh === false), 'Actual xterm_web must rebuild, not reuse a stale artifact');
    report.webFresh = web.map(message => message.fresh);
    const browser = await command('browser', process.execPath, ['scripts/serialize-browser-check.mjs', '--suite'], directory);
    report.commands.push(browser);
    const browserFiles = ['verification.json', ...['chromium', 'firefox', 'webkit'].flatMap(name =>
      [`${name}-source.json`, `${name}-rust.json`, `${name}-terminal.png`]),
    ...['source-suite', 'rust-suite'].flatMap(name => ['configuration.cjs.txt', 'playwright.json', 'playwright.log'].map(file => `${name}/${file}`))];
    for (const file of browserFiles) {
      await mkdir(join(directory, 'browser', file, '..'), { recursive: true });
      await cp(join(root, 'results/serialize-browser', file), join(directory, 'browser', file));
    }
    const observed = await json(join(directory, 'browser/verification.json'));
    assert.equal(observed.error, undefined); assert.deepEqual(observed.cleanupErrors, []);
    assert.equal(observed.referenceBefore.mismatches.length, 0); assert.equal(observed.referenceAfter.mismatches.length, 0);
    assert.equal(observed.browsers.length, 3);
    for (const engine of observed.browsers) {
      assert.equal(engine.error, undefined); assert.equal(engine.discovered, 77); assert.equal(engine.executed, 77);
      if (name === 'baseline') {
        const names = engine.mismatches.map(item => item.name);
        assert(names.includes('ansi-scrollback-primitive-array-errors'));
        assert(names.includes('ansi-scrollback-getter-reentry-errors'));
        assert(names.every(name => name.startsWith('ansi-scrollback-')));
      } else assert.equal(engine.failed, 0);
    }
    for (const suite of observed.suites) {
      assert.equal(suite.result.status, 'passed');
      assert.equal(suite.counts.passed, 120);
      for (const field of ['failed', 'skipped', 'unrun']) assert.equal(suite.counts[field], 0);
    }
    assert.equal(observed.suites.length, 2);
    assert.equal(browser.exitCode, name === 'baseline' ? 1 : 0);
    report.browser = { status: observed.status, engines: observed.browsers.map(({ name, passed, failed, mismatches }) => ({ name, passed, failed, mismatches })), suites: observed.suites.map(({ engine, counts }) => ({ engine, counts })) };
    report.status = 'passed'; complete = true;
  } catch (error) { report.error = String(error.stack ?? error); throw error; }
  finally {
    await writeFile(join(root, sourcePath), entry.source);
    assert.equal(digest(await readFile(join(root, sourcePath))), digest(entry.source));
    if (!complete || !retainPackage) await restoreEntry(entry);
    else await verifyBuiltPackage(root, await sourceIdentity(root));
    report.restoredSourceHash = digest(await readFile(join(root, sourcePath)));
    report.entryRestored = !complete || !retainPackage;
    await save(join(directory, 'variant.json'), report);
    await rm(entry.backup, { recursive: true, force: true });
  }
  return report;
}
if (mode === 'proof') {
  const fixed = await readFile(join(root, sourcePath));
  const baseline = Buffer.from(execFileSync('git', ['show', `fbea6472:${sourcePath}`], { cwd: root, timeout: 30000 }));
  const before = await variant('baseline', baseline, false);
  const after = await variant('fixed', fixed, true);
  assert.notEqual(before.build.artifacts[1].sha256, after.build.artifacts[1].sha256);
  await save(join(output, 'summary.json'), { status: 'passed', baseline: before.browser, fixed: after.browser,
    baselineInput: before.input, fixedInput: after.input, baselineWasm: before.build.artifacts[1], fixedWasm: after.build.artifacts[1],
    scope: 'Primitive numeric ANSI scrollback only; full raw346/322 and overall equivalence NOT PROVEN.' });
}
if (mode === 'quality') {
  const tests = (await readdir(join(root, 'crates/xterm-core/tests'))).filter(name => /^serialize.*\.rs$/.test(name)).map(name => name.slice(0, -3)).sort();
  const selection = tests.flatMap(name => ['--test', name]);
  const commands = [];
  for (const [name, args, mustPass] of [
    ['native', ['test', '--locked', '-j2', '-p', 'xterm-core', ...selection], true],
    ['clippy-native', ['clippy', '--locked', '-j2', '-p', 'xterm-core', '--lib', ...selection, '--', '-D', 'warnings'], true],
    ['clippy-wasm', ['clippy', '--locked', '-j2', '-p', 'xterm-web', '--lib', '--target', 'wasm32-unknown-unknown', '--', '-D', 'warnings'], true],
    ['fmt-workspace', ['fmt', '--all', '--', '--check'], false],
    ...['xterm-core', 'xterm-addons', 'xterm-web', 'xterm-wasm-support', 'xterm-font-addons'].map(name => [`fmt-${name}`, ['fmt', '-p', name, '--', '--check'], true])
  ]) {
    const result = await command(name, process.env.CARGO ?? 'cargo', args);
    commands.push({ name, ...result }); await save(join(output, 'quality.json'), { commands, tests });
    if (mustPass) assert.equal(result.status, 'passed', name);
  }
  const binaries = [];
  for (const file of await readdir(join(root, 'target/debug/deps'))) if (/^serialize.*\.exe$/.test(file)) binaries.push(`target/debug/deps/${file}`);
  await save(join(output, 'native-binaries.json'), await artifactIdentity(root, binaries));
}
if (mode === 'audit') {
  const manifest = await json(join(root, 'docs/baseline/reference-manifest.json'));
  const reference = referenceIntegrity(resolve(process.env.SOURCE_ROOT ?? join(root, 'source_repo')), manifest);
  const copy = referenceIntegrity(join(root, 'source_repo'), manifest);
  assert.equal(reference.mismatches.length, 0); assert.equal(copy.mismatches.length, 0);
  const summary = await json(join(output, 'summary.json'));
  const fixedSource = await readFile(join(root, sourcePath));
  const expectedBaseline = execFileSync('git', ['show', `fbea6472:${sourcePath}`], { cwd: root, timeout: 30000 });
  try {
    await writeFile(join(root, sourcePath), expectedBaseline);
    assert.equal((await sourceIdentity(root)).sha256, summary.baselineInput.sha256);
  } finally { await writeFile(join(root, sourcePath), fixedSource); }
  assert.equal((await sourceIdentity(root)).sha256, summary.fixedInput.sha256);
  const traces = [];
  for (const name of ['chromium', 'firefox', 'webkit']) {
    const load = (phase, engine) => json(join(output, phase, 'browser', `${name}-${engine}.json`));
    const [beforeSource, afterSource, beforeRust, afterRust] = await Promise.all([
      load('baseline', 'source'), load('fixed', 'source'), load('baseline', 'rust'), load('fixed', 'rust')]);
    assert.deepEqual(beforeSource, afterSource);
    assert.deepEqual(afterRust, afterSource);
    const inherited = entries => entries.filter(entry => !entry.name.startsWith('ansi-scrollback-'));
    assert.equal(inherited(beforeRust).length, 74);
    assert.deepEqual(inherited(beforeRust), inherited(afterRust));
    const before = beforeRust.find(entry => entry.name === 'ansi-scrollback-primitive-array-errors').result.values;
    const after = afterRust.find(entry => entry.name === 'ansi-scrollback-primitive-array-errors').result.values;
    traces.push({ name, sourceStable: true, fixedExact: 77, inheritedUnchanged: 74,
      repairedMatrixInputs: before.flatMap((value, index) => JSON.stringify(value) === JSON.stringify(after[index]) ? [] : [{ scrollback: value.scrollback, before: value.ansi, after: after[index].ansi }]) });
  }
  await save(join(output, 'trace-audit.json'), traces);
  const sources = [sourcePath, 'crates/xterm-web/src/serialize_addon/options.rs', 'crates/xterm-web/src/serialize_addon/html.rs',
    'scripts/serialize-browser-check.mjs', 'scripts/serialize-browser-cases.mjs', 'scripts/e2e/build-identity.mjs'];
  const artifacts = [];
  async function walk(relative) {
    for (const entry of await readdir(join(root, relative), { withFileTypes: true })) {
      const child = `${relative}/${entry.name}`;
      assert(!entry.isSymbolicLink());
      if (entry.isDirectory()) await walk(child);
      else if (entry.isFile() && entry.name !== 'pins.json') artifacts.push(child);
    }
  }
  await walk('results/serialize-scrollback-array');
  await save(join(output, 'pins.json'), { revision: git(['rev-parse', 'HEAD']), reference, copy,
    input: await sourceIdentity(root), package: await verifyBuiltPackage(root, await sourceIdentity(root)),
    sources: await artifactIdentity(root, sources), artifacts: await artifactIdentity(root, artifacts.sort()) });
}
console.log(`serialize-scrollback-array ${mode} passed`);
