// Runs original SerializeAddon assertions and snapshots exact JS UTF-16 output.
// Source and assertions stay read-only; fixture adapters do not parse terminal input.
import { createRequire } from 'node:module';
import { readFile, writeFile, mkdir, statfs } from 'node:fs/promises';
import { createHash } from 'node:crypto';
import { writeFileSync } from 'node:fs';
import { fileURLToPath } from 'node:url';
import path from 'node:path';
import assert from 'node:assert/strict';
import { FixtureWriter, directedCases, headerCases, seededCases } from './serialize-oracle-support.mjs';

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..');
const require = createRequire(path.join(root, 'source_repo/package.json'));
const temp = path.join(root, 'tmp/serialize-oracle');
const results = path.join(root, 'results/serialize');
const fixture = path.join(root, 'crates/xterm-core/tests/fixtures/serialize-oracle.tsv');
await mkdir(results, { recursive: true });
const attempt = { command: 'node scripts/serialize-oracle.mjs', startedAt: new Date().toISOString(), node: process.version };
const receipt = path.join(results, 'last-attempt.json');
await writeFile(receipt, JSON.stringify({ ...attempt, status: 'running' }, null, 2) + '\n');
process.once('uncaughtExceptionMonitor', error => {
  writeFileSync(receipt, JSON.stringify({ ...attempt, status: 'failed', error: String(error.stack) }, null, 2) + '\n');
});
await mkdir(temp, { recursive: true });
const { build } = require('esbuild');
const Mocha = require('mocha');
const disk = await statfs(root);
assert.ok(disk.bavail * disk.bsize >= 256 * 1024 ** 2, 'Insufficient free disk space for oracle');
const source = 'source_repo/addons/addon-serialize/src/SerializeAddon.ts';
const testSource = 'source_repo/addons/addon-serialize/src/SerializeAddon.test.ts';
const referenceFile = path.join(temp, 'reference.cjs');
const sourceHashes = {};
const hash = bytes => createHash('sha256').update(bytes).digest('hex');
async function bundle(options) {
  const result = await build({ absWorkingDir: root, bundle: true, platform: 'node',
    format: 'cjs', target: 'node24', tsconfig: 'source_repo/src/browser/tsconfig.json',
    alias: { common: path.join(root, 'source_repo/src/common'), browser: path.join(root, 'source_repo/src/browser') },
    metafile: true, logLevel: 'silent', ...options });
  for (const file of Object.keys(result.metafile.inputs).filter(file => file.startsWith('source_repo/src/') || file.startsWith('source_repo/addons/'))) {
    sourceHashes[file] = hash(await readFile(path.join(root, file)));
  }
}
await bundle({ stdin: { resolveDir: root, contents: [
  `export { SerializeAddon } from './${source}';`,
  "export { CellData } from './source_repo/src/common/buffer/CellData.ts';",
  "export { AttributeData } from './source_repo/src/common/buffer/AttributeData.ts';",
  "export { DEFAULT_ANSI_COLORS } from './source_repo/src/browser/Types.ts';"
].join('\n'), sourcefile: 'serialize-oracle-entry.ts' }, outfile: referenceFile });
const reference = require(referenceFile);
const writer = new FixtureWriter(reference.DEFAULT_ANSI_COLORS.map(color => color.css));
let currentTest = 'suite setup';
globalThis.__serializeCapture = (terminal, options, kind, output, error) =>
  writer.capture(terminal, options, kind, output, error, currentTest);
