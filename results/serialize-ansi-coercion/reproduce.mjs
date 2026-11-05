// Reuse the source/Rust serializer runner with exact baseline/fixed packages.
import assert from 'node:assert/strict';
import { cp, mkdir, readFile, readdir, rm, statfs, writeFile } from 'node:fs/promises';
import { execFileSync } from 'node:child_process';
import { freemem } from 'node:os';
import { join } from 'node:path';
import { runCommand } from '../../scripts/quality-process.mjs';
import { artifactIdentity, sourceIdentity, verifyBuiltPackage } from '../../scripts/e2e/build-identity.mjs';
import { referenceIntegrity } from '../../scripts/upstream-baseline-utils.mjs';
const root = process.cwd(), output = join(root, 'results/serialize-ansi-coercion');
const scratch = join(root, 'tmp/ansi-coercion'), mode = process.argv[2];
assert(['build', 'proof', 'quality', 'packages', 'audit'].includes(mode));
const selectedPhase = process.argv[3] ?? 'both';
assert(['baseline', 'fixed', 'both'].includes(selectedPhase));
const prefix = 'ansi-scrollback-coercion-', total = 90, inheritedTotal = 85;
const missingLine = 'ansi-scrollback-captured-missing-line';
const failingCases = ['values', 'default-hint', 'reentry'].map(name => prefix + name);
const expectedFailures = (phase, engine) => phase === 'baseline' ? [
  ...failingCases, ...(engine === 'chromium' ? [] : [missingLine])
] : [];
const sources = ['crates/xterm-web/src/serialize_addon/mod.rs', 'crates/xterm-web/src/serialize_addon/options.rs',
  'crates/xterm-web/src/serialize_addon/view.rs'];
