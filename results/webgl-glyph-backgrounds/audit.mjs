import {readFile, writeFile} from 'node:fs/promises';
import {createHash} from 'node:crypto';
import {execFileSync} from 'node:child_process';
import {fileURLToPath} from 'node:url';
import {resolve} from 'node:path';
const root = fileURLToPath(new URL('../../', import.meta.url));
const output = resolve(root, 'results/webgl-glyph-backgrounds');
const source = process.env.SOURCE_ROOT;
if (!source) throw Error('SOURCE_ROOT must point at the unchanged supplied reference');
const hash = bytes => createHash('sha256').update(bytes).digest('hex');
const json = async path => JSON.parse(await readFile(resolve(root, path), 'utf8'));
const reference = await json('docs/baseline/reference-manifest.json');
for (const pin of reference) {
  if (hash(await readFile(resolve(source, pin.path))) !== pin.sha256) throw Error('Reference changed: ' + pin.path);
}
const inherited = await json('results/webgl-renderer/checkpoint.json');
const inheritedPins = [];
for (const pin of inherited.pins) {
  // This task intentionally moves this type, not the accepted449 algorithm.
  const historical = pin.path === 'crates/xterm-web/src/webgl/renderer_types.rs';
  const bytes = historical ? execFileSync(process.env.GIT ?? 'git', ['show', 'be4e8da1:' + pin.path], {cwd: root, timeout: 30000, maxBuffer: 16 * 1024 * 1024})
    : await readFile(resolve(root, pin.path));
  const crlfDocument = ['docs/webgl-renderer-update.md','.pi/swarm/skills/webgl-renderer-check/SKILL.md'].includes(pin.path);
  const checked = crlfDocument ? Buffer.from(bytes.toString('utf8').replaceAll('\r\n', '\n')) : bytes;
  if (hash(checked) !== pin.sha256) throw Error('Inherited449 pin mismatch: ' + pin.path);
  inheritedPins.push({path: pin.path, verifiedFrom: historical ? 'be4e8da1' : 'working-tree',
    crlfDocument, workingSha256: hash(bytes), sha256: pin.sha256});
}
const fixtures = {};
for (const file of ['webgl-glyphs.json','webgl-glyph-backgrounds.json']) {
  const fixture = await json('crates/xterm-web/tests/fixtures/' + file);
  for (const pin of fixture.inputs) {
    if (hash(await readFile(resolve(source, pin.path))) !== pin.sha256) throw Error('Fixture source changed: ' + pin.path);
  }
  fixtures[file] = {cases: fixture.cases.length, sourceInputs: fixture.inputs.length};
}
const originalFixture = execFileSync(process.env.GIT ?? 'git', ['show', '32bf9544:crates/xterm-web/tests/fixtures/webgl-glyphs.json'], {cwd: root, timeout: 30000, maxBuffer: 16 * 1024 * 1024});
if (hash(originalFixture) !== hash(await readFile(resolve(root, 'crates/xterm-web/tests/fixtures/webgl-glyphs.json')))) {
  throw Error('Original147-case fixture changed');
}
const files = [
  'crates/xterm-web/src/webgl/glyphs.rs','crates/xterm-web/src/webgl/glyph_gpu.rs',
  'crates/xterm-web/src/webgl/renderer_types.rs','crates/xterm-web/tests/webgl_glyph_backgrounds.rs',
  'crates/xterm-web/tests/wasm_webgl_glyphs.rs','scripts/webgl-glyph-oracle.mjs',
  'crates/xterm-web/tests/fixtures/webgl-glyph-backgrounds.json','crates/xterm-web/tests/fixtures/webgl-glyphs.json',
  'docs/webgl-signed-backgrounds.md','.pi/swarm/skills/webgl-glyph-check/SKILL.md',
  ...['native.log','native-default.log','release.log','clippy-native.log','clippy-wasm.log','fmt-all.log','fmt-web.log',
    'wasm-build.log','negative-control.mjs','audit.mjs'].map(file => 'results/webgl-glyph-backgrounds/' + file)
];
for (const file of ['negative-control.json','negative-native.log','negative-browser.json','negative-browser.log',
  'restored-native.log','final-browser.json','final-browser.log']) files.push('results/webgl-glyph-backgrounds/' + file);
const pins = [];
for (const path of files) {
  const bytes = await readFile(resolve(root, path));
  pins.push({path, bytes: bytes.length, sha256: hash(bytes)});
}
const report = {task: 462, base: '32bf9544', dependency449: 'edea56b2->9ce14376->9845d26d->be4e8da1',
  date: new Date().toISOString(), platform: process.platform, node: process.version,
  originalReferenceFiles: reference.length, inheritedPins, fixtures,
  originalFixtureUnchanged: true, pins,
  scope: 'Signed background overlap clipping only. Packed atlas identity/raw browser host/constructor/Addon/Root/full suites remain306. Full equivalence NOT PROVEN.'};
await writeFile(resolve(output, 'checkpoint.json'), JSON.stringify(report, null, 2) + '\n');
console.log('PASS', reference.length, 'original hashes,', inheritedPins.length, 'inherited449 pins,', pins.length, 'new pins');
