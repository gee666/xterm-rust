import fs from 'node:fs/promises';
import path from 'node:path';
import assert from 'node:assert/strict';
import {spawnSync} from 'node:child_process';
import {pathToFileURL} from 'node:url';
import {createHash} from 'node:crypto';
import {construct,cases,base} from './font-outline-oracle.mjs';
import {outlineLoadingScenario} from '../crates/xterm-font-addons/tests/support/font_outline_scenarios.mjs';
const baseline=spawnSync(process.execPath,['scripts/ligature-addon-oracle.mjs'],{encoding:'utf8',timeout:120000,env:process.env});
if(baseline.error||baseline.status!==0)throw new Error(baseline.error?.message??baseline.stderr);
const {LigaturesAddon}=await import(pathToFileURL(path.resolve('tmp/ligature-addon-oracle/LigaturesAddon.js')));
const names={Reject:'loca EOF undefined/0/0 SFNT',CompressedReject:'loca EOF undefined/0/2 WOFF',LazyOffsets:'loca arbitrary offsets remain lazy SFNT',EmptyGlyf:'glyf empty lazy body WOFF'};
const inputs=Object.fromEntries(Object.entries(names).map(([mode,name])=>[mode,new Uint8Array(construct(cases.find(c=>c.name===name)))]));
const result=await outlineLoadingScenario(options=>new LigaturesAddon(options),inputs,new Uint8Array(base));
await fs.writeFile('results/font-outline/addon-first-probe.json',JSON.stringify(result,null,2)+'\n');
for(const [mode,value]of Object.entries(result)){
 const rejected=mode.endsWith('Reject');
 assert.deepEqual(value.first.loading,[[0,3]]);
 assert.deepEqual(value.first.ranges,rejected?[[0,3]]:[]);
 assert.equal(value.first.fetches,1);assert.equal(value.cached.fetches,1);
 assert.equal(value.first.errors.length,rejected?1:0);assert.equal(value.cached.errors,rejected?1:0);
 assert.deepEqual(value.accepted,{ranges:[],refreshes:rejected?1:2,fetches:1});
}
const content=JSON.stringify(result,null,2)+'\n',file='crates/xterm-font-addons/tests/fixtures/font-outline-addon.json';
if(process.argv.includes('--generate'))await fs.writeFile(file,content);else assert.equal(await fs.readFile(file,'utf8'),content,'Outline addon fixture drift');
const baselineReport=JSON.parse(await fs.readFile('results/font-addons/ligature-addon-oracle.json','utf8'));
const report={node:process.version,scope:'TrueType constructor real addon loading/cache/recovery, not full Terminal',sourceHashes:baselineReport.sourceHashes,fixtureSha256:createHash('sha256').update(content).digest('hex'),scenarios:Object.keys(result).length};
await fs.writeFile('results/font-outline/addon-oracle.json',JSON.stringify(report,null,2)+'\n');console.log(JSON.stringify(report));
