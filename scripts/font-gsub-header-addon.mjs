import fs from 'node:fs/promises';
import path from 'node:path';
import assert from 'node:assert/strict';
import {spawnSync} from 'node:child_process';
import {pathToFileURL} from 'node:url';
import {createHash} from 'node:crypto';
import {construct,cases,base} from './font-gsub-header-oracle.mjs';
import {buildHeaderFont,headerAddonCases} from '../crates/xterm-font-addons/tests/support/font_gsub_header_scenarios.mjs';
const baseline=spawnSync(process.execPath,['scripts/ligature-addon-oracle.mjs'],{encoding:'utf8',timeout:120000,env:process.env});
if(baseline.error||baseline.status!==0)throw new Error(baseline.error?.message??baseline.stderr);
const {LigaturesAddon}=await import(pathToFileURL(path.resolve('tmp/ligature-addon-oracle/LigaturesAddon.js')));
const fixture=JSON.parse(await fs.readFile('crates/xterm-addons/tests/fixtures/font-gsub-header.json','utf8'));
for(const item of cases)assert.deepEqual(Buffer.from(buildHeaderFont(base,fixture,item)),construct(item),item.name);
const actual=await headerAddonCases(o=>new LigaturesAddon(o),i=>new Uint8Array(construct(cases[i])),cases.length);
for(const [i,record]of actual.entries()){
 const accepted=cases[i].outcome.status==='ok';
 assert.deepEqual(record,{loading:[[0,3]],ranges:accepted?[]:[[0,3]],reads:1,cacheReads:0,refreshes:accepted?1:0},cases[i].name);
}
const content=JSON.stringify(actual)+'\n',file='crates/xterm-font-addons/tests/fixtures/font-gsub-header-addon.json';
if(process.argv.includes('--generate'))await fs.writeFile(file,content);else assert.equal(await fs.readFile(file,'utf8'),content,'GSUB header addon drift');
const baselineReport=JSON.parse(await fs.readFile('results/font-addons/ligature-addon-oracle.json','utf8'));
const report={node:process.version,scope:'Actual source addon constructor/load/cache component, not terminal rendering',sourceHashes:baselineReport.sourceHashes,cases:actual.length,fixtureSha256:createHash('sha256').update(content).digest('hex')};
await fs.writeFile('results/font-gsub-header/addon-oracle.json',JSON.stringify(report,null,2)+'\n');console.log(JSON.stringify(report));
