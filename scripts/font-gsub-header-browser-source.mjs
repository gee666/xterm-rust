// Browser baseline bundles unchanged source only. Rust never imports this bundle.
import fs from 'node:fs/promises';
import path from 'node:path';
import {createRequire} from 'node:module';
import {pathToFileURL} from 'node:url';
import {createHash} from 'node:crypto';
import assert from 'node:assert/strict';
import {withArtifactServer} from './browser-artifact-server.mjs';
const corpus=process.env.GSUB_CORPUS??'header';
if(!['header','lookup','type9','null','refs','subtables','single','noncontext'].includes(corpus))throw new Error('Unknown GSUB_CORPUS');
const prefix=`font-gsub-${corpus}`;
const root=process.cwd(),work=path.join(root,`tmp/gsub-${corpus}-source`);
const require=createRequire(path.resolve(process.env.SOURCE_PACKAGE_JSON??'source_repo/package.json'));
const source=path.dirname(require.resolve('./package.json')),esbuild=require('esbuild'),parse=require.resolve('opentype.js');
const hash=bytes=>createHash('sha256').update(bytes).digest('hex');
const fixtureBytes=await fs.readFile(`crates/xterm-addons/tests/fixtures/${prefix}.json`),fixture=JSON.parse(fixtureBytes);
const addon=JSON.parse(await fs.readFile(`crates/xterm-font-addons/tests/fixtures/${prefix}-addon.json`,'utf8'));
const base=[...await fs.readFile('crates/xterm-addons/tests/fixtures/ligature-fonts/UbuntuMono-Regular.ttf')];
const report={node:process.version,status:'running',scope:`Unchanged-source GSUB ${corpus} constructor/addon baseline, not Rust`,fixtureSha256:hash(fixtureBytes),sourceSha256:hash(await fs.readFile(parse)),browsers:[]};
await fs.mkdir(work,{recursive:true});
try{
 await esbuild.build({stdin:{contents:`import {LigaturesAddon} from ${JSON.stringify(path.join(source,'addons/addon-ligatures/src/LigaturesAddon.ts'))};import {parse} from ${JSON.stringify(parse)};import {loadBuffer} from ${JSON.stringify(path.join(source,'addons/addon-ligatures/src/fontLigatures/index.ts'))};import {buildHeaderFont,headerAddonCases} from ${JSON.stringify(path.join(root,'crates/xterm-font-addons/tests/support/font_gsub_header_scenarios.mjs'))};
 export async function run(base,fixture){
  const bytes=new Uint8Array(base),input=i=>buildHeaderFont(bytes,fixture,fixture.cases[i]);
  const constructors=fixture.cases.map((item,i)=>{try{parse(input(i).buffer);return{status:'ok'};}catch(error){return{status:'error',name:error.name,message:error.message};}});
  const loads=fixture.cases[0].loadOutcome?fixture.cases.map((item,i)=>{try{loadBuffer(input(i).buffer);return{status:'ok'};}catch(error){return{status:'error',name:error.name,message:error.message};}}):undefined;
  return{constructors,loads,addon:await headerAddonCases(o=>new LigaturesAddon(o),input,fixture.cases.length)};
 }`,resolveDir:root,sourcefile:'gsub-header-source-entry.mjs'},bundle:true,format:'esm',platform:'browser',outfile:path.join(work,'source.js'),logLevel:'silent'});
 report.bundleSha256=hash(await fs.readFile(path.join(work,'source.js')));
 const playwright=await import(pathToFileURL(process.env.PLAYWRIGHT_MODULE??path.join(path.dirname(require.resolve('playwright')),'index.mjs')));
 await withArtifactServer({directory:work,css:'',html:'<!doctype html><meta charset="utf-8"><title>Source GSUB header baseline</title>'},async url=>{
  for(const name of ['chromium','firefox','webkit']){
   let browser;const record={name,status:'unrun'};report.browsers.push(record);
   try{
    browser=await playwright[name].launch({headless:true,timeout:30000});record.version=browser.version();
    const page=await browser.newPage();page.setDefaultTimeout(180000);const errors=[];page.on('pageerror',error=>errors.push(String(error)));
    await page.goto(url);
    await page.evaluate(({base,fixture})=>{import('/source.js').then(m=>m.run(base,fixture)).then(value=>{globalThis.result=value;},error=>{globalThis.failure=String(error.stack??error);});},{base,fixture});
    await page.waitForFunction(()=>globalThis.result!==undefined||globalThis.failure!==undefined);
    const actual=await page.evaluate(()=>({result:globalThis.result,failure:globalThis.failure}));
    assert.equal(actual.failure,undefined);assert.deepEqual(errors,[]);
    // Native DataView messages vary by engine. Missing required descriptors
    // also have different TypeError prose; assert the exact missing property.
    const normalize=(result,field='outcome',browserMessages=true)=>result.map((o,i)=>{
     if(o.status==='error'&&o.name==='RangeError')return{status:'error',name:'RangeError'};
     if(o.status==='error'&&o.name==='TypeError'){
      if(['type9','null','refs','subtables','noncontext'].includes(corpus)){
       const expected=fixture.cases[i][field].message;
       const property=expected.match(/reading '([^']+)'/)?.[1];
       const receiver={lookupListIndexes:'val.feature',lookupType:'lookup',subtables:'lookups[substitution.lookupListIndex]',substFormat:'table',format:'table'}[property];
       const messages=receiver?{
        chromium:expected,
        firefox:`can't access property "${property}", ${receiver} is undefined`,
        webkit:`undefined is not an object (evaluating '${receiver}.${property}')`
       }:expected.includes('coverage')?{
        chromium:"Cannot read properties of undefined (reading 'coverage')",
        firefox:'can\'t access property "coverage", table is undefined',
        webkit:"undefined is not an object (evaluating 'table.coverage')"
       }:{
        chromium:'Cannot convert undefined or null to object',
        firefox:"can't convert undefined to object",
        webkit:"undefined is not an object (evaluating 'Object.keys(description)')"
       };
       assert.equal(o.message,messages[browserMessages?name:'chromium']);
       return{status:'error',name:'TypeError',message:expected};
      }
      assert.ok(fixture.cases[i].remove?.length&&o.message.includes('compression'));
      return{status:'error',name:'TypeError',missing:fixture.cases[i].remove[0],property:'compression'};
     }
     return o;
    });
    assert.deepEqual(normalize(actual.result.constructors),normalize(fixture.cases.map(c=>c.outcome),'outcome',false));
    if(['type9','null','refs','subtables','single','noncontext'].includes(corpus))assert.deepEqual(normalize(actual.result.loads,'loadOutcome'),normalize(fixture.cases.map(c=>c.loadOutcome),'loadOutcome',false));
    assert.deepEqual(actual.result.addon,addon);
    Object.assign(record,{status:'passed',constructors:fixture.cases.length,...(['type9','null','refs','subtables','single','noncontext'].includes(corpus)?{loads:fixture.cases.length,loadErrors:[...new Set(actual.result.loads.filter(o=>o.status==='error').map(o=>o.name+': '+o.message))]}:{}),addonCases:addon.length,errorMessages:[...new Set(actual.result.constructors.filter(o=>o.status==='error').map(o=>o.name+': '+o.message))]});
   }catch(error){record.status='failed';record.error=String(error.stack??error);throw error;}
   finally{if(browser)await browser.close();}
  }
 });report.status='passed';
}catch(error){report.status='failed';report.error=String(error.stack??error);throw error;}
finally{esbuild.stop();await fs.writeFile(`results/${prefix}/source-browser.json`,JSON.stringify(report,null,2)+'\n');await fs.rm(work,{recursive:true,force:true});}
console.log(JSON.stringify(report));
