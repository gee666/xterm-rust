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
const root = process.cwd(), out = join(root, 'results/integration543');
const mode = process.argv[2];
assert(['light', 'browser', 'quality', 'canonical', 'components', 'cff2'].includes(mode));
const json = async file => JSON.parse(await readFile(file, 'utf8'));
const save = (name, data) => writeFile(join(out, name), JSON.stringify(data, null, 2) + '\n');
await mkdir(join(root, 'tmp/543-system'), {recursive:true});
process.env.TEMP = process.env.TMP = join(root, 'tmp/543-system');
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
    const browserFiles = ['verification.json', ...['chromium','firefox','webkit'].flatMap(name =>
      [`${name}-source.json`,`${name}-rust.json`,`${name}-terminal.png`]),
      ...['source-suite','rust-suite'].flatMap(name => ['configuration.cjs.txt','playwright.json','playwright.log'].map(file => `${name}/${file}`))];
    for (const file of browserFiles) {
      const destination = join(join(out,'browser'),file);
      await mkdir(join(destination,'..'),{recursive:true});
      await cp(join(root,'results/serialize-browser',file),destination);
    }
    const report = await json(join(out, 'browser/verification.json'));
    assert.equal(report.error, undefined); assert.deepEqual(report.cleanupErrors, []);
    assert.equal(report.browsers.length, 3);
    for (const entry of report.browsers) {
      assert.equal(entry.error, undefined); assert.equal(entry.status, 'passed');
      assert.equal(entry.discovered, 90); assert.equal(entry.executed, 90);
      assert.equal(entry.passed, 90); assert.equal(entry.failed, 0); assert.deepEqual(entry.mismatches, []);
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
if (mode === 'components' || mode === 'cff2') {
  const input = await sourceIdentity(root);
  await save(mode === 'cff2' ? 'cff2-input.json' : 'components-input.json', input);
  if (mode === 'cff2') process.env.BROWSERS = 'firefox';
  const runs = [
    {test:'image_addon_browser', count:74, command:'scripts/image-canvas-check.mjs'},
    {test:'wasm_link_render_recheck', count:2, command:'scripts/browser-host-check.mjs',
      prepare:'scripts/public-link-provider-source.mjs', prepareArgs:['--render-recheck']},
    {test:'wasm_link_mouse_path', count:1, command:'scripts/browser-host-check.mjs',
      prepare:'scripts/public-link-provider-source.mjs', prepareArgs:['--mouse-path']},
    {test:'wasm_webgl_canvas_layer', count:12, command:'scripts/browser-host-check.mjs',
      prepare:'scripts/canvas-layer-oracle.mjs', prelude:join(root,'tmp/canvas-layer/source.js')},
    {test:'wasm_search_addon', count:21, command:'scripts/browser-host-check.mjs',
      prepare:'scripts/search-call-args-source.mjs', env:{SEARCH_CASES:'search-engine-receiver'}},
    {test:'mouse_coordinate_zero', count:1, command:'scripts/browser-host-check.mjs'},
    {test:'wasm_attach_disposal_reentry', count:1, command:'scripts/browser-host-check.mjs',
      prepare:'scripts/attach-disposal-reentry-oracle.mjs'},
    {test:'font_gsub_single_browser', count:2, command:'scripts/browser-host-check.mjs',
      package:'xterm-font-addons', features:'', timeoutMs:900000, env:{WASM_TEST_TIMEOUT_MS:'360000'}},
    {test:'font_cff2_index_browser', count:2, command:'scripts/browser-host-check.mjs',
      package:'xterm-font-addons', features:'', timeoutMs:1200000, env:{WASM_TEST_TIMEOUT_MS:'600000'}}
  ];
  for (const entry of mode === 'cff2' ? runs.filter(entry => entry.test === 'font_cff2_index_browser') : runs) {
    process.env.WASM_TEST = entry.test;
    process.env.WASM_PACKAGE = entry.package ?? 'xterm-web';
    process.env.CARGO_FEATURES = entry.features ?? 'acceptance-hooks';
    process.env.WASM_TEST_TIMEOUT_MS = '60000';
    Object.assign(process.env,entry.env ?? {});
    if (entry.prepare) assert.equal((await run(entry.test+'-source',process.execPath,[entry.prepare,...(entry.prepareArgs??[])])).status,'passed');
    if (entry.prelude) process.env.WASM_PRELUDE = entry.prelude;
    else delete process.env.WASM_PRELUDE;
    assert.equal((await run(entry.test,process.execPath,[entry.command,...(entry.args??[])],entry.timeoutMs ?? 600000)).status,'passed');
    const key = entry.package ? entry.package+'__'+entry.test : entry.test;
    const file = join(root,'results/browser-host',key+'-results.json');
    const report = await json(file);
    await cp(file,join(out,(mode === 'cff2' ? 'cff2-firefox-final' : entry.test)+'-results.json'));
    if (entry.test === 'image_addon_browser') {
      await cp(join(root,'results/image-addon/verification.json'),join(out,'image-addon-verification.json'));
      await cp(join(root,'results/image-addon/source-inputs.json'),join(out,'image-addon-source-inputs.json'));
    }
    if (entry.test === 'wasm_link_mouse_path') await cp(join(root,'results/link-mouse-path/source.json'),join(out,'link-mouse-path-source.json'));
    if (entry.test === 'wasm_webgl_browser_cache') await cp(join(root,'tmp/browser-atlas-cache/source.js.json'),join(out,'atlas-source-inputs.json'));
    assert.equal(report.status,'passed');
    assert.equal(report.browsers.length,mode === 'cff2' ? 1 : 3);
    for (const browser of report.browsers) {
      assert.equal(browser.status,'passed');
      assert.equal(browser.tests.length,entry.count);
      assert.deepEqual(browser.counts,{passed:entry.count,failed:0,ignored:0,filtered:0});
      assert.deepEqual(browser.errors,[]);
    }
  }
  assert.equal((await sourceIdentity(root)).sha256,input.sha256);
  if (mode === 'cff2') {
    const initial = await json(join(out,'cff2-index-initial-60s-failed.json'));
    const final = await json(join(out,'cff2-firefox-final-results.json'));
    assert.equal(final.browsers[0].name,'firefox');
    for (const part of ['compiler','wasm','bindings']) assert.equal(final.artifacts[part].sha256,initial.artifacts[part].sha256);
    assert.equal((await json(join(out,'components-input.json'))).sha256,input.sha256);
    const browsers = ['chromium','webkit'].map(name => {
      const entry = initial.browsers.find(entry => entry.name === name);
      assert.equal(entry.status,'passed'); assert.deepEqual(entry.errors,[]);
      return {name,receipt:'cff2-index-initial-60s-failed.json',counts:entry.counts};
    });
    browsers.push({name:'firefox',receipt:'cff2-firefox-final-results.json',counts:final.browsers[0].counts});
    await save('cff2-union.json',{status:'passed',sameSourceSha256:input.sha256,artifacts:final.artifacts,browsers,
      note:'First Chromium/WebKit passes plus corrected documented600s Firefox execution on identical bytes. Initial60s Firefox deadline failure retained, no passed engine rerun.'});
  }
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
