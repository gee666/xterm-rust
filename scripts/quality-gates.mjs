#!/usr/bin/env node
import { snapshot } from './quality-snapshot.mjs';
import { diskHeadroom, requireHeadroom } from './quality-resources.mjs';
import { parseOptions, reserveOutput, assertNativeHost, assertBuildEnvironment, assertResolvedTarget, assertSharedTarget, testCounts } from './quality-policy.mjs';
import { readFile, writeFile } from 'node:fs/promises';
import { relative, join, sep } from 'node:path';
import { fileURLToPath } from 'node:url';
import os from 'node:os';
import { runCommand } from './quality-process.mjs';

const root = fileURLToPath(new URL('../', import.meta.url));
const { destination, timeoutMs, wasm, minFreeMiB } = parseOptions(process.argv.slice(2), process.platform, process.arch);
// Atomically reserve a new attempt and reject symlinked output parents.
const out = await reserveOutput(root, destination);
const report = {
  schemaVersion: 1, startedAt: new Date().toISOString(), overallEquivalence: 'NOT PROVEN',
  environment: {
    platform: process.platform, architecture: process.arch, release: os.release(),
    cpu: os.cpus()[0]?.model ?? null, node: process.version,
    ci: process.env.GITHUB_ACTIONS === 'true',
    ciCommit: process.env.GITHUB_SHA ?? null,
    execution: /microsoft/i.test(os.release()) ? 'WSL Linux runtime' : process.env.GITHUB_ACTIONS === 'true' ? 'GitHub Actions runner' : 'local OS runtime',
    runnerEnvironment: process.env.RUNNER_ENVIRONMENT ?? null,
    ciRunUrl: process.env.GITHUB_ACTIONS === 'true'
      ? `${process.env.GITHUB_SERVER_URL}/${process.env.GITHUB_REPOSITORY}/actions/runs/${process.env.GITHUB_RUN_ID}` : null,
    declaredTarget: process.env.CARGO_BUILD_TARGET ?? null,
    cargoIncremental: process.env.CARGO_INCREMENTAL ?? null,
    cargoEncodedRustflags: process.env.CARGO_ENCODED_RUSTFLAGS ?? null,
    rustflags: process.env.RUSTFLAGS ?? null
  },
  disk: { minFreeMiB, observations: [] },
  commands: [], gaps: [], featurePolicy: 'default, no-default, all, and each named feature individually'
};
async function checkDisk(stage) {
  const observation = await diskHeadroom(root, minFreeMiB);
  report.disk.observations.push({ stage, observedAt: new Date().toISOString(), ...observation });
  requireHeadroom(observation);
}
async function run(id, command, commandArgs) {
  await checkDisk(`before-${id}`);
  const log = join(out, `${id}.log`);
  console.log(`Running ${id}: ${command} ${commandArgs.join(' ')}`);
  const result = await runCommand(command, commandArgs, {
    cwd: root, env: process.env, timeoutMs, log,
    resourceCheck: () => checkDisk(`during-${id}`)
  });
  result.id = id;
  result.testSummaries = testCounts(await readFile(log, 'utf8'));
  result.log = relative(root, log).split(sep).join('/');
  report.commands.push(result);
  await writeFile(join(out, 'summary.json'), JSON.stringify(report, null, 2) + '\n');
  console.log(`${id}: ${result.status}`);
  await checkDisk(`after-${id}`);
  if (result.resourceError) throw new Error(result.resourceError);
  return result;
}
async function textLog(id) { return readFile(join(out, `${id}.log`), 'utf8'); }
let fatal = null;
try {
  report.sourceBefore = await snapshot(root);
  report.environment.buildOverrideKeys = Object.keys(process.env).filter(key => key === 'CARGO_TARGET_DIR' || key === 'CARGO_INCREMENTAL' || /^CARGO_PROFILE_(DEV|TEST|RELEASE)_/.test(key)).sort();
  assertBuildEnvironment(process.env);
  await run('git-revision', 'git', ['rev-parse', 'HEAD']);
  await run('git-status', 'git', ['status', '--short']);
  await run('rustc-version', 'rustc', ['-vV']);
  await run('cargo-version', 'cargo', ['--version']);
  await run('installed-targets', 'rustup', ['target', 'list', '--installed']);
  await run('installed-components', 'rustup', ['component', 'list', '--installed']);
  const host = (await textLog('rustc-version')).match(/^host: (.+)$/m)?.[1]?.trim();
  report.environment.rustHost = host ?? null;
  assertNativeHost(process.platform, host, process.env.CARGO_BUILD_TARGET);
  const metadataResult = await run('metadata', 'cargo', ['metadata', '--no-deps', '--format-version', '1', '--locked']);
  if (metadataResult.status !== 'passed') throw new Error('Cargo metadata failed; feature inventory unavailable');
  const metadata = JSON.parse(await textLog('metadata'));
  report.environment.cargoTargetDirectory = metadata.target_directory;
  assertSharedTarget(root, metadata.target_directory, process.platform);
  const cfg = await run('resolved-native-target', 'cargo', ['rustc', '-p', 'xterm-core', '--lib', '--locked', '--', '--print', 'cfg']);
  if (cfg.status !== 'passed') throw new Error('Cannot observe the resolved Cargo target before native tests');
  report.environment.resolvedCargoTargetOs = assertResolvedTarget(process.platform, await textLog('resolved-native-target'));
  const packages = metadata.packages.filter(pkg => metadata.workspace_members.includes(pkg.id));
  report.packages = packages.map(pkg => ({ name: pkg.name, rustVersion: pkg.rust_version, features: pkg.features }));
  const workspace = ['--workspace', '--locked'];
  // Native commands share Cargo's existing target directory. Do not create a
  // duplicate target/<host>/ tree merely to restate the verified native host.
  await run('fmt', 'cargo', ['fmt', '--all', '--', '--check']);
  await run('clippy-all-features', 'cargo', ['clippy', ...workspace, '--all-targets', '--all-features', '--', '-D', 'warnings']);
  await run('test-all-features', 'cargo', ['test', ...workspace, '--all-targets', '--all-features']);
  await run('doctest-all-features', 'cargo', ['test', ...workspace, '--doc', '--all-features']);
  await run('build-release', 'cargo', ['build', ...workspace, '--release', '--all-targets', '--all-features']);
  await run('test-default', 'cargo', ['test', ...workspace, '--all-targets']);
  await run('test-no-default-features', 'cargo', ['test', ...workspace, '--all-targets', '--no-default-features']);
  await run('clippy-no-default-features', 'cargo', ['clippy', ...workspace, '--all-targets', '--no-default-features', '--', '-D', 'warnings']);
  for (const pkg of packages) {
    for (const feature of Object.keys(pkg.features).filter(value => value !== 'default').sort()) {
      const label = `${pkg.name}-${feature}`.replace(/[^\w.-]/g, '_');
      await run(`feature-${label}`, 'cargo', ['test', '--locked', '-p', pkg.name, '--all-targets', '--no-default-features', '--features', feature]);
    }
  }
  if (wasm) {
    await run('wasm-check', 'cargo', ['check', ...workspace, '--lib', '--all-features', '--target', 'wasm32-unknown-unknown']);
    report.gaps.push('WASM compilation is not browser runtime evidence. Browser E2E must execute separately.');
  } else report.gaps.push('WASM target/build not requested; rerun with --wasm-check after installing the target.');
  const miri = await run('miri-availability', 'cargo', ['miri', '--version']);
  report.safety = {
    scan: 'Lexical unsafe candidates in workspace Rust files, including comments and strings. Not a semantic proof.',
    candidates: report.sourceBefore.unsafeCandidates,
    miriAvailability: miri.status,
    miriExecuted: false,
    sanitizerExecuted: false
  };
  report.gaps.push('Miri and sanitizers were not run. Review unsafe candidates and choose an applicable toolchain/target.');
  report.gaps.push('Native library tests do not establish terminal/demo/browser/addon equivalence or other OS runtime success.');
  report.sourceAfter = await snapshot(root);
  report.sourceStable = report.sourceBefore.sha256 === report.sourceAfter.sha256;
  if (!report.sourceStable) report.gaps.push('Workspace changed during execution. Repeat on an immutable commit before claiming one-snapshot gate success.');
} catch (error) {
  fatal = error.message;
  report.gaps.push(fatal);
}
report.finishedAt = new Date().toISOString();
report.fatal = fatal;
const optional = new Set(['miri-availability']);
report.failedGates = report.commands.filter(item => item.status !== 'passed' && !optional.has(item.id)).map(item => item.id);
report.gateStatus = !fatal && report.failedGates.length === 0 && report.sourceStable ? 'passed' : 'failed';
await writeFile(join(out, 'summary.json'), JSON.stringify(report, null, 2) + '\n');
console.log(`Native quality gates: ${report.gateStatus}. Overall equivalence: NOT PROVEN. Results: ${relative(root, out)}`);
if (report.gateStatus !== 'passed') process.exitCode = 1;
