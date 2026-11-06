import { createHash } from 'node:crypto';
import { readFileSync, writeFileSync, mkdtempSync, rmSync } from 'node:fs';
import { resolve, dirname, join } from 'node:path';
import { fileURLToPath } from 'node:url';
import os from 'node:os';
import { outputHash, validateManifest } from './benchmark-contract.mjs';
import { verify } from './matched-benchmark-adapter.mjs';
import { loadEngine, sample } from './matched-benchmark-js.mjs';
import { runCommand } from './quality-process.mjs';
const write = (text, encoding = 'utf16le') => ({ kind: 'write', encoding,
  data: Buffer.from(text, encoding === 'utf16le' ? 'utf16le' : 'utf8').toString('base64') });
const resize = (cols, rows) => ({ kind: 'resize', cols, rows });
const serialize = { kind: 'addon', addon: 'serialize', method: 'serialize', arguments: [] };
export function workloads() {
  const options = (provider = 'builtin', version = '6') => ({ cols: 32, rows: 8, scrollback: 16,
    unicode: { provider, version, settings: {} }, settings: { allowProposedApi: true } });
  const ascii = 'The quick brown fox jumps over 0123456789.\r\n'.repeat(1024);
  const unicode = '界語🙂 e\u0301 \u{1f1fa}\u{1f1f8} \u{1f469}\u200d\u{1f4bb}\r\n'.repeat(512);
  const ansi = '\x1b[31;1mred\x1b[0m\tplain\r\n\x1b[2;3Hmove\x1b[K\x1b[6n'.repeat(256);
  return [
    { id: 'ascii-utf16', options: options(), operations: [write(ascii)] },
    { id: 'ascii-utf8', options: options(), operations: [write(ascii, 'utf8-bytes')] },
    { id: 'unicode11-stream', options: options('unicode11', '11'), operations: [write(unicode.slice(0, 23)), write(unicode.slice(23))] },
    { id: 'graphemes-stream', options: options('unicode-graphemes', '15-graphemes'), operations: [write(unicode.slice(0, 7)), write(unicode.slice(7))] },
    { id: 'ansi-controls-replies', options: options(), operations: [write(ansi)] },
    { id: 'scrollback-resize-reflow', options: options(), operations: [write(ascii.slice(0, 4096)), resize(20, 6), write('tail\r\n'.repeat(128)), resize(40, 10)] },
    { id: 'serialize-live-addon', options: options('unicode11', '11'), operations: [write(unicode.slice(0, 4096)), serialize, write('\x1b[32mfinal\x1b[0m'), serialize] }
  ];
}
export async function capture(provenance, environment) {
  verify(provenance);
  const engine = await loadEngine(provenance.artifacts), cases = [];
  for (const row of workloads()) {
    const input = createHash('sha256');
    let inputBytes = 0;
    for (const op of row.operations.filter(op => op.kind === 'write')) {
      const bytes = Buffer.from(op.data, 'base64'); inputBytes += bytes.length; input.update(bytes);
    }
    const expectedOutput = (await sample(row, engine, undefined, environment.writeMode)).output;
    cases.push({ ...row, inputBytes, inputSha256: input.digest('hex'), operationCount: row.operations.length,
      expectedOutput, expectedOutputSha256: outputHash(expectedOutput) });
  }
  verify(provenance);
  const manifest = { schemaVersion: 1, execution: 'terminal', environment, warmup: 2, repetitions: 10, cases };
  validateManifest(manifest);
  return manifest;
}
if (process.argv[1] && resolve(process.argv[1]) === fileURLToPath(import.meta.url)) {
  const synchronous = process.argv.length === 6 && process.argv[5] === '--synchronous-core';
  if (process.argv.length !== 5 && !synchronous) throw new Error('Usage: matched-benchmark-workloads.mjs JS_PROVENANCE.json RUST_PROVENANCE.json MANIFEST.json [--synchronous-core]');
  const provenance = JSON.parse(readFileSync(process.argv[2], 'utf8'));
  const rust = JSON.parse(readFileSync(process.argv[3], 'utf8'));
  verify(rust);
  let minimum, previous = process.hrtime.bigint();
  for (let i = 0; i < 10000; i++) {
    const current = process.hrtime.bigint(), delta = current - previous;
    if (delta > 0n && (minimum === undefined || delta < minimum)) minimum = delta;
    previous = current;
  }
  if (minimum === undefined) throw new Error('No positive monotonic clock delta');
  const scratch = mkdtempSync(join(process.env.TMPDIR ?? 'tmp', 'clock-probe-'));
  let nativeClock;
  try {
    const output = join(scratch, 'clock.json'), log = join(scratch, 'clock.log');
    const command = await runCommand(rust.artifacts.native.path, ['--clock-probe', output],
      { cwd: dirname(rust.artifacts.native.path), env: process.env, log, timeoutMs: 10000 });
    if (command.status !== 'passed') throw new Error(`Native clock probe failed: ${JSON.stringify(command)}`);
    nativeClock = JSON.parse(readFileSync(output, 'utf8'));
  } finally { rmSync(scratch, { recursive: true }); }
  verify(rust);
  const manifest = await capture(provenance, { cpu: os.cpus()[0].model, logicalProcessors: os.cpus().length,
    os: `${os.type()} ${os.release()} ${os.version()}`, arch: os.arch(), node: process.version,
    clock: 'JS process.hrtime.bigint and Rust std::time::Instant',
    clockResolution: { javascriptMinimumObservedDeltaNs: Number(minimum), rust: nativeClock,
      method: 'minimum positive consecutive delta across 10000 reads outside measurements; observed granularity, not a clock accuracy claim' },
    concurrency: 'one measured engine at a time; inspect retained resource receipt for other machine activity',
    order: 'all JavaScript cases then all Rust cases; 2 fresh-terminal warmups then 10 fresh-terminal measurements per case',
    writeMode: synchronous ? 'synchronous-core' : 'public-callback',
    scope: synchronous ? 'deprecated internal production JS CoreTerminal.writeSync versus native synchronous CoreTerminal, builtin synchronous handlers only; no public API/async/browser/startup claim' :
      'headless public JS callback versus native synchronous Rust engine, not browser rendering or startup' });
  writeFileSync(process.argv[4], JSON.stringify(manifest));
}