const baselineFile = path.join(temp, 'upstream.cjs');
// The subclass only records inputs/results after calling the unchanged source.
// Every original assertion executes against the same source return value/error.
await bundle({ entryPoints: [testSource], outfile: baselineFile, plugins: [{
  name: 'serialize-observer', setup(build) {
    build.onResolve({ filter: /^(chai|jsdom)$/ }, args => ({ path: require.resolve(args.path), external: true }));
    build.onResolve({ filter: /^\.\/SerializeAddon$/ }, args => {
      if (args.importer.replaceAll('\\', '/').endsWith('/SerializeAddon.test.ts')) return { path: 'hook', namespace: 'serialize-observer' };
    });
    build.onResolve({ filter: /.*/, namespace: 'serialize-observer' }, args => ({ path: args.path, external: true }));
    build.onLoad({ filter: /.*/, namespace: 'serialize-observer' }, () => ({
      loader: 'js', contents: `const { SerializeAddon: Base } = require(${JSON.stringify(referenceFile)});
export class SerializeAddon extends Base {
  serialize(options) { return this.observe('ansi', options, () => super.serialize(options)); }
  serializeAsHTML(options) { return this.observe('html', options, () => super.serializeAsHTML(options)); }
  observe(kind, options, callback) {
    let value;
    try { value = callback(); }
    catch (error) { globalThis.__serializeCapture(this._terminal, options, kind, undefined, error); throw error; }
    globalThis.__serializeCapture(this._terminal, options, kind, value, undefined);
    return value;
  }
}`
    }));
  }
}] });
const mocha = new Mocha({ timeout: 10000, reporter: 'spec' });
mocha.addFile(baselineFile);
const outcomes = [];
const assertionCounts = new Map();
const chai = require('chai');
const savedAssertions = new Map();
for (const method of ['equal', 'notEqual', 'ok']) {
  const original = chai.assert[method];
  savedAssertions.set(method, original);
  chai.assert[method] = function (...args) {
    const counts = assertionCounts.get(currentTest) || {};
    counts[method] = (counts[method] || 0) + 1;
    assertionCounts.set(currentTest, counts);
    return original.apply(this, args);
  };
}
let discovered = 0;
let failures = 0;
try {
  await new Promise(resolve => {
    const runner = mocha.run(count => { failures = count; resolve(); });
    discovered = runner.total;
    runner.on('test', test => { currentTest = test.fullTitle(); });
    runner.on('pass', test => outcomes.push({ name: test.fullTitle(), status: 'passed' }));
    runner.on('pending', test => outcomes.push({ name: test.fullTitle(), status: 'pending' }));
    runner.on('fail', (test, error) => outcomes.push({ name: test.fullTitle(), status: 'failed', error: String(error.stack) }));
  });
} finally {
  delete globalThis.__serializeCapture;
  for (const [method, original] of savedAssertions) chai.assert[method] = original;
}
const originalCaptures = writer.cases.length;
const directed = directedCases(reference, (...args) => writer.capture(...args));
const seeded = seededCases(reference, (...args) => writer.capture(...args));
const headers = headerCases(reference, (...args) => writer.capture(...args));
await writeFile(fixture, writer.text());
const count = status => outcomes.filter(test => test.status === status).length;
for (const outcome of outcomes) outcome.assertions = assertionCounts.get(outcome.name) || {};
const assertionCalls = outcomes.reduce((total, outcome) => total + Object.values(outcome.assertions).reduce((a, b) => a + b, 0), 0);
for (const [file, expected] of Object.entries(sourceHashes)) {
  assert.equal(hash(await readFile(path.join(root, file))), expected, `Reference changed while running: ${file}`);
}
const report = {
  command: 'node scripts/serialize-oracle.mjs',
  environment: { node: process.version, platform: process.platform, arch: process.arch,
    esbuild: require('esbuild/package.json').version, mocha: require('mocha/package.json').version },
  sourceHashes, source, testSource, referenceUnchanged: true,
  instrumentation: 'Original assertion bodies bundled unchanged. A subclass records source serializer inputs/returns/errors; Chai methods retain original behavior and count calls.',
  baseline: { discovered, assertionCalls, executed: count('passed') + count('failed'), passed: count('passed'), failed: count('failed'),
    pending: count('pending'), skipped: 0, unrun: discovered - outcomes.length, outcomes },
  fixtures: { path: path.relative(root, fixture).replaceAll('\\', '/'), sha256: hash(await readFile(fixture)),
    cases: writer.cases.length, originalCaptures, directed, seeded, headers, palettes: writer.palettes.size,
    ansi: writer.cases.filter(test => test.kind === 'ansi').length,
    html: writer.cases.filter(test => test.kind === 'html').length,
    errors: writer.cases.filter(test => test.status === 'error').length },
  cases: writer.cases,
  scope: 'Exact serializer function results from original cases and raw BufferAccess inputs. Original terminal round-trips are JS baseline only; Rust integration and browser binding tests remain task88.'
};
await writeFile(path.join(results, 'oracle.json'), JSON.stringify(report, null, 2) + '\n');
assert.equal(failures, 0, 'Original serialize assertions failed');
assert.equal(report.baseline.unrun, 0, 'Original test discovery mismatch');
assert.equal(discovered, 44, 'Original suite case count changed');
assert.equal(assertionCalls, 60, 'Original assertion count changed');
assert.equal(originalCaptures, 44, 'Original serializer capture count changed');
assert.equal(seeded.outputs + directed.outputs + headers.outputs + originalCaptures, writer.cases.length);
await writeFile(receipt, JSON.stringify({ ...attempt, status: 'passed', fixtureSha256: report.fixtures.sha256 }, null, 2) + '\n');
console.log(JSON.stringify({ baseline: { ...report.baseline, outcomes: undefined }, fixtures: report.fixtures }));
