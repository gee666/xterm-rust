// Browser baseline bundles unchanged source only. Never linked into Rust.
import fs from 'node:fs/promises';
import path from 'node:path';
import {createRequire} from 'node:module';
import {pathToFileURL} from 'node:url';
import {createHash} from 'node:crypto';
import assert from 'node:assert/strict';
import {withArtifactServer} from './browser-artifact-server.mjs';
const root=process.cwd(),work=path.join(root,'tmp/gpos-source');
const require=createRequire(path.resolve(process.env.SOURCE_PACKAGE_JSON??'source_repo/package.json'));
const source=path.dirname(require.resolve('./package.json'));
const esbuild=require('esbuild'),parse=require.resolve('opentype.js');
const hash=bytes=>createHash('sha256').update(bytes).digest('hex');
const fixtureBytes=await fs.readFile('crates/xterm-addons/tests/fixtures/font-gpos.json'),fixture=JSON.parse(fixtureBytes);
const expectedAddon=JSON.parse(await fs.readFile('crates/xterm-font-addons/tests/fixtures/font-gpos-addon.json','utf8'));
const base=[...await fs.readFile('crates/xterm-addons/tests/fixtures/ligature-fonts/UbuntuMono-Regular.ttf')];
const report={node:process.version,status:'running',scope:'Unchanged-source GPOS browser constructor/addon baseline, not Rust',fixtureSha256:hash(fixtureBytes),sourceSha256:hash(await fs.readFile(parse)),browsers:[]};
await fs.mkdir(work,{recursive:true});
try{
 await esbuild.build({stdin:{contents:`import {LigaturesAddon} from ${JSON.stringify(path.join(source,'addons/addon-ligatures/src/LigaturesAddon.ts'))};import {parse} from ${JSON.stringify(parse)};import {buildGposFont} from ${JSON.stringify(path.join(root,'crates/xterm-font-addons/tests/support/font_gpos_input.mjs'))};import {gposAddonCases} from ${JSON.stringify(path.join(root,'crates/xterm-font-addons/tests/support/font_gpos_scenarios.mjs'))};
 export async function run(base,fixture){
  const bytes=new Uint8Array(base),input=i=>buildGposFont(bytes,fixture,fixture.cases[i]);
  const constructors=[],saved=Object.getOwnPropertyDescriptor(console,'warn');
  try{for(const [index,item]of fixture.cases.entries()){
   const font=input(index),warnings=[];
   Object.defineProperty(console,'warn',{configurable:true,value(message){warnings.push(message);if(item.throws)throw undefined;
    if(warnings.length===1&&item.warningMutation){const m=item.warningMutation,view=new DataView(font.buffer),woff=view.getUint32(0)===0x774f4646;for(let i=0;i<view.getUint16(woff?12:4);i++){const at=(woff?44:12)+i*(woff?20:16);if(String.fromCharCode(...font.subarray(at,at+4))===m.tag){view.setUint16(view.getUint32(at+(woff?4:8))+m.at,m.value);break;}}}
   }});
   let outcome;try{parse(font.buffer);outcome={status:'ok'}}catch(error){outcome=item.throws&&warnings.length?{status:'warning'}:{status:'error',name:error.name};}
   constructors.push({outcome,warnings});
  }}finally{if(saved)Object.defineProperty(console,'warn',saved);else delete console.warn;}
  return{constructors,addon:await gposAddonCases(o=>new LigaturesAddon(o),input,fixture.cases)};
 }`,resolveDir:root,sourcefile:'gpos-source-entry.mjs'},bundle:true,format:'esm',platform:'browser',outfile:path.join(work,'source.js'),logLevel:'silent'});
 report.bundleSha256=hash(await fs.readFile(path.join(work,'source.js')));
 const playwright=await import(pathToFileURL(process.env.PLAYWRIGHT_MODULE??require.resolve('playwright/index.mjs')));
 await withArtifactServer({directory:work,css:'',html:'<!doctype html><meta charset="utf-8"><title>Source GPOS baseline</title>'},async url=>{
  for(const name of ['chromium','firefox','webkit']){
   let browser;const record={name,status:'unrun'};report.browsers.push(record);
   try{
    browser=await playwright[name].launch({headless:true,timeout:30000});record.version=browser.version();
    const page=await browser.newPage();page.setDefaultTimeout(180000);const errors=[];page.on('pageerror',error=>errors.push(String(error)));
    await page.goto(url);
    await page.evaluate(({base,fixture})=>{globalThis.done=import('/source.js').then(module=>module.run(base,fixture));globalThis.done.then(value=>{globalThis.result=value;},error=>{globalThis.failure=String(error.stack??error);});},{base,fixture});
    await page.waitForFunction(()=>globalThis.result!==undefined||globalThis.failure!==undefined);
    const actual=await page.evaluate(()=>({result:globalThis.result,failure:globalThis.failure}));
    assert.equal(actual.failure,undefined);assert.deepEqual(errors,[]);
    const expected=fixture.cases.map(item=>({outcome:item.outcome.status==='error'?{status:'error',name:item.outcome.name}:item.outcome,warnings:item.warnings}));
    assert.deepEqual(actual.result.constructors,expected);assert.deepEqual(actual.result.addon,expectedAddon);
    Object.assign(record,{status:'passed',constructors:fixture.cases.length,addonCases:expectedAddon.length});
   }catch(error){record.status='failed';record.error=String(error.stack??error);throw error;}
   finally{if(browser)await browser.close();}
  }
 });report.status='passed';
}catch(error){report.status='failed';report.error=String(error.stack??error);throw error;}
finally{esbuild.stop();await fs.mkdir('results/font-gpos',{recursive:true});await fs.writeFile('results/font-gpos/source-browser.json',JSON.stringify(report,null,2)+'\n');await fs.rm(work,{recursive:true,force:true});}
console.log(JSON.stringify(report));
