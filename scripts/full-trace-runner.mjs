// Shared by explicit VT adapters. Both engines independently consume one trace.
import assert from 'node:assert/strict';
import fs from 'node:fs';
import path from 'node:path';
import { homedir } from 'node:os';
import { randomUUID, createHash } from 'node:crypto';
import { protocol, parseTrace, capture } from './full-trace-capture.mjs';
import { diskHeadroom, requireHeadroom, DEFAULT_MIN_FREE_MIB } from './quality-resources.mjs';
import { pinArtifact } from './acceptance-artifacts.mjs';

const digest = bytes => createHash('sha256').update(bytes).digest('hex');
function rustTool(name) {
  const home = process.env.CARGO_HOME ?? path.join(homedir(), '.cargo');
  const candidate = path.join(home, 'bin', process.platform === 'win32' ? `${name}.exe` : name);
  return process.env[name.toUpperCase()] ?? (fs.existsSync(candidate) ? candidate : name);
}
export async function runTrace(context, traces) {
  const { root, outputDirectory, signal } = context;
  fs.mkdirSync(outputDirectory, { recursive: true });
  const traceDirectory = path.join(outputDirectory, `trace-${randomUUID()}`);
  fs.mkdirSync(traceDirectory);
  const relative = file => path.relative(root, file).replaceAll('\\', '/');
  const input = path.join(traceDirectory, 'full-trace-input.tsv');
  const referenceFile = path.join(traceDirectory, 'full-trace-reference.tsv');
  const nativeFile = path.join(traceDirectory, 'full-trace-native.tsv');
  const sourceHashesFile = path.join(traceDirectory, 'full-trace-sources.json');
  const temp = path.join(root, 'tmp/full-trace-reference', randomUUID());
  const commands = [];
  fs.writeFileSync(input, protocol(traces));
  fs.writeFileSync(path.join(traceDirectory, 'full-trace-scenarios.json'), JSON.stringify(traces, null, 2) + '\n');
  const { runCommand } = await import('./quality-process.mjs');
  const resourceCheck = async () => {
    if (signal?.aborted) throw signal.reason ?? new Error('Acceptance case aborted');
    requireHeadroom(await diskHeadroom(root, DEFAULT_MIN_FREE_MIB));
  };
  async function command(name, program, args, timeoutMs) {
    await resourceCheck();
    const log = path.join(traceDirectory, `${name}.log`);
    const result = await runCommand(program, args, { cwd: root, env: process.env,
      log, timeoutMs, resourceCheck, signal });
    commands.push({ argv: [program, ...args], exitCode: result.exitCode, log: relative(log), process: result });
    assert.equal(result.status, 'passed', `${name} failed; see ${relative(log)}`);
    return fs.readFileSync(log, 'utf8');
  }
  await command('reference', process.execPath, ['scripts/full-trace-source.mjs', input,
    referenceFile, sourceHashesFile, temp], 120_000);
  const build = await command('cargo-build', rustTool('cargo'), ['build', '-p', 'xterm-core',
    '--example', 'full_trace', '--message-format=json'], 180_000);
  const artifacts = build.split(/\r?\n/).filter(line => line.startsWith('{')).map(line => JSON.parse(line))
    .filter(message => message.reason === 'compiler-artifact' && message.target?.name === 'full_trace' && message.executable);
  assert.equal(artifacts.length, 1, 'Exactly one native example artifact is required');
  const artifact = await pinArtifact(root, path.resolve(artifacts[0].executable));
  const executable = path.resolve(root, artifact.path);
  await command('native', executable, [input, nativeFile], 120_000);
  assert.equal(digest(fs.readFileSync(executable)), artifact.sha256, 'Pinned executable changed after execution');
  const runtimeVersion = (await command('rustc-version', rustTool('rustc'), ['--version'], 15_000)).trim();
  const reference = parseTrace(fs.readFileSync(referenceFile, 'utf8'));
  const rust = parseTrace(fs.readFileSync(nativeFile, 'utf8'));
  const captures = { reference: capture(reference), rust: capture(rust) };
  // Retain complete independent captures before equality so disagreements survive.
  fs.writeFileSync(path.join(traceDirectory, 'full-trace-captures.json'), JSON.stringify(captures, null, 2) + '\n');
  const operations = traces.reduce((count, trace) => count + trace.ops.length, 0);
  assert.equal(reference.length, operations, 'Source snapshots omitted trace operations');
  assert.equal(rust.length, reference.length, 'Native/source operation counts disagree');
  for (let index = 0; index < reference.length; index++) {
    assert.deepEqual(rust[index], reference[index], `first divergent terminal operation ${index}`);
  }
  return { corpus: { scenarios: traces.length, operations: reference.length,
      input: relative(input), sha256: digest(fs.readFileSync(input)),
      snapshotSchema: 'RICH2',
      scope: 'Completed handler-step state/events. Asynchronous WriteParsed and browser-only InputScroll notifications have separate acceptance cases.' },
    sourceHashes: JSON.parse(fs.readFileSync(sourceHashesFile, 'utf8')),
    artifactHashes: [artifact],
    commands, captures: [{ engine: 'native', implementation: 'rust-native', runtimeVersion,
      assertions: reference.length, ...captures }] };
}