const pointer = join(root, 'results/demos/build/latest.json'), pkg = join(root, 'browser/pkg');
const json = async file => JSON.parse(await readFile(file, 'utf8'));
const save = (file, data) => writeFile(file, JSON.stringify(data, null, 2) + '\n');
const git = args => execFileSync('git', args, { cwd: root, timeout: 30000 });
async function optional(file) { try { return await readFile(file); } catch (error) { if (error.code !== 'ENOENT') throw error; return null; } }
async function headroom() {
  const disk = await statfs(root), free = Number(disk.bavail) * Number(disk.bsize);
  assert(freemem() >= 8 * 1024 ** 3 && free >= 10 * 1024 ** 3, 'Insufficient RAM/O: headroom');
}
async function command(name, executable, args, directory = output) {
  await mkdir(directory, { recursive: true }); await headroom();
  return runCommand(executable, args, { cwd: root, env: process.env,
    log: join(directory, `${name}.log`), timeoutMs: 300000, resourceCheck: headroom });
}
async function writeSources(bytes) {
  for (let i = 0; i < sources.length; i++) await writeFile(join(root, sources[i]), bytes[i]);
}
async function copyPackage(from, to) {
  await rm(to, { recursive: true, force: true });
  await cp(from, to, { recursive: true, errorOnExist: true });
}
async function packageFiles(directory) {
  const files = [];
  async function walk(relative = '') {
    for (const entry of await readdir(join(directory, relative), { withFileTypes: true })) {
      assert(!entry.isSymbolicLink()); const path = relative ? `${relative}/${entry.name}` : entry.name;
      if (entry.isDirectory()) await walk(path); else { assert(entry.isFile()); files.push(path); }
    }
  }
  await walk(); return artifactIdentity(directory, files.sort());
}
function validate(report, phase) {
  assert.equal(report.error, undefined); assert.deepEqual(report.cleanupErrors, []);
  assert.equal(report.browsers.length, 3); assert.equal(report.suites.length, 2);
  for (const engine of report.browsers) {
    assert.equal(engine.error, undefined); assert.equal(engine.discovered, total); assert.equal(engine.executed, total);
    assert.deepEqual(engine.mismatches.map(entry => entry.name), expectedFailures(phase, engine.name));
    assert.equal(engine.failed, expectedFailures(phase, engine.name).length);
  }
  for (const suite of report.suites) {
    assert.equal(suite.result.status, 'passed'); assert.equal(suite.counts.passed, 120);
    for (const field of ['failed', 'skipped', 'unrun']) assert.equal(suite.counts[field], 0);
  }
  assert.equal(report.status, phase === 'baseline' ? 'failed' : 'passed');
}
await mkdir(scratch, { recursive: true });
const fixed = await Promise.all(sources.map(file => readFile(join(root, file))));
const baseline = sources.map(file => git(['show', `ec2b22ac:${file}`]));
if (mode === 'build' || mode === 'proof') {
  const entryPointer = await optional(pointer), entryBackup = join(scratch, 'entry-package');
  const entryPresent = (await optional(join(pkg, 'xterm_web_bg.wasm'))) !== null;
  if (entryPresent) await copyPackage(pkg, entryBackup);
  let complete = false;
  try {
    for (const [phase, bytes] of [['baseline', baseline], ['fixed', fixed]]) {
      if (selectedPhase !== 'both' && selectedPhase !== phase) continue;
      const directory = join(output, phase), retained = join(scratch, phase);
      await mkdir(directory, { recursive: true }); await mkdir(retained, { recursive: true });
      // Update mtimes: restoring old bytes must not reuse Cargo's newer artifact.
      await writeSources(bytes);
      const input = await sourceIdentity(root);
      if (mode === 'build') {
        const result = await command('build', process.execPath, ['scripts/build-demo.mjs', '--debug'], directory);
        assert.equal(result.status, 'passed');
        const build = await verifyBuiltPackage(root, input), latest = await json(pointer);
        await cp(join(root, latest.report, '..'), join(directory, 'build'), { recursive: true });
        const report = await json(join(directory, 'build/report.json'));
        const log = report.commands.find(entry => entry.log.endsWith('debug-wasm.log')).log;
        const messages = (await readFile(join(root, log), 'utf8')).split(/\r?\n/).filter(line => line.startsWith('{')).map(JSON.parse);
        const web = messages.filter(entry => entry.reason === 'compiler-artifact' && entry.target.name === 'xterm_web');
        assert(web.length && web.every(entry => entry.fresh === false), 'Actual web WASM must rebuild');
        await copyPackage(pkg, join(retained, 'pkg'));
        await writeFile(join(retained, 'latest.json'), await readFile(pointer));
        await save(join(directory, 'build-identity.json'), { input, build, webFresh: web.map(entry => entry.fresh) });
      } else {
        const built = await json(join(directory, 'build-identity.json'));
        assert.equal(input.sha256, built.input.sha256);
        await copyPackage(join(retained, 'pkg'), pkg);
        await writeFile(pointer, await readFile(join(retained, 'latest.json')));
        assert.deepEqual(await verifyBuiltPackage(root, input), built.build);
        assert.deepEqual(await packageFiles(pkg), await json(join(directory, 'package-files.json')));
        const result = await command('browser', process.execPath, ['scripts/serialize-browser-check.mjs', '--suite'], directory);
        const files = ['verification.json', ...['chromium', 'firefox', 'webkit'].flatMap(name =>
          [`${name}-source.json`, `${name}-rust.json`, `${name}-terminal.png`]),
        ...['source-suite', 'rust-suite'].flatMap(name => ['configuration.cjs.txt', 'playwright.json', 'playwright.log'].map(file => `${name}/${file}`))];
        for (const file of files) {
          await mkdir(join(directory, 'browser', file, '..'), { recursive: true });
          await cp(join(root, 'results/serialize-browser', file), join(directory, 'browser', file));
        }
        const report = await json(join(directory, 'browser/verification.json'));
        validate(report, phase);
        assert.equal(result.exitCode, phase === 'baseline' ? 1 : 0);
        await save(join(directory, 'proof.json'), { status: 'passed', result, engines: report.browsers, suites: report.suites });
      }
    }
    complete = true;
  } finally {
    await writeSources(fixed);
    for (let i = 0; i < sources.length; i++) assert.deepEqual(await readFile(join(root, sources[i])), fixed[i]);
    if (!complete || selectedPhase === 'baseline') {
      await rm(pkg, { recursive: true, force: true });
      if (entryPresent) await copyPackage(entryBackup, pkg);
      if (entryPointer === null) await rm(pointer, { force: true }); else await writeFile(pointer, entryPointer);
      assert.deepEqual(await optional(pointer), entryPointer);
    } else await verifyBuiltPackage(root, await sourceIdentity(root));
    await rm(entryBackup, { recursive: true, force: true });
  }
}
if (mode === 'packages') {
  for (const phase of ['baseline', 'fixed']) {
    const directory = join(scratch, phase, 'pkg'), files = await packageFiles(directory);
    const built = await json(join(output, phase, 'build-identity.json'));
    for (const artifact of built.build.artifacts) {
      assert.equal(files.find(file => `browser/pkg/${file.path}` === artifact.path)?.sha256, artifact.sha256);
    }
    const helpers = [];
    for (const file of files.filter(file => file.path.endsWith('.js'))) {
      const text = await readFile(join(directory, file.path), 'utf8');
      if (text.includes('export function serializeMissingWrappedLineError()')) helpers.push({ ...file, text });
    }
    assert.equal(helpers.length, phase === 'fixed' ? 1 : 0);
    if (phase === 'fixed') {
      assert.match(helpers[0].text, /const nextLine = undefined; return nextLine\.isWrapped;/);
      assert.deepEqual(await packageFiles(pkg), files);
    }
    await save(join(output, phase, 'package-files.json'), files);
    await save(join(output, phase, 'missing-line-helper.json'), helpers);
  }
}
if (mode === 'quality') {
  const tests = (await readdir(join(root, 'crates/xterm-core/tests'))).filter(file => /^serialize.*\.rs$/.test(file)).map(file => file.slice(0, -3)).sort();
  const selection = tests.flatMap(name => ['--test', name]), commands = [];
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
  const binaries = (await readdir(join(root, 'target/debug/deps'))).filter(file => /^serialize.*\.exe$/.test(file)).map(file => `target/debug/deps/${file}`);
  await save(join(output, 'native-binaries.json'), await artifactIdentity(root, binaries));
}
if (mode === 'audit') {
  const reports = [];
  for (const phase of ['baseline', 'fixed']) {
    const report = await json(join(output, phase, 'browser/verification.json')); validate(report, phase);
    reports.push({ phase, fullRunnerStatus: report.status, browsers: report.browsers.map(({ name, passed, failed, mismatches }) =>
      ({ name, passed, failed, mismatches: mismatches.map(entry => entry.name) })), suites: report.suites.map(({ engine, counts }) => ({ engine, counts })) });
  }
  const manifest = await json(join(root, 'docs/baseline/reference-manifest.json'));
  const reference = referenceIntegrity(process.env.SOURCE_ROOT, manifest), copy = referenceIntegrity(join(root, 'source_repo'), manifest);
  assert.equal(reference.mismatches.length, 0); assert.equal(copy.mismatches.length, 0);
  const traceAudit = [];
  for (const browser of ['chromium', 'firefox', 'webkit']) {
    const load = (phase, engine) => json(join(output, phase, 'browser', `${browser}-${engine}.json`));
    const [beforeSource, afterSource, beforeRust, afterRust] = await Promise.all([
      load('baseline', 'source'), load('fixed', 'source'), load('baseline', 'rust'), load('fixed', 'rust')]);
    assert.deepEqual(beforeSource, afterSource);
    const mismatches = [];
    for (let index = 0; index < afterSource.length; index++) {
      try { assert.deepEqual(afterRust[index], afterSource[index]); }
      catch { mismatches.push(afterSource[index].name); }
    }
    assert.deepEqual(mismatches, expectedFailures('fixed', browser));
    const inherited = entries => entries.filter(entry => !entry.name.startsWith(prefix) && entry.name !== missingLine);
    assert.equal(inherited(beforeRust).length, inheritedTotal); assert.deepEqual(inherited(beforeRust), inherited(afterRust));
    traceAudit.push({ browser, sourceStable: true, fixedExact: total - mismatches.length, fixedTotal: total,
      retainedFailures: mismatches, allocationCasesExact: 4, missingLineCaseExact: true, inheritedUnchanged: inheritedTotal,
      repaired: beforeRust.filter((entry, index) => JSON.stringify(entry) !== JSON.stringify(afterRust[index])).map(entry => entry.name) });
  }
  const before = await json(join(output, 'baseline/build-identity.json')), after = await json(join(output, 'fixed/build-identity.json'));
  try { await writeSources(baseline); assert.equal((await sourceIdentity(root)).sha256, before.input.sha256); }
  finally { await writeSources(fixed); }
  assert.equal((await sourceIdentity(root)).sha256, after.input.sha256);
  assert.deepEqual(await packageFiles(pkg), await json(join(output, 'fixed/package-files.json')));
  assert.notEqual(before.build.artifacts[1].sha256, after.build.artifacts[1].sha256);
  await save(join(output, 'summary.json'), { status: 'scoped-pass', reports, traceAudit, baselineInput: before.input, fixedInput: after.input,
    baselineWasm: before.build.artifacts[1], fixedWasm: after.build.artifacts[1],
    scope: 'Nonnumeric ANSI scrollback allocation phases521 and browser-native missing-line diagnostics526. Full346/322 and overall equivalence NOT PROVEN.' });
  const files = [];
  async function walk(relative) {
    for (const entry of await readdir(join(root, relative), { withFileTypes: true })) {
      const child = `${relative}/${entry.name}`; assert(!entry.isSymbolicLink());
      if (entry.isDirectory()) await walk(child); else if (entry.isFile() && entry.name !== 'pins.json') files.push(child);
    }
  }
  await walk('results/serialize-ansi-coercion');
  await save(join(output, 'pins.json'), { revision: git(['rev-parse', 'HEAD']).toString().trim(), reference, copy,
    input: await sourceIdentity(root), package: await verifyBuiltPackage(root, await sourceIdentity(root)),
    packageContents: await packageFiles(pkg),
    sources: await artifactIdentity(root, [...sources, 'scripts/serialize-browser-check.mjs', 'scripts/serialize-browser-cases.mjs']),
    artifacts: await artifactIdentity(root, files.sort()) });
}
console.log(`serialize-ansi-coercion ${mode} completed; inspect retained results for scope and failures`);
