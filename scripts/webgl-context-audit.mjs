// Copyright (c) 2026 The xterm.js authors. All rights reserved. MIT license.
// Producer --record snapshot, durable verification by default, --runtime before cleanup.
import {readFile, writeFile, readdir, lstat} from 'node:fs/promises';
import {createHash} from 'node:crypto';
import path from 'node:path';
import assert from 'node:assert/strict';
const root = process.cwd(), directory = 'results/webgl-context';
const runtime = process.argv.includes('--runtime'), record = process.argv.includes('--record');
const hash = value => createHash('sha256').update(value).digest('hex');
const json = async file => JSON.parse(await readFile(file, 'utf8'));
const pin = async file => ({path: file.replaceAll('\\', '/'), sha256: hash(await readFile(file))});
const verify = async entry => assert.equal(hash(await readFile(path.resolve(root, entry.path))), entry.sha256, entry.path);
const oracle = await json(`${directory}/oracle.json`);
const sourceRoot = path.resolve(process.env.SOURCE_ROOT ?? oracle.sourceRoot);
const reference = await json('docs/baseline/reference-manifest.json');
for (const entry of reference) await verify({path: path.join(sourceRoot, entry.path), sha256: entry.sha256});
assert.equal(oracle.inputs.length, 126);
for (const entry of oracle.inputs) {
  assert.ok(reference.some(value => value.path.replaceAll('\\', '/') === entry.path && value.sha256 === entry.sha256), entry.path);
  await verify({path: path.join(sourceRoot, entry.path), sha256: entry.sha256});
}
await verify(oracle.wrapper);
for (const [file, sha256] of [
  ['scripts/webgl-context-oracle.mjs', oracle.generatorSha256],
  ['crates/xterm-web/tests/support/webgl_context.js', oracle.supportSha256],
  ['crates/xterm-web/tests/support/webgl_context_native.js', oracle.nativeSupportSha256],
]) await verify({path: file, sha256});
const initial = await json(`${directory}/browser-controlled-report.json`);
const final = await json(`${directory}/browser-final-report.json`);
for (const report of [initial, final]) {
  assert.equal(report.status, 'passed');
  assert.deepEqual(report.browsers.map(value => value.name), ['chromium', 'firefox', 'webkit']);
  for (const browser of report.browsers) {
    assert.deepEqual(browser.counts, {passed: 4, failed: 0, ignored: 0, filtered: 0});
    assert.deepEqual(browser.errors, []);
    assert.equal(browser.console.filter(value => /too many active/i.test(value.text)).length, 0);
    const traces = prefix => browser.console.filter(value => value.text.startsWith(prefix)).map(value => JSON.parse(value.text.slice(prefix.length)));
    const source = traces('webgl-context-source '), rust = traces('webgl-context-rust ');
    assert.equal(source.length, 29); assert.deepEqual(source, rust);
    assert.equal(new Set(source.map(value => value.name)).size, 29);
  }
  assert.equal(report.artifacts.testSourceSha256, (await pin('crates/xterm-web/tests/wasm_webgl_context.rs')).sha256);
}
assert.equal(initial.artifacts.wasm.sha256, final.artifacts.wasm.sha256);
assert.equal(initial.artifacts.bindings.sha256, final.artifacts.bindings.sha256);
const negative = await json(`${directory}/negative.json`);
assert.equal(negative.expectedFailureObserved, true); assert.equal(negative.restored, true);
assert.equal(negative.originalSha256, negative.restoredSha256);
await verify({path: negative.productionFile, sha256: negative.restoredSha256});
assert.deepEqual(negative.counts, {passed: 3, failed: 1, ignored: 0, filtered: 0});
const rejected = await json(`${directory}/browser-negative-report.json`);
assert.notEqual(rejected.artifacts.wasm.sha256, final.artifacts.wasm.sha256);
const regression = await json(`${directory}/regressions/first-runtime/inputs.json`);
for (const entry of regression.inputs) await verify(entry);
const first = await json(`${directory}/browser-first-runtime-report.json`);
assert.equal(regression.inputs.find(entry => entry.path.endsWith('.rs.txt')).sha256, first.artifacts.testSourceSha256);
const verificationFile = `${directory}/verification.json`;
if (record) {
  const inputs = ['crates/xterm-web/src/webgl/canvas_context.rs', 'crates/xterm-web/src/webgl/context_lifecycle.rs',
    'crates/xterm-web/src/webgl/mod.rs', 'crates/xterm-web/tests/wasm_webgl_context.rs',
    'crates/xterm-web/tests/support/webgl_context.js', 'crates/xterm-web/tests/support/webgl_context_native.js',
    'scripts/webgl-context-oracle.mjs', 'scripts/webgl-context-negative.mjs', 'scripts/webgl-context-audit.mjs',
    'scripts/support/webgl-context-source.ts', 'docs/webgl-context.md', '.pi/swarm/skills/webgl-context-check/SKILL.md'];
  const evidence = [], binaries = new Set();
  async function tree(folder, files) {
    for (const item of await readdir(folder, {withFileTypes: true})) {
      const file = path.join(folder, item.name);
      assert.equal((await lstat(file)).isSymbolicLink(), false, file);
      if (item.isDirectory()) await tree(file, files); else if (item.isFile()) files.push(file);
    }
  }
  await tree(directory, evidence);
  const receipts = evidence.filter(file => path.resolve(file) !== path.resolve(verificationFile));
  for (const file of receipts.filter(value => /browser-.+-report\.json$/.test(value))) {
    const report = await json(file);
    if (!report.artifacts) continue;
    for (const key of ['compiler', 'wasm', 'bindings']) {
      await verify(report.artifacts[key]); binaries.add(path.resolve(root, report.artifacts[key].path));
    }
    const artifacts = [];
    await tree(path.dirname(report.artifacts.bindings.path), artifacts);
    for (const artifact of artifacts) binaries.add(artifact);
  }
  const native = (await readFile(`${directory}/native-webgl-final.log`, 'utf8')).match(/target[\\/]debug[\\/]deps[\\/]webgl-[0-9a-f]+\.exe/);
  assert.ok(native); binaries.add(path.resolve(native[0].replaceAll('\\', '/')));
  const snapshot = {referenceHashes: reference.length, sourceInputs: oracle.inputs.length,
    durable: await Promise.all([...inputs, ...receipts].sort().map(pin)),
    runtime: await Promise.all([...binaries].sort().map(pin)),
    scope: 'Context component only. Controlled29 phases, native GPU tests, initial unresolved burst/GC input retained for306. Full equivalence NOT PROVEN.'};
  await writeFile(verificationFile, JSON.stringify(snapshot, null, 2) + '\n');
}
const snapshot = await json(verificationFile);
for (const entry of snapshot.durable) await verify(entry);
if (runtime) for (const entry of snapshot.runtime) await verify(entry);
console.log(`PASS ${reference.length} original,126source,29tracepairs perengine,final4x3,negative3P1F,${snapshot.durable.length}durable${runtime ? `,${snapshot.runtime.length}runtime` : ''}`);
