// Scoped oracle lifecycle, using the project's shared evidence persistence.
import { createEvidenceRun, persistEvidence } from './upstream-baseline-utils.mjs';
import { readFileSync } from 'node:fs';
import { join, relative } from 'node:path';

/** Allocate before setup, preserve partial stages, and never reuse older success. */
export function recordSyncCommand(context, command, result, logs = {}) {
  context.recordCommand({ command, exitCode: result.status, signal: result.signal ?? null,
    error: result.error?.message ?? null, timedOut: result.error?.code === 'ETIMEDOUT', ...logs,
    status: result.status === 0 && !result.error && !result.signal ? 'passed' : 'failed' });
}

export async function runScopedOracle(directory, metadata, execute) {
  let report = { ...metadata, node: process.version, platform: process.platform,
    architecture: process.arch, commands: [], status: 'incomplete', phase: 'setup' };
  const attempt = createEvidenceRun(directory, report);
  let failed = false;
  let originalError;
  const context = {
    directory: attempt.directory,
    runId: attempt.id,
    checkpoint(partial) {
      report = { ...report, ...partial, status: 'incomplete' };
      persistEvidence(attempt, report);
    },
    recordCommand(command) {
      context.checkpoint({ commands: [...report.commands, command] });
    }
  };
  try {
    const result = await execute(context);
    if (result?.status !== 'passed') throw new Error('Oracle did not return a completed passing report');
    report = { ...report, ...result, runId: attempt.id, phase: 'complete', activeCommand: null };
    return report;
  } catch (error) {
    failed = true;
    originalError = error;
    report = { ...report, status: 'failed', error: {
      name: error?.name ?? typeof error, message: String(error?.message ?? error),
      code: error?.code ?? null, exitCode: error?.exitCode ?? error?.status ?? null,
      signal: error?.signal ?? null, timedOut: error?.code === 'ETIMEDOUT'
    } };
    throw error;
  } finally {
    try { persistEvidence(attempt, report); }
    catch (persistenceError) {
      if (failed) throw new AggregateError([originalError, persistenceError], 'Oracle and evidence persistence both failed');
      throw persistenceError;
    }
  }
}

/** Build via bounded process-tree handling; execute only this native artifact. */
export async function buildNativeOracle(context, root, example) {
  const runtime = Object.assign({}, ...await Promise.all([
    import('./quality-process.mjs'), import('./quality-resources.mjs'), import('./quality-policy.mjs'), import('./acceptance-artifacts.mjs')
  ]));
  runtime.assertBuildEnvironment(process.env);
  async function checkDisk() {
    runtime.requireHeadroom(await runtime.diskHeadroom(root, runtime.DEFAULT_MIN_FREE_MIB));
    try { runtime.requireHeadroom(await runtime.diskHeadroom(join(root, 'target'), runtime.DEFAULT_MIN_FREE_MIB)); }
    catch (error) { if (error.code !== 'ENOENT') throw error; }
  }
  async function success(name, command, args) {
    context.checkpoint({ phase: name, activeCommand: [command, ...args] });
    await checkDisk();
    const log = join(context.directory, `${name}.log`);
    const result = await runtime.runCommand(command, args, {
      cwd: root, env: process.env, log, timeoutMs: 120000, resourceCheck: checkDisk
    });
    context.recordCommand({ ...result, log: relative(root, log).replaceAll('\\', '/') });
    if (result.status !== 'passed') throw Object.assign(new Error(`${name} failed; inspect ${log}`), {
      exitCode: result.exitCode, signal: result.signal, code: result.timedOut ? 'ETIMEDOUT' : 'COMMAND_FAILED'
    });
    return readFileSync(log, 'utf8');
  }
  const cargo = process.env.CARGO ?? 'cargo';
  const rustc = await success('rustc-version', process.env.RUSTC ?? 'rustc', ['-vV']);
  const host = rustc.match(/^host:\s*(.+)$/m)?.[1];
  runtime.assertNativeHost(process.platform, host, process.env.CARGO_BUILD_TARGET);
  const cargoVersion = await success('cargo-version', cargo, ['--version']);
  const metadataText = await success('cargo-metadata', cargo, ['metadata', '--locked', '--no-deps', '--format-version', '1']);
  const metadataLines = metadataText.split(/\r?\n/).filter(line => line.startsWith('{'));
  if (metadataLines.length !== 1) throw new Error('Expected one Cargo metadata object');
  const metadata = JSON.parse(metadataLines[0]);
  runtime.assertSharedTarget(root, metadata.target_directory, process.platform);
  const cfg = await success('native-target-cfg', cargo, ['rustc', '--locked', '-p', 'xterm-core', '--lib', '--', '--print', 'cfg']);
  const targetOs = runtime.assertResolvedTarget(process.platform, cfg);
  const build = await success('native-build', cargo, ['build', '--locked', '-p', 'xterm-core', '--example', example, '--message-format=json-render-diagnostics']);
  const artifacts = build.split(/\r?\n/).filter(line => line.startsWith('{')).map(line => JSON.parse(line))
    .filter(message => message.reason === 'compiler-artifact' && message.target.name === example && message.target.kind.includes('example') && !message.profile.test && message.executable);
  if (artifacts.length !== 1) throw new Error('Expected one executable from this Cargo build');
  const compilerArtifact = artifacts[0].executable;
  const pinned = await runtime.pinArtifact(root, compilerArtifact);
  const executable = join(root, pinned.path);
  const info = { executable, compilerArtifact, rustc: rustc.split(/\r?\n/)[0], host, targetOs,
    cargo: cargoVersion.trim(), targetDirectory: metadata.target_directory,
    sha256: pinned.sha256, bytes: pinned.bytes };
  context.checkpoint({ phase: 'native-ready', nativeArtifact: info });
  await checkDisk();
  return info;
}
