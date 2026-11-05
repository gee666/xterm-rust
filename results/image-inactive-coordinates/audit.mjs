// Use when auditing the retained inactive-coordinate matrices before cleanup.
import {readFile, writeFile} from 'node:fs/promises';
import {createHash} from 'node:crypto';
import {spawnSync} from 'node:child_process';
import {resolve} from 'node:path';
const out = 'results/image-inactive-coordinates';
const hash = bytes => createHash('sha256').update(bytes).digest('hex');
const json = async name => JSON.parse(await readFile(`${out}/${name}.json`, 'utf8'));
const reports = {};
const audit = {status: 'running', inputPins: 0, artifactPins: 0, referenceFiles: 0, compilations: []};
for (const phase of ['initial', 'negative', 'final']) {
  const report = reports[phase] = await json(`${phase}-browser`);
  const verification = await json(`${phase}-verification`);
  const positive = phase !== 'negative';
  if (report.status !== (positive ? 'passed' : 'failed')) throw Error(`Unexpected ${phase} status`);
  for (const browser of report.browsers) {
    if (browser.tests.length !== 56 || browser.counts.passed !== (positive ? 56 : 54) ||
        browser.counts.failed !== (positive ? 0 : 2) || browser.counts.ignored || browser.counts.filtered || browser.errors.length) {
      throw Error(`Unexpected ${phase}/${browser.name} counters`);
    }
  }
  if (report.browsers.map(b => b.name).join(',') !== (positive ? 'chromium,firefox,webkit' : 'chromium')) {
    throw Error(`Unexpected ${phase} engines`);
  }
  for (const entry of [...verification.rustInputs, ...verification.sourceInputs]) {
    // Negative product files were restored. Check their base bytes separately.
    if (phase === 'negative' && ['crates/xterm-web/src/image/addon/bindings.rs', 'crates/xterm-web/src/image/addon/options.rs'].includes(entry.path)) continue;
    if (hash(await readFile(resolve(entry.path))) !== entry.sha256) throw Error(`Changed comparator/product input: ${phase}/${entry.path}`);
    audit.inputPins++;
  }
  for (const name of ['wasm', 'bindings']) {
    const pin = report.artifacts[name];
    if (hash(await readFile(pin.path)) !== pin.sha256) throw Error(`Changed ${phase}/${name} artifact`);
    audit.artifactPins++;
  }
  const messages = (await readFile(`${out}/${phase}-build.log`, 'utf8')).split(/\r?\n/)
    .filter(line => line.startsWith('{')).map(line => JSON.parse(line));
  const test = messages.find(message => message.reason === 'compiler-artifact' && message.target.name === 'image_addon_browser' && message.profile.test);
  if (!test || test.fresh !== false) throw Error(`No fresh ${phase} test compilation`);
  audit.compilations.push({phase, fresh: test.fresh, filenames: test.filenames});
}
if (reports.negative.artifacts.wasm.sha256 === reports.final.artifacts.wasm.sha256) throw Error('Negative and final WASM are identical');
const restored = await json('negative-audit');
if (restored.status !== 'passed-negative-control' || !restored.restored) throw Error('Negative restoration not proven');
for (const item of restored.files) {
  const baseline = spawnSync('git', ['show', `${restored.base}:${item.file}`], {timeout: 30_000, maxBuffer: 1024 * 1024});
  if (baseline.status !== 0 || baseline.error || hash(baseline.stdout) !== item.baselineSha256) throw Error(`Wrong baseline ${item.file}`);
  if (hash(await readFile(item.file)) !== item.fixedSha256 || item.restoredSha256 !== item.fixedSha256) throw Error(`Wrong restoration ${item.file}`);
}
const compiler = reports.final.artifacts.compiler;
if (hash(await readFile(compiler.originalPath)) !== compiler.sha256) throw Error('Final compiler artifact changed');
audit.artifactPins++;
const manifest = JSON.parse(await readFile('docs/baseline/reference-manifest.json', 'utf8'));
for (const root of ['source_repo', process.env.ORIGINAL_REFERENCE]) {
  if (!root) throw Error('Set ORIGINAL_REFERENCE to the preserved original source_repo');
  for (const item of manifest) {
    if (hash(await readFile(resolve(root, item.path))) !== item.sha256) throw Error(`Reference changed ${root}/${item.path}`);
    audit.referenceFiles++;
  }
}
audit.status = 'passed';
await writeFile(`${out}/audit.json`, JSON.stringify(audit, null, 2) + '\n');
console.log(JSON.stringify(audit));
