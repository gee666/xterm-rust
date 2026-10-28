// Copyright (c) 2019 The xterm.js authors. MIT license.
// Compile unchanged reference code. esbuild erases type-only imports, unlike
// Node's transform-types loader for this source's non-type import declaration.
import assert from 'node:assert/strict';
import { mkdirSync, writeFileSync, readFileSync } from 'node:fs';
import { createHash } from 'node:crypto';
import { resolve } from 'node:path';
import { pathToFileURL } from 'node:url';
const root = resolve(import.meta.dirname,'..');
const { build } = await import(pathToFileURL(resolve(root,'source_repo/node_modules/esbuild/lib/main.js')));
const temporary = resolve(root,'tmp/public-addon-oracle');
mkdirSync(temporary,{recursive:true});
await build({entryPoints:[resolve(root,'source_repo/src/common/public/AddonManager.ts')],bundle:true,platform:'node',format:'esm',outfile:resolve(temporary,'addon.mjs')});
const { AddonManager } = await import(pathToFileURL(resolve(temporary,'addon.mjs')));
const { default: Mocha } = await import(pathToFileURL(resolve(root,'source_repo/node_modules/mocha/index.js')));
await build({entryPoints:[resolve(root,'source_repo/src/common/public/AddonManager.test.ts')],bundle:true,platform:'node',format:'cjs',outfile:resolve(temporary,'upstream.cjs')});
const mocha = new Mocha({reporter:'spec'});
mocha.addFile(resolve(temporary,'upstream.cjs'));
const cases = [];
const failures = await new Promise(done => {
  const runner = mocha.run(done);
  runner.on('pass',test=>cases.push({title:test.fullTitle(),status:'passed'}));
  runner.on('fail',(test,error)=>cases.push({title:test.fullTitle(),status:'failed',error:error.message}));
});
assert.equal(failures,0);

const observations = [];
let checks = 0;
const equal = (actual, expected) => { assert.deepEqual(actual, expected); checks++; };
const terminal = { cols: 5 };
const manager = new AddonManager();
const log = [];
const addon = label => ({ activate: t => { equal(t, terminal); log.push(`activate:${label}`); }, dispose: () => log.push(`dispose:${label}`) });
const a = addon('a'), b = addon('b'), c = addon('c');
manager.loadAddon(terminal,a);
manager.loadAddon(terminal,b);
manager.loadAddon(terminal,c);
equal(manager._addons.length,3);
b.dispose(); b.dispose();
manager.dispose(); manager.dispose();
equal(log,['activate:a','activate:b','activate:c','dispose:b','dispose:c','dispose:a']);
equal(manager._addons.length,0);
observations.push({ case:'direct-and-reverse-disposal', log });

const failed = new AddonManager();
const failureLog = [];
for (const [label, throws] of [['a',false],['b',true],['c',false]]) {
  failed.loadAddon(terminal,{ activate() {}, dispose() { failureLog.push(label); if (throws) throw new Error('cleanup'); } });
}
assert.throws(() => failed.dispose(), /cleanup/); checks++;
equal(failed._addons.length,2);
failed.dispose(); failed.dispose();
equal(failed._addons.length,1);
equal(failureLog,['c','b','a']);
observations.push({ case:'cleanup-failure-retains-disposed-record', log:failureLog, remaining:failed._addons.length });

const activation = new AddonManager();
let cleaned = 0;
assert.throws(() => activation.loadAddon(terminal,{ activate() { throw new Error('activate'); }, dispose() { cleaned++; } }),/activate/); checks++;
equal(activation._addons.length,1);
activation.dispose();
equal(cleaned,1);
observations.push({ case:'activation-failure-retains-registration', cleaned });

const recursive = new AddonManager();
let recursiveCalls = 0;
const self = { activate() {}, dispose() { recursiveCalls++; self.dispose(); } };
recursive.loadAddon(terminal,self);
recursive.dispose();
equal(recursiveCalls,1);
equal(recursive._addons.length,0);
observations.push({ case:'recursive-self-disposal', calls:recursiveCalls });

const reentrant = new AddonManager();
reentrant.loadAddon(terminal,{ activate() {}, dispose() { reentrant.loadAddon(terminal,{activate() {},dispose() {}}); } });
reentrant.dispose();
equal(reentrant._addons.length,1);
reentrant.dispose();
equal(reentrant._addons.length,0);
observations.push({ case:'load-during-disposal', remaining:reentrant._addons.length });

const nested = new AddonManager();
const nestedLog = [];
for (const label of ['a','b']) nested.loadAddon(terminal,{activate() {},dispose() {nestedLog.push(label);}});
nested.loadAddon(terminal,{activate() {},dispose() {nestedLog.push('c');nested.dispose();}});
assert.throws(()=>nested.dispose(),TypeError); checks++;
equal(nestedLog,['c','b','a']);
equal(nested._addons.length,1);
nested.dispose();
equal(nestedLog,['c','b','a']);
observations.push({case:'nested-manager-invalidates-index',log:nestedLog,remaining:nested._addons.length});

const lower = new AddonManager();
const first = {activate() {},dispose() {}};
lower.loadAddon(terminal,first);
lower.loadAddon(terminal,{activate() {},dispose() {first.dispose();}});
lower.dispose();
equal(lower._addons.length,1);
lower.dispose();
equal(lower._addons.length,1);
observations.push({case:'remove-lower-captured-splice-index',remaining:lower._addons.length});

const report = { source:'source_repo/src/common/public/AddonManager.ts', node:process.version, status:'passed', checks, observations,
  upstream:{discovered:cases.length,executed:cases.length,passed:cases.filter(c=>c.status==='passed').length,failed:failures,skipped:0,unrun:0,cases},
  hashes:Object.fromEntries(['AddonManager.ts','AddonManager.test.ts'].map(name=>[name,createHash('sha256').update(readFileSync(resolve(root,'source_repo/src/common/public',name))).digest('hex')])),
  scope:'The two original AddonManager cases and directed source observations. Does not execute headless/browser suites or prove terminal/browser equivalence.' };
mkdirSync(new URL('../results/public-adapters/',import.meta.url),{recursive:true});
writeFileSync(new URL('../results/public-adapters/addon-oracle.json',import.meta.url),`${JSON.stringify(report,null,2)}\n`);
console.log(JSON.stringify(report));
