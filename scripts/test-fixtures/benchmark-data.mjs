// Synthetic protocol fixtures only, never performance evidence.
import { sha256, outputHash, caseIdentity, validateManifest } from '../benchmark-contract.mjs';
export function output() {
  const header = [2, 1, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0].join(',');
  const buffer = '0,0,0,0,0,0/0:4194304,0,0,,0,0;4194304,0,0,61,301,0,0';
  const saved = ['0,0', '0,0,0,0', '-', '', 0, '0,1', '0', 2].join('~');
  const meta = ['-', '', 0, 'd800', '', '-', '-', 0, 0, 1, 0, '62,6c,6f,63,6b', 0, '36'].join('~');
  return { terminalTrace: ['RICH2', header, buffer, buffer, saved, saved, meta, 0].join('|') + '\tD61\n', addons: {} };
}
export function manifest() {
  const bytes = Buffer.from('a\ud800', 'utf16le');
  const expectedOutput = output();
  return { schemaVersion: 1, execution: 'terminal', environment: { os: 'synthetic-test-only' }, warmup: 1, repetitions: 10,
    cases: [{ id: 'synthetic', options: { cols: 2, rows: 1, scrollback: 0, settings: {}, unicode: { provider: 'builtin', version: '6', settings: {} } },
      operations: [{ kind: 'write', encoding: 'utf16le', data: bytes.toString('base64') }], inputBytes: bytes.length,
      inputSha256: sha256(bytes), operationCount: 1, expectedOutput, expectedOutputSha256: outputHash(expectedOutput) }] };
}
export function result(input, engine) {
  return { schemaVersion: 1, engine, profile: engine === 'javascript' ? 'production' : 'release', manifestSha256: validateManifest(input),
    timing: { clock: 'monotonic', unit: 'ms', startupIncluded: false, completion: 'awaited' },
    build: { commands: ['synthetic fixture, not a build'], sourceSha256: '0'.repeat(64), artifactSha256: '1'.repeat(64), toolchain: 'synthetic', flags: 'synthetic' },
    cases: input.cases.map(row => {
      const sample = () => ({ durationMs: 1, completion: 'completed', pendingOperations: 0, output: structuredClone(row.expectedOutput), outputSha256: row.expectedOutputSha256 });
      return { id: row.id, caseSha256: caseIdentity(row), inputBytes: row.inputBytes, operationCount: row.operationCount,
        warmup: Array.from({ length: input.warmup }, sample), measurements: Array.from({ length: input.repetitions }, sample) };
    }) };
}
