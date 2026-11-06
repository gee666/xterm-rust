import assert from 'node:assert/strict';
import { readFileSync, writeFileSync, mkdtempSync, rmSync, lstatSync } from 'node:fs';
import { createHash } from 'node:crypto';
import { dirname, join, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';
import { jsonHash, validateManifest, caseIdentity, outputHash, validateResult } from './benchmark-contract.mjs';
import { loadEngine, prepare, sample } from './matched-benchmark-js.mjs';
import { runCommand } from './quality-process.mjs';
export const sha256 = path => createHash('sha256').update(readFileSync(path)).digest('hex');
export function runtimePins(root) {
  const names = ['matched-benchmark-adapter', 'matched-benchmark-js', 'matched-benchmark-workloads',
    'matched-benchmark-preflight', 'matched-benchmark-native-build', 'benchmark-runner', 'benchmark-contract',
    'full-trace-source', 'full-trace-capture', 'full-trace-codec', 'terminal-snapshot', 'upstream-baseline-utils',
    'quality-process', 'quality-tree', 'quality-managed', 'quality-failure'];
  return names.map(name => ({ name: `scripts/${name}.mjs`, path: join(root, 'scripts', `${name}.mjs`),
    sha256: sha256(join(root, 'scripts', `${name}.mjs`)) }));
}
export function verify(provenance) {
  assert.ok(Array.isArray(provenance.sources) && provenance.sources.length, 'Missing source pins');
  assert.equal(new Set(provenance.sources.map(pin => pin.name)).size, provenance.sources.length, 'Duplicate source pins');
  assert.ok(Object.keys(provenance.artifacts).length, 'Missing artifact pins');
  for (const pin of [...provenance.sources, ...Object.values(provenance.artifacts), ...(provenance.runtime ?? [])]) {
    assert.ok(lstatSync(pin.path).isFile() && !lstatSync(pin.path).isSymbolicLink(), 'Expected a physical pinned file');
    assert.equal(sha256(pin.path), pin.sha256, `Input drift: ${pin.path}`);
  }
  assert.equal(jsonHash(provenance.sources.map(({ name, sha256 }) => ({ name, sha256 }))), provenance.build.sourceSha256);
  assert.equal(jsonHash(Object.entries(provenance.artifacts).map(([name, pin]) => ({ name, sha256: pin.sha256 }))), provenance.build.artifactSha256);
  return provenance.build;
}
export function prepared(manifest) {
  return { warmup: manifest.warmup, repetitions: manifest.repetitions, cases: manifest.cases.map(row => ({
    id: row.id, options: row.options, operations: prepare(row).map(operation => {
      if (operation.kind !== 'write') return operation;
      const bytes = Buffer.from(operation.data, 'base64');
      const units = operation.encoding === 'utf16le' ?
        Array.from({ length: bytes.length / 2 }, (_, i) => bytes.readUInt16LE(i * 2)) : [...bytes];
      return { kind: 'write', encoding: operation.encoding, units: units.map(unit => unit.toString(16)).join(',') };
    }) })) };
}
export function checkPairedReceipt(pairedReceipt, manifestSha256, build, engine) {
  assert.equal(pairedReceipt.status, 'passed', 'Actual paired capture must pass before timing');
  assert.equal(pairedReceipt.manifestSha256, manifestSha256, 'Stale paired workload receipt');
  assert.equal(pairedReceipt.builds[engine].sourceSha256, build.sourceSha256, 'Stale paired source receipt');
  assert.equal(pairedReceipt.builds[engine].artifactSha256, build.artifactSha256, 'Stale paired artifact receipt');
  assert.equal(pairedReceipt.builds[engine].flags, build.flags, 'Stale paired build or adapter flags');
}
export async function adapt({ engine, provenance, manifest, pairedReceipt, workerOutput, timeoutMs = 120000 }) {
  assert.ok(['javascript', 'rust'].includes(engine));
  const manifestSha256 = validateManifest(manifest);
  const build = verify(provenance);
  checkPairedReceipt(pairedReceipt, manifestSha256, build, engine);
  let raw;
  if (engine === 'javascript') {
    const implementation = await loadEngine(provenance.artifacts);
    raw = [];
    for (const row of manifest.cases) {
      const operations = prepare(row), warmup = [], measurements = [];
      for (let i = 0; i < manifest.warmup; i++) warmup.push(await sample(row, implementation, operations));
      for (let i = 0; i < manifest.repetitions; i++) measurements.push(await sample(row, implementation, operations));
      raw.push({ id: row.id, warmup, measurements });
    }
  } else {
    // Retain native raw files on worker failure until the outer runner captures
    // diagnostics; finally removes this owned child on both success and failure.
    const scratch = mkdtempSync(join(dirname(workerOutput), 'native-'));
    try {
      const input = join(scratch, 'prepared.json'), output = join(scratch, 'raw.json'), log = join(scratch, 'worker.log');
      writeFileSync(input, JSON.stringify(prepared(manifest)));
      const command = await runCommand(provenance.artifacts.native.path, [input, output],
        { cwd: dirname(provenance.artifacts.native.path), env: process.env, log, timeoutMs,
          resourceCheck: () => { for (const file of [output, log]) {
            try { assert.ok(lstatSync(file).size <= 8 * 1024 * 1024, 'Worker output limit'); }
            catch (error) { if (error.code !== 'ENOENT') throw error; }
          } } });
      console.log(JSON.stringify(command));
      console.log(readFileSync(log, 'utf8'));
      if (command.status !== 'passed') {
        try { writeFileSync(workerOutput, readFileSync(output)); } catch (error) { if (error.code !== 'ENOENT') throw error; }
        throw new Error('Native worker failed');
      }
      raw = JSON.parse(readFileSync(output, 'utf8'));
    } finally { rmSync(scratch, { recursive: true }); }
  }
  verify(provenance);
  assert.equal(raw.length, manifest.cases.length);
  const result = { schemaVersion: 1, engine, profile: engine === 'rust' ? 'release' : 'production', manifestSha256,
    timing: { clock: 'monotonic', unit: 'ms', startupIncluded: false, completion: 'awaited' }, build,
    cases: raw.map((row, i) => {
      assert.equal(row.id, manifest.cases[i].id);
      for (const field of ['warmup', 'measurements']) for (const sample of row[field]) sample.outputSha256 = outputHash(sample.output);
      return { ...row, caseSha256: caseIdentity(manifest.cases[i]), inputBytes: manifest.cases[i].inputBytes,
        operationCount: manifest.cases[i].operationCount };
    }) };
  // Write the entire actual result before comparing. Mismatches must survive.
  writeFileSync(workerOutput, JSON.stringify(result));
  validateResult(manifest, result, engine);
  return result;
}
if (process.argv[1] && resolve(process.argv[1]) === fileURLToPath(import.meta.url)) {
  const args = process.argv.slice(2), values = {};
  for (let i = 0; i < args.length; i += 2) {
    assert.ok(['--engine', '--provenance', '--paired-receipt', '--benchmark-manifest', '--benchmark-output'].includes(args[i]) && !Object.hasOwn(values, args[i]) && args[i + 1]);
    values[args[i]] = args[i + 1];
  }
  await adapt({ engine: values['--engine'], provenance: JSON.parse(readFileSync(values['--provenance'], 'utf8')),
    pairedReceipt: JSON.parse(readFileSync(values['--paired-receipt'], 'utf8')),
    manifest: JSON.parse(readFileSync(values['--benchmark-manifest'], 'utf8')), workerOutput: values['--benchmark-output'] });
}
