// Use when reviewing474 source identity, original integrity and retained results.
// Adapted from the accepted465 audit; no compiler or browser process is launched.
import {readFile, writeFile} from 'node:fs/promises';
import {createHash} from 'node:crypto';
import {resolve, relative, isAbsolute} from 'node:path';
const hash = bytes => createHash('sha256').update(bytes).digest('hex');
const root = process.cwd(), out = 'results/image-iip-size-limit';
const original = process.env.ORIGINAL_REFERENCE;
if (!original) throw Error('Set ORIGINAL_REFERENCE to the untouched original source_repo');
const json = async path => JSON.parse(await readFile(path, 'utf8'));
const audit = {status:'checking', originalReference:original, source:[], comparator:[], artifacts:[], originalCount:0};
async function check(path, expected) {
  const actual = hash(await readFile(path));
  if (actual !== expected) throw Error(`Hash mismatch ${path}`);
}
const seen = new Set();
for (const name of ['final', 'iip']) {
  const verification = await json(`${out}/${name}-verification.json`);
  for (const entry of verification.rustInputs) {
    await check(entry.path, entry.sha256);
    if (!seen.has(entry.path)) audit.source.push(entry);
    seen.add(entry.path);
  }
  for (const entry of verification.sourceInputs) {
    if (entry.path.replaceAll('\\', '/').endsWith('/tmp/image-canvas/image-comparators.ts')) {
      // The shared runner overwrites this generated entry between suites.
      // Reconstruct it from the pinned generator and verify the retained hash.
      const names = [...(name === 'final' ? ['ImageAddon'] : []), 'ImageRenderer', 'IIPHandler', 'ImageStorage', 'IIPImageStorage'];
      const exports = names.map(value => `export { ${value} } from ${JSON.stringify(resolve(root, `source_repo/addons/addon-image/src/${value}.ts`))};`);
      exports.push(`export { Terminal } from ${JSON.stringify(resolve(root, 'source_repo/src/browser/public/Terminal.ts'))};`);
      const generated = exports.join('\n');
      if (hash(generated) !== entry.sha256) throw Error(`Generated comparator differs ${name}`);
      await writeFile(`${out}/reconstructed-${name}-comparator.ts`, generated);
      audit.comparator.push({...entry, reconstructed:true, checkedAgainstOriginal:false, suite:name});
      continue;
    }
    await check(entry.path, entry.sha256);
    const local = relative(resolve(root, 'source_repo'), entry.path);
    const referenceFile = !isAbsolute(local) && local !== '..' && !local.startsWith('..\\') && !local.startsWith('../');
    if (referenceFile) await check(resolve(original, local), entry.sha256);
    if (!seen.has(entry.path)) audit.comparator.push({...entry, checkedAgainstOriginal:referenceFile});
    seen.add(entry.path);
  }
}
for (const path of ['crates/xterm-addons/tests/image_iip.rs', 'crates/xterm-addons/tests/image_iip_raw_limit.rs', 'crates/xterm-addons/tests/iip_payload.rs', 'crates/xterm-addons/tests/iip_warning.rs', 'crates/xterm-addons/examples/image_iip_trace.rs', 'scripts/image-iip-oracle.mjs', `${out}/negative-control.mjs`]) {
  audit.source.push({path, sha256:hash(await readFile(path))});
}
for (const entry of await json('docs/baseline/reference-manifest.json')) {
  await check(resolve(original, entry.path), entry.sha256);
  audit.originalCount++;
}
const control = await json(`${out}/negative-audit.json`);
if (control.status !== 'passed-negative-control' || !control.restored || control.files.length !== 8) throw Error('Incomplete control');
for (const entry of control.files) await check(entry.file, entry.fixedSha256);
for (const [name, count] of [['initial',54], ['final',54], ['iip',12]]) {
  const report = await json(`${out}/${name}-browser.json`);
  if (report.status !== 'passed' || report.browsers.length !== 3) throw Error(`Incomplete ${name} matrix`);
  for (const browser of report.browsers) {
    if (browser.tests.length !== count || browser.counts.passed !== count || browser.counts.failed || browser.counts.ignored || browser.counts.filtered || browser.errors.length) throw Error(`Invalid ${name}/${browser.name} counts`);
  }
}
for (const name of ['initial','negative','final','iip']) {
  const report = await json(`${out}/${name}-browser.json`);
  for (const key of ['wasm','bindings']) {
    const entry = report.artifacts[key];
    await check(entry.path, entry.sha256);
    audit.artifacts.push({kind:`${name}-${key}`,...entry});
  }
}
for (const native of ['target/debug/deps/image_iip-c4dcba0a2627e085.exe', 'target/debug/deps/image_iip_raw_limit-5d16742fcf11ff27.exe', 'target/debug/deps/iip_payload-f9bbefda8e9f8383.exe', 'target/debug/deps/iip_warning-33e8726afba80b86.exe', 'target/debug/examples/image_iip_trace.exe']) {
  audit.artifacts.push({kind:'native-executable', path:resolve(native), sha256:hash(await readFile(native))});
}
const oracle = await json(`${out}/native-oracle.json`);
if (oracle.status !== 'passed' || oracle.comparisons !== 9925 || oracle.mismatches !== 0) throw Error('Invalid oracle counts');
for (const entry of oracle.implementation) await check(entry.path, entry.sha256);
await check('results/image-iip/operations.ndjson', oracle.operationsSha256);
audit.status = 'passed';
audit.note = 'Artifacts checked before owned cleanup. Historical temporary paths are not permanent installations. Invalid decoder payload paths remain unproven.';
await writeFile(`${out}/audit.json`, JSON.stringify(audit, null, 2)+'\n');
console.log(JSON.stringify({status:audit.status,source:audit.source.length,comparator:audit.comparator.length,originalComparator:audit.comparator.filter(x=>x.checkedAgainstOriginal).length,original:audit.originalCount,artifacts:audit.artifacts.length}));
