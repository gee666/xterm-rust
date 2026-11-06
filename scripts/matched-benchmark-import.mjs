// Import only independently verified unchanged production bundles, never deps.
import assert from 'node:assert/strict';
import fs from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';
import { sha256, runtimePins, verify } from './matched-benchmark-adapter.mjs';
import { jsonHash } from './benchmark-contract.mjs';
const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..');
function within(parent, child) {
  const relative = path.relative(parent, child);
  return relative === '' || (!path.isAbsolute(relative) && relative !== '..' && !relative.startsWith(`..${path.sep}`));
}
export function importOriginal({ source, report, artifacts, output }) {
  source = path.resolve(source); artifacts = path.resolve(artifacts); output = path.resolve(output);
  const relative = path.relative(path.join(root, 'tmp'), artifacts);
  assert.ok(relative && relative !== '..' && !relative.startsWith(`..${path.sep}`) && !path.isAbsolute(relative), 'Artifact copy must be a fresh child of owned repository tmp');
  const parent = path.dirname(artifacts);
  assert.ok(!fs.lstatSync(parent).isSymbolicLink() && path.relative(parent, fs.realpathSync(parent)) === '', 'Physical artifact parent required');
  assert.ok(!within(source, output) && !within(artifacts, output), 'Durable output must be outside source and disposable artifacts');
  fs.mkdirSync(path.dirname(output), { recursive: true });
  assert.ok(!fs.existsSync(output) || !fs.lstatSync(output).isSymbolicLink(), 'Physical provenance output required');
  writeFailed(output);
  assert.ok(!fs.existsSync(artifacts), 'Artifact destination already exists');
  const producer = JSON.parse(fs.readFileSync(report, 'utf8'));
  const headless = producer.distributions.find(row => row.id === 'original-headless-core-esm');
  const all = producer.distributions.find(row => row.id === 'original-web-all-addons-esm');
  assert.ok(headless && all, 'Need actual headless and addon production receipts');
  assert.equal(headless.profile, 'release', 'Headless receipt is not production');
  assert.equal(all.profile, 'release', 'Addon receipt is not production');
  assert.equal(headless.build.sourceSha256, all.build.sourceSha256, 'Producer source mismatch');
  for (const command of ['node bin/esbuild.mjs --prod --headless', ...['unicode11', 'unicode-graphemes', 'serialize'].map(name => `node bin/esbuild.mjs --prod --addon=${name}`)]) {
    assert.ok(headless.build.commands.includes(command), `Missing actual production command: ${command}`);
  }
  const baseline = path.join(root, 'docs/baseline/reference-manifest.json');
  const sources = JSON.parse(fs.readFileSync(baseline, 'utf8')).map(pin => {
    const file = path.join(source, pin.path);
    assert.equal(sha256(file), pin.sha256, `Reference source changed: ${pin.path}`);
    return { name: `source_repo/${pin.path}`, path: file, sha256: pin.sha256 };
  });
  sources.push({ name: 'docs/baseline/reference-manifest.json', path: baseline, sha256: sha256(baseline) });
  fs.mkdirSync(artifacts);
  try {
    const pins = {};
    for (const name of ['headless', 'unicode11', 'unicode-graphemes', 'serialize']) {
      const filename = name === 'headless' ? 'xterm-headless.mjs' : `addon-${name}.mjs`;
      const expected = (name === 'headless' ? headless : all).files.find(row => row.path.endsWith(filename));
      assert.ok(expected, `Missing production pin: ${name}`);
      const input = path.join(source, name === 'headless' ? 'headless/lib-headless' : `addons/addon-${name}/lib`, filename);
      assert.ok(fs.lstatSync(input).isFile() && !fs.lstatSync(input).isSymbolicLink(), 'Physical bundle required');
      assert.equal(sha256(input), expected.sha256, `Production bundle changed: ${name}`);
      const destination = path.join(artifacts, filename);
      fs.copyFileSync(input, destination);
      assert.equal(sha256(destination), expected.sha256);
      pins[name] = { path: destination, sha256: expected.sha256 };
    }
    const runtime = runtimePins(root);
    const build = { ...headless.build,
      sourceSha256: jsonHash(sources.map(({ name, sha256 }) => ({ name, sha256 }))),
      artifactSha256: jsonHash(Object.entries(pins).map(([name, pin]) => ({ name, sha256: pin.sha256 }))),
      flags: `${headless.build.flags}; independently verified reference/source and copied production pins; producer-source-sha256=${headless.build.sourceSha256}; adapter-runtime-sha256=${jsonHash(runtime.map(({ name, sha256 }) => ({ name, sha256 })))}` };
    const provenance = { sources, artifacts: pins, runtime, build };
    verify(provenance);
    fs.writeFileSync(output, JSON.stringify(provenance, null, 2));
    return provenance;
  } catch (error) {
    fs.rmSync(artifacts, { recursive: true });
    throw error;
  }
}
function writeFailed(output) {
  fs.writeFileSync(output, JSON.stringify({ status: 'invalidated until import succeeds' }));
}
if (process.argv[1] && path.resolve(process.argv[1]) === fileURLToPath(import.meta.url)) {
  if (process.argv.length !== 6) throw new Error('Usage: matched-benchmark-import.mjs SOURCE_ROOT PRODUCER_SIZES.json FRESH_TMP_ARTIFACTS OUTPUT_PROVENANCE.json');
  const [source, report, artifacts, output] = process.argv.slice(2);
  const imported = importOriginal({ source, report, artifacts, output });
  console.log(`Verified ${imported.sources.length} source and ${Object.keys(imported.artifacts).length} production pins`);
}
