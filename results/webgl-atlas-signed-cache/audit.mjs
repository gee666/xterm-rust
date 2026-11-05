// Read-only source/reference/artifact checks; checkpoint output is durable evidence.
import {readFile, writeFile, readdir} from 'node:fs/promises';
import {createHash} from 'node:crypto';
import {execFileSync} from 'node:child_process';
import {fileURLToPath} from 'node:url';
import {resolve} from 'node:path';
const root = fileURLToPath(new URL('../../', import.meta.url));
const out = resolve(root, 'results/webgl-atlas-signed-cache');
const hash = bytes => createHash('sha256').update(bytes).digest('hex');
const json = async path => JSON.parse(await readFile(resolve(root, path), 'utf8'));
if (!process.env.SOURCE_ROOT) throw Error('SOURCE_ROOT must point to the unchanged supplied reference');
const manifest = await json('docs/baseline/reference-manifest.json');
for (const pin of manifest) {
  if (hash(await readFile(resolve(process.env.SOURCE_ROOT, pin.path))) !== pin.sha256) throw Error('Reference changed: ' + pin.path);
}
const inherited = await json('results/webgl-glyph-backgrounds/checkpoint.json');
const inheritedPins = [];
for (const pin of inherited.pins) {
  // GPU forwarding changes intentionally; all other accepted462 files remain intact.
  const historical = pin.path === 'crates/xterm-web/src/webgl/glyph_gpu.rs';
  const bytes = historical ? execFileSync(process.env.GIT ?? 'git', ['show', '4b82174d:' + pin.path], {
    cwd: root, timeout: 30000, maxBuffer: 16 * 1024 * 1024,
  }) : await readFile(resolve(root, pin.path));
  const crlfDocument = ['docs/webgl-signed-backgrounds.md', '.pi/swarm/skills/webgl-glyph-check/SKILL.md'].includes(pin.path);
  const checked = crlfDocument ? Buffer.from(bytes.toString('utf8').replaceAll('\r\n', '\n')) : bytes;
  if (hash(checked) !== pin.sha256) throw Error('Accepted462 pin mismatch: ' + pin.path);
  inheritedPins.push({path: pin.path, verifiedFrom: historical ? '4b82174d' : 'working-tree',
    crlfDocument, workingSha256: hash(bytes), sha256: pin.sha256});
}
const probe = await json('tmp/webgl-texture-atlas/source.js.json');
for (const pin of probe.inputs) {
  if (hash(await readFile(resolve(process.env.SOURCE_ROOT, pin.path))) !== pin.sha256) throw Error('Probe source input changed: ' + pin.path);
}
if (hash(await readFile(resolve(root, probe.wrapper.path))) !== probe.wrapper.sha256) throw Error('Probe wrapper changed');
if (hash(await readFile(resolve(root, 'tmp/webgl-texture-atlas/source.js'))) !== probe.bundleSha256) throw Error('Probe bundle changed');
const browserArtifacts = [];
for (const name of ['initial-atlas', 'packed-cache-negative', 'packed-gpu-negative', 'final-atlas', 'final-gpu']) {
  const report = await json('results/webgl-atlas-signed-cache/' + name + '.json');
  for (const kind of ['wasm', 'bindings']) {
    const pin = report.artifacts[kind];
    if (hash(await readFile(pin.path)) !== pin.sha256) throw Error('Actual browser artifact changed: ' + name + '/' + kind);
    browserArtifacts.push({report: name, kind, ...pin});
  }
}
const negative = await json('results/webgl-atlas-signed-cache/negative-control.json');
for (const spec of negative.mutations) {
  if (spec.before !== spec.restored || hash(await readFile(resolve(root, spec.path))) !== spec.before) throw Error('Mutated source not restored: ' + spec.path);
}
const files = [
  'crates/xterm-web/src/webgl/texture_atlas.rs', 'crates/xterm-web/src/webgl/glyph_atlas.rs',
  'crates/xterm-web/src/webgl/glyph_gpu.rs', 'crates/xterm-web/tests/wasm_webgl_texture_atlas.rs',
  'crates/xterm-web/tests/support/texture_atlas_gpu.rs', 'crates/xterm-web/tests/reference/webgl-texture-atlas.mjs',
  'docs/webgl-signed-atlas-cache.md',
];
for (const name of (await readdir(out)).sort()) if (name !== 'checkpoint.json') files.push('results/webgl-atlas-signed-cache/' + name);
const pins = [];
for (const path of files) {
  const bytes = await readFile(resolve(root, path));
  pins.push({path, bytes: bytes.length, sha256: hash(bytes)});
}
const report = {task: 471, date: new Date().toISOString(), platform: process.platform, node: process.version,
  base: '32bf9544', privatePrerequisiteHead: '7b4d907a',
  canonicalDependencies: {449: ['edea56b2', '9ce14376', '9845d26d', 'be4e8da1'], 462: ['4b82174d', 'ca53b416', '0009d676', '87fdccbf']},
  originalReferenceFiles: manifest.length, inheritedPins, probe, browserArtifacts, pins,
  scope: 'Signed integer glyphcache keys/GPU-to-atlas forwarding only.306 retains raw browserhost/contrast-cache identities/renderer/Addon/Root/full suites. Full equivalence NOT PROVEN.'};
await writeFile(resolve(out, 'checkpoint.json'), JSON.stringify(report, null, 2) + '\n');
console.log('PASS', manifest.length, 'reference hashes,', inheritedPins.length, 'accepted462 pins,', pins.length,
  'new pins,', browserArtifacts.length, 'actual browser WASM/binding pins');
