// Copyright (c) 2026 The xterm.js authors. All rights reserved. MIT license.
// Verify durable proof by default; --runtime also checks binaries before cleanup.
import {readFile, writeFile, readdir, lstat} from 'node:fs/promises';
import {createHash} from 'node:crypto';
import path from 'node:path';
import assert from 'node:assert/strict';
const root = process.cwd(), directory = 'results/device-pixel-observer';
const recordMode = process.argv.includes('--record'), runtime = process.argv.includes('--runtime');
const hash = bytes => createHash('sha256').update(bytes).digest('hex');
const json = async file => JSON.parse(await readFile(file, 'utf8'));
const pin = async file => ({path: file.replaceAll('\\', '/'), sha256: hash(await readFile(file))});
const verify = async entry => assert.equal(hash(await readFile(path.resolve(root, entry.path))), entry.sha256, entry.path);
const source = 'crates/xterm-web/src/webgl/device_pixel_observer.rs';
const negative = await json(`${directory}/negative.json`);
assert.equal(negative.expectedFailureObserved, true);
assert.equal(negative.restored, true);
assert.equal(negative.originalSha256, negative.restoredSha256);
assert.equal((await pin(source)).sha256, negative.restoredSha256);
assert.deepEqual(negative.counts, {passed: 2, failed: 1, ignored: 0, filtered: 0});
assert.equal((await json(`${directory}/source-traces.json`)).length, 25);
const final = await json(`${directory}/browser-final-report.json`);
assert.equal(final.status, 'passed');
assert.deepEqual(final.browsers.map(b => b.name), ['chromium', 'firefox', 'webkit']);
for (const browser of final.browsers) {
  assert.deepEqual(browser.counts, {passed: 3, failed: 0, ignored: 0, filtered: 0});
  const traces = browser.console.filter(c => c.text.startsWith('native-device-pixel '))
    .map(c => JSON.parse(c.text.slice('native-device-pixel '.length)));
  assert.deepEqual(traces.map(t => t.implementation), ['source', 'rust']);
  assert.deepEqual(traces[0].result, traces[1].result);
  assert.equal(traces[0].result.constructors, 1);
  assert.ok(traces[0].result.contentControl.every(n => n > 0));
  if (browser.name === 'webkit') {
    assert.equal(traces[0].result.observeError, 'TypeError');
    assert.deepEqual(traces[0].result.callbacks, []);
    assert.equal(traces[0].result.disconnects, 1);
  } else {
    assert.equal(traces[0].result.observeError, null);
    assert.deepEqual(traces[0].result.callbacks, traces[0].result.sizes);
    assert.equal(traces[0].result.callbacks.length, 2);
    assert.notEqual(traces[0].result.callbacks[0][0], traces[0].result.callbacks[1][0]);
  }
}
assert.equal(final.artifacts.testSourceSha256,
  (await pin('crates/xterm-web/tests/wasm_device_pixel_observer.rs')).sha256);
const original = path.resolve(process.env.SOURCE_ROOT ?? 'source_repo');
const manifest = await json('docs/baseline/reference-manifest.json');
for (const entry of manifest) await verify({path: path.join(original, entry.path), sha256: entry.sha256});
const verificationPath = `${directory}/verification.json`;
if (recordMode) {
  const inputs = [source, 'crates/xterm-web/src/webgl/mod.rs',
    'crates/xterm-web/tests/wasm_device_pixel_observer.rs',
    'crates/xterm-web/tests/support/device_pixel_observer.js',
    'scripts/device-pixel-observer-oracle.mjs', 'scripts/device-pixel-observer-negative.mjs',
    'scripts/device-pixel-observer-audit.mjs'];
  const evidence = (await readdir(directory)).filter(name => /\.(json|log)$/.test(name)
    && name !== 'verification.json').sort().map(name => `${directory}/${name}`);
  const runtimeFiles = new Set();
  async function tree(directory) {
    for (const item of await readdir(directory, {withFileTypes: true})) {
      const file = path.join(directory, item.name);
      assert.equal((await lstat(file)).isSymbolicLink(), false, file);
      if (item.isDirectory()) await tree(file);
      else if (item.isFile()) runtimeFiles.add(file);
    }
  }
  for (const file of evidence.filter(p => /browser-.+-report\.json$/.test(p))) {
    const report = await json(file);
    for (const key of ['compiler', 'wasm', 'bindings']) {
      const artifact = report.artifacts[key];
      await verify(artifact);
      runtimeFiles.add(path.resolve(root, artifact.path));
    }
    await tree(path.dirname(report.artifacts.bindings.path));
  }
  const nativeLog = await readFile(`${directory}/native.log`, 'utf8');
  const native = nativeLog.match(/target[\\/]debug[\\/]deps[\\/]webgl-[0-9a-f]+\.exe/);
  assert.ok(native, 'native executable must be identified by the actual cargo log');
  runtimeFiles.add(path.resolve(root, native[0].replaceAll('\\', '/')));
  const record = {sourceReferenceHashes: manifest.length,
    durable: await Promise.all([...inputs, ...evidence].map(pin)),
    runtime: await Promise.all([...runtimeFiles].sort().map(pin)),
    scope: 'DevicePixelObserver component only. Full equivalence NOT PROVEN.'};
  await writeFile(verificationPath, JSON.stringify(record, null, 2) + '\n');
}
const record = await json(verificationPath);
for (const entry of record.durable) await verify(entry);
if (runtime) for (const entry of record.runtime) await verify(entry);
console.log(`PASS ${manifest.length} original hashes, ${record.durable.length} durable pins,25 source scenarios, final3x3 and exact2P1F restoration${runtime ? `, ${record.runtime.length} runtime pins` : ''}`);
