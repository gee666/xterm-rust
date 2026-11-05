import fs from 'node:fs/promises';
import path from 'node:path';
import assert from 'node:assert/strict';
import {spawnSync} from 'node:child_process';
import {pathToFileURL} from 'node:url';
import {createHash} from 'node:crypto';
import {construct,cases,base} from './font-gdef-oracle.mjs';
import {gdefLoadingScenario} from '../crates/xterm-font-addons/tests/support/font_gdef_scenarios.mjs';
// Reuse the reviewed source-copy/transpile adapter instead of implementing a
// second module-resolution adaptation. It runs its own baseline assertions.
const baseline=spawnSync(process.execPath,['scripts/ligature-addon-oracle.mjs'],{encoding:'utf8',timeout:120000,env:process.env});
if(baseline.error||baseline.status!==0)throw new Error(baseline.error?.message??baseline.stderr);
const {LigaturesAddon}=await import(pathToFileURL(path.resolve('tmp/ligature-addon-oracle/LigaturesAddon.js')));
const names={Warnings:'GDEF two class warnings SFNT',Throw:'GDEF two class warnings SFNT',GetterThrow:'GDEF two class warnings SFNT',Reenter:'GDEF two class warnings SFNT',Mutate:'GDEF class warning skips next pointer SFNT',Reject:'GDEF invalid version SFNT',Compressed:'GDEF large classes warning mutation compressed',LaterMeta:'GDEF warning breaks later META SFNT',LaterFvar:'GDEF warning breaks later fvar SFNT'};
const inputs=Object.fromEntries(Object.entries(names).map(([mode,name])=>[mode,new Uint8Array(construct(cases.find(c=>c.name===name)))]));
const result=await gdefLoadingScenario(options=>new LigaturesAddon(options),inputs,new Uint8Array(base));
for(const [mode,value]of Object.entries(result)){
 assert.deepEqual(value.first.loading,[[0,3]]);assert.deepEqual(value.cached.reads,0);
 assert.deepEqual(value.accepted,{ranges:[],refreshes:['Warnings','Reenter','Mutate','Compressed'].includes(mode)?2:1});
 assert.deepEqual(value.first.ranges,['Warnings','Reenter','Mutate','Compressed'].includes(mode)?[]:[[0,3]]);
}
assert.equal(result.Throw.retainedIdentity,true);assert.equal(result.GetterThrow.retainedIdentity,true);
assert.deepEqual(result.Reenter.first.trace[2],['reenter',[[0,3]]]);
assert.equal(result.Warnings.first.trace.length,4);assert.equal(result.Mutate.first.trace.length,2);assert.equal(result.Compressed.first.trace.length,4);
const content=JSON.stringify(result,null,2)+'\n',file='crates/xterm-font-addons/tests/fixtures/font-gdef-addon.json';
if(process.argv.includes('--generate'))await fs.writeFile(file,content);else assert.equal(await fs.readFile(file,'utf8'),content,'GDEF addon source fixture drift');
const baselineReport=JSON.parse(await fs.readFile('results/font-addons/ligature-addon-oracle.json','utf8'));
const report={node:process.version,scope:'GDEF real font reader/addon boundary, not full Terminal renderer',sourceHashes:baselineReport.sourceHashes,fixtureSha256:createHash('sha256').update(content).digest('hex'),scenarios:Object.keys(result).length};
await fs.writeFile('results/font-gdef/addon-oracle.json',JSON.stringify(report,null,2)+'\n');console.log(JSON.stringify(report));
