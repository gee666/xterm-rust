import fs from 'node:fs/promises';
import path from 'node:path';
import assert from 'node:assert/strict';
import {spawnSync} from 'node:child_process';
import {pathToFileURL} from 'node:url';
import {createHash} from 'node:crypto';
import {construct,cases,base} from './font-gpos-oracle.mjs';
import {buildGposFont} from '../crates/xterm-font-addons/tests/support/font_gpos_input.mjs';
import {gposAddonCases} from '../crates/xterm-font-addons/tests/support/font_gpos_scenarios.mjs';
// Reuse the retained unchanged-source module resolution adapter and its guards.
const baseline=spawnSync(process.execPath,['scripts/ligature-addon-oracle.mjs'],{encoding:'utf8',timeout:120000,env:process.env});
if(baseline.error||baseline.status!==0)throw new Error(baseline.error?.message??baseline.stderr);
const {LigaturesAddon}=await import(pathToFileURL(path.resolve('tmp/ligature-addon-oracle/LigaturesAddon.js')));
const fixture=JSON.parse(await fs.readFile('crates/xterm-addons/tests/fixtures/font-gpos.json','utf8'));
for(const item of cases)assert.deepEqual(Buffer.from(buildGposFont(base,fixture,item)),construct(item),item.name);
const addon=await gposAddonCases(options=>new LigaturesAddon(options),i=>new Uint8Array(construct(cases[i])),cases);
for(const [i,record]of addon.entries()){
 const accepted=cases[i].outcome.status==='ok';
 assert.deepEqual(record.loading,[[0,3]]);assert.deepEqual(record.ranges,accepted?[]:[[0,3]],cases[i].name);
 assert.equal(record.refreshes,accepted?1:0,cases[i].name);assert.equal(record.reads,1);assert.equal(record.cacheReads,0);
 assert.deepEqual(record.warnings,cases[i].warnings,cases[i].name);
}
const content=JSON.stringify(addon)+'\n',file='crates/xterm-font-addons/tests/fixtures/font-gpos-addon.json';
if(process.argv.includes('--generate'))await fs.writeFile(file,content);else assert.equal(await fs.readFile(file,'utf8'),content,'GPOS addon fixture drift');
const baselineReport=JSON.parse(await fs.readFile('results/font-addons/ligature-addon-oracle.json','utf8'));
const report={node:process.version,scope:'Real addon GPOS constructor/load/cache component, not full Terminal rendering',sourceHashes:baselineReport.sourceHashes,fixtureSha256:createHash('sha256').update(content).digest('hex'),scenarios:addon.length};
await fs.writeFile('results/font-gpos/addon-oracle.json',JSON.stringify(report,null,2)+'\n');console.log(JSON.stringify(report));
