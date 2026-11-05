// Use when reviewing465 source identity, original-reference integrity and retained results.
import {readFile, writeFile} from 'node:fs/promises';
import {createHash} from 'node:crypto';
import {resolve, relative, isAbsolute} from 'node:path';
const hash = bytes => createHash('sha256').update(bytes).digest('hex');
const root = process.cwd();
const out = 'results/image-kitty-size-limit';
const original = process.env.ORIGINAL_REFERENCE;
if (!original) throw Error('Set ORIGINAL_REFERENCE to the untouched original source_repo');
const json = async path => JSON.parse(await readFile(path, 'utf8'));
const audit = {status:'checking', originalReference:original, source:[], comparator:[], artifacts:[], originalCount:0};
async function check(path, expected) {
  const actual = hash(await readFile(path));
  if (actual !== expected) throw Error(`Hash mismatch ${path}`);
  return actual;
}
const final = await json(`${out}/final-verification.json`);
for (const entry of final.rustInputs) {
  await check(entry.path, entry.sha256);
  audit.source.push(entry);
}
for (const path of ['crates/xterm-addons/tests/kitty_protocol.rs', 'crates/xterm-addons/tests/support/kitty_runtime.rs', 'crates/xterm-addons/tests/fixtures/kitty-protocol.json', `${out}/negative-control.mjs`]) {
  audit.source.push({path, sha256:hash(await readFile(path))});
}
for (const entry of final.sourceInputs) {
  await check(entry.path, entry.sha256);
  const local = relative(resolve(root, 'source_repo'), entry.path);
  const referenceFile = !isAbsolute(local) && local !== '..' && !local.startsWith('..\\') && !local.startsWith('../');
  if (referenceFile) await check(resolve(original, local), entry.sha256);
  audit.comparator.push({...entry, checkedAgainstOriginal:referenceFile});
}
for (const entry of await json('docs/baseline/reference-manifest.json')) {
  await check(resolve(original, entry.path), entry.sha256);
  audit.originalCount++;
}
const control = await json(`${out}/negative-audit.json`);
if (control.status !== 'passed-negative-control' || !control.restored || control.files.length !== 7) throw Error('Incomplete control');
for (const entry of control.files) await check(entry.file, entry.fixedSha256);
for (const name of ['fixed', 'final']) {
  const report = await json(`${out}/${name}-browser.json`);
  if (report.status !== 'passed' || report.browsers.length !== 3) throw Error(`Incomplete ${name} matrix`);
  for (const browser of report.browsers) {
    if (browser.tests.length !== 48 || browser.counts.passed !== 48 || browser.counts.failed || browser.counts.ignored || browser.counts.filtered || browser.errors.length) throw Error(`Invalid ${name}/${browser.name} counts`);
  }
}
const report = await json(`${out}/final-browser.json`);
for (const key of ['wasm', 'bindings']) {
  const entry = report.artifacts[key];
  await check(entry.path, entry.sha256);
  audit.artifacts.push({kind:key,...entry});
}
const native = 'target/debug/deps/kitty_protocol-d2aed5746fde5476.exe';
audit.artifacts.push({kind:'native-two-test-executable', path:resolve(native), sha256:hash(await readFile(native))});
audit.status = 'passed';
audit.note = 'Artifacts checked before owned cleanup. Their historical paths are not permanent installations.';
await writeFile(`${out}/audit.json`, JSON.stringify(audit, null, 2)+'\n');
console.log(JSON.stringify({status:audit.status,source:audit.source.length,comparator:audit.comparator.length,originalComparator:audit.comparator.filter(x=>x.checkedAgainstOriginal).length,original:audit.originalCount,artifacts:audit.artifacts.length}));
