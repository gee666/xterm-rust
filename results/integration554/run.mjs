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
const root = process.cwd(), out = join(root, 'results/integration554');
const mode = process.argv[2];
assert(['light', 'quality', 'canonical'].includes(mode));
const json = async file => JSON.parse(await readFile(file, 'utf8'));
const save = (name, data) => writeFile(join(out, name), JSON.stringify(data, null, 2) + '\n');
await mkdir(join(root, 'tmp/554-system'), {recursive:true});
process.env.TEMP = process.env.TMP = process.env.TMPDIR = join(root, 'tmp/554-system');
process.env.CARGO_TARGET_DIR = join(root, 'target');
process.env.CARGO_BUILD_JOBS = '2';
async function headroom() {
  const disk = await statfs(root);
  const snapshot = {ramGiB:freemem()/1024**3, diskGiB:Number(disk.bavail)*Number(disk.bsize)/1024**3};
  assert(snapshot.ramGiB >= 8 && snapshot.diskGiB >= 10, JSON.stringify(snapshot));
  return snapshot;
}
const commands = [];
process.env.CARGO = process.env.CARGO ?? 'C:/Users/user/.cargo/bin/cargo.exe';
process.env.RUSTC = process.env.RUSTC ?? 'C:/Users/user/.cargo/bin/rustc.exe';
process.env.WASM_BINDGEN = 'P:/workshop/Windows_Worshop/oira666/xterm-rust/tmp/wasm-tools/wasm-bindgen-0.2.100-x86_64-pc-windows-msvc/wasm-bindgen.exe';
const reference = 'P:/workshop/Windows_Worshop/oira666/xterm-rust/source_repo';
process.env.SOURCE_ROOT = process.env.XTERM_SOURCE_ROOT = process.env.SOURCE_REPO = reference;
process.env.REFERENCE_NODE_MODULES = process.env.NODE_PATH = reference + '/node_modules';
process.env.PLAYWRIGHT_MODULE = reference + '/node_modules/playwright/index.mjs';
process.env.CARGO_FEATURES = 'acceptance-hooks';
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
