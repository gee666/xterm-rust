// Reuses bounded commands and the unchanged canonical merge helper.
import assert from 'node:assert/strict';
import {cp, mkdir, readFile, statfs, writeFile} from 'node:fs/promises';
import {freemem} from 'node:os';
import {join} from 'node:path';
import {createHash} from 'node:crypto';
import {execFileSync} from 'node:child_process';
import {runCommand} from '../../scripts/quality-process.mjs';
import {referenceIntegrity} from '../../scripts/upstream-baseline-utils.mjs';
import {sourceIdentity, verifyBuiltPackage} from '../../scripts/e2e/build-identity.mjs';
const root = process.cwd(), out = join(root, 'results/integration520');
const mode = process.argv[2];
assert(['light', 'browser', 'quality', 'canonical'].includes(mode));
const json = async file => JSON.parse(await readFile(file, 'utf8'));
const save = (name, data) => writeFile(join(out, name), JSON.stringify(data, null, 2) + '\n');
await mkdir(join(root, 'tmp/520-system'), {recursive:true});
process.env.TEMP = process.env.TMP = join(root, 'tmp/520-system');
process.env.CARGO_TARGET_DIR = join(root, 'target');
process.env.CARGO_BUILD_JOBS = '2';
async function headroom() {
  const disk = await statfs(root);
  const snapshot = {ramGiB:freemem()/1024**3, diskGiB:Number(disk.bavail)*Number(disk.bsize)/1024**3};
  assert(snapshot.ramGiB >= 8 && snapshot.diskGiB >= 10, JSON.stringify(snapshot));
  return snapshot;
}
const commands = [];
async function run(name, executable, args, timeoutMs = 600000) {
  const before = await headroom();
  const result = await runCommand(executable, args, {cwd:root, env:process.env,
    log:join(out, `${name}.log`), timeoutMs, resourceCheck:headroom});
  commands.push({name, before, ...result});
  await save(`${mode}-commands.json`, commands);
  return result;
}
const cargo = process.env.CARGO ?? 'cargo';
if (mode === 'light') {
  for (const [name, args] of [['fmt-all',['fmt','--all','--','--check']],
    ...['xterm-core','xterm-addons','xterm-web','xterm-wasm-support','xterm-font-addons']
      .map(name => [`fmt-${name}`,['fmt','-p',name,'--','--check']])]) {
    const result = await run(name, cargo, args);
    if (name !== 'fmt-all') assert.equal(result.status, 'passed', name);
  }
  const manifest = await json(join(root, 'docs/baseline/reference-manifest.json'));
  const references = [process.env.SOURCE_ROOT, join(root, 'source_repo')].map(base => ({
    base, ...referenceIntegrity(base, manifest)}));
  references.forEach(entry => assert.equal(entry.mismatches.length, 0));
  await save('references.json', references);
  const audit = await json(join(out, 'manifest.json'));
  const scripts = audit.sourcePins.map(pin => pin.path).filter(file => /\.(mjs|js)$/.test(file));
  for (const file of scripts) execFileSync(process.execPath, ['--check', file], {timeout:30000});
  await save('syntax.json', {status:'passed', scripts});
}
if (mode === 'browser') {
  const pointer = join(root, 'results/demos/build/latest.json'), oldPointer = await readFile(pointer);
  try {
    assert.equal((await run('browser-build', process.execPath, ['scripts/build-demo.mjs','--debug'], 300000)).status, 'passed');
    const input = await sourceIdentity(root), build = await verifyBuiltPackage(root, input);
    await save('browser-build-identity.json', {input, build});
    const wasm = await readFile(join(root, 'browser/pkg/xterm_web_bg.wasm'));
    const sha = createHash('sha256').update(wasm).digest('hex');
    const retained = join(root, 'results/acceptance-artifacts', sha);
    await mkdir(retained, {recursive:true}); await writeFile(join(retained,'artifact.wasm'), wasm);
    const builtPointer = await json(pointer);
    await save('browser-build-pointer.json', builtPointer);
    const result = await run('serialize-browser', process.execPath, ['scripts/serialize-browser-check.mjs','--suite'], 600000);
    await cp(join(root, 'results/serialize-browser'), join(out,'browser'), {recursive:true});
    const report = await json(join(out, 'browser/verification.json'));
    assert.equal(report.error, undefined); assert.deepEqual(report.cleanupErrors, []);
    assert.equal(report.browsers.length, 3);
    for (const entry of report.browsers) {
      assert.equal(entry.error, undefined); assert.equal(entry.status, 'passed');
      assert.equal(entry.discovered, 85); assert.equal(entry.executed, 85);
      assert.equal(entry.passed, 85); assert.equal(entry.failed, 0); assert.deepEqual(entry.mismatches, []);
    }
    assert.equal(report.suites.length, 2);
    for (const suite of report.suites) {
      assert.equal(suite.result.status, 'passed'); assert.equal(suite.counts.passed, 120);
      for (const field of ['failed','skipped','unrun']) assert.equal(suite.counts[field], 0);
    }
    assert.equal(result.status, 'passed');
    assert.equal((await sourceIdentity(root)).sha256, input.sha256);
    assert.deepEqual(await verifyBuiltPackage(root, input), build);
  } finally { await writeFile(pointer, oldPointer); }
}
if (mode === 'quality') {
  for (const [name,args] of [
    ['native-clippy',['clippy','--locked','-j2','--workspace','--all-targets','--all-features','--','-D','warnings']],
    ['wasm-clippy',['clippy','--locked','-j2','--workspace','--lib','--all-features','--target','wasm32-unknown-unknown','--','-D','warnings']],
    ['native-all-features',['test','--locked','-j2','--workspace','--all-targets','--all-features']],
    ['native-doctests',['test','--locked','-j2','--workspace','--doc','--all-features']]
  ]) assert.equal((await run(name,cargo,args)).status,'passed',name);
}
if (mode === 'canonical') {
  assert.equal((await run('canonical-merge', process.execPath, ['scripts/worktree-merge.mjs'], 1200000)).status, 'passed');
}
console.log(JSON.stringify({mode, status:'passed', commands}));
