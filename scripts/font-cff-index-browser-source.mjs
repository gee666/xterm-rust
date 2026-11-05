// Bundles unchanged reference modules only. No reference source file is edited.
import fs from 'node:fs/promises';
import path from 'node:path';
import {createRequire} from 'node:module';
import {pathToFileURL} from 'node:url';
import {createHash} from 'node:crypto';
import assert from 'node:assert/strict';
import {withArtifactServer} from './browser-artifact-server.mjs';
const root=process.cwd(),work=path.join(root,'tmp/cff-index-source'),require=createRequire(process.env.SOURCE_PACKAGE_JSON??path.join(root,'source_repo/package.json'));
const esbuild=require('esbuild'),source=path.dirname(require.resolve('./package.json')),addon=path.join(source,'addons/addon-ligatures/src/LigaturesAddon.ts'),parse=require.resolve('opentype.js');
const helper=path.join(root,'crates/xterm-font-addons/tests/support/font_cff_index_input.mjs'),scenarios=path.join(root,'crates/xterm-font-addons/tests/support/font_cff_header_addon.mjs');
const sha=x=>createHash('sha256').update(x).digest('hex'),fixtureBytes=await fs.readFile('crates/xterm-addons/tests/fixtures/font-cff-index.json'),fixture=JSON.parse(fixtureBytes);
const expectedAddon=JSON.parse(await fs.readFile('crates/xterm-font-addons/tests/fixtures/font-cff-header-addon.json'));expectedAddon.debug[1]={name:'RangeError'};
const base=[...await fs.readFile('crates/xterm-addons/tests/fixtures/ligature-fonts/FiraCode-Regular.otf')];
const report={node:process.version,status:'running',fixtureSha256:sha(fixtureBytes),sourceSha256:sha(await fs.readFile(parse)),scope:'Unchanged source constructor/addon only. Error kind/lifecycle comparison; raw engine-specific messages retained, Rust Bounds message remains a documented342 gap.',browsers:[]};
await fs.mkdir(work,{recursive:true});
try {
  await esbuild.build({stdin:{contents:`import {LigaturesAddon} from ${JSON.stringify(addon)};import {parse} from ${JSON.stringify(parse)};import {cffIndexInput} from ${JSON.stringify(helper)};import {cffHeaderAddonScenarios} from ${JSON.stringify(scenarios)};
    export async function run(base,fixture){base=new Uint8Array(base);const inputs={},constructors=[];
    for(const c of fixture.cases){const bytes=cffIndexInput(base,c,{...fixture.woffTemplates.otf,...c.encoded}),warnings=[],warn=console.warn,error=console.error;let outcome;
      try{console.warn=(...args)=>warnings.push(['warn',...args.map(String)]);console.error=(...args)=>warnings.push(['error',...args.map(String)]);parse(bytes.buffer);outcome={status:'ok'};}
      catch(e){outcome={status:'error',name:e.name,message:e.message};}finally{console.warn=warn;console.error=error;}
      constructors.push({outcome,warnings});
      if(c.tag==='CFF '&&!c.woff&&c.truncate===4)inputs.BadCff=bytes;
      if(c.tag==='CFF2'&&c.woff&&c.truncate===15353)inputs.BadCff2=bytes;
      if(c.truncate===undefined&&!c.patches&&!c.data){if(c.tag==='CFF '&&!c.woff)inputs.GoodCff=bytes;if(c.tag==='CFF2'&&c.woff)inputs.GoodCff2=bytes;}
    }return {constructors,addon:await cffHeaderAddonScenarios(o=>new LigaturesAddon(o),inputs)};}`,
    resolveDir:root,sourcefile:'cff-index-source-entry.mjs'},bundle:true,format:'esm',platform:'browser',outfile:path.join(work,'source.js'),logLevel:'silent'});
  report.bundleSha256=sha(await fs.readFile(path.join(work,'source.js')));
  const playwright=await import(pathToFileURL(process.env.PLAYWRIGHT_MODULE??require.resolve('playwright/index.mjs')));
  await withArtifactServer({directory:work,css:'',html:'<!doctype html><meta charset="utf-8"><title>Source CFF INDEX baseline</title>'},async url=>{
    for(const name of ['chromium','firefox','webkit']) {
      let browser;const record={name,status:'unrun'};report.browsers.push(record);
      try {
        browser=await playwright[name].launch({headless:true,timeout:30000});record.version=browser.version();
        const page=await browser.newPage();page.setDefaultTimeout(180000);const errors=[];page.on('pageerror',e=>errors.push(String(e)));await page.goto(url);
        await page.evaluate(({base,fixture})=>{import('/source.js').then(m=>m.run(base,fixture)).then(result=>{globalThis.result=result;},e=>{globalThis.failure=String(e.stack??e);});},{base,fixture});
        await page.waitForFunction(()=>globalThis.result!==undefined||globalThis.failure!==undefined);
        const actual=await page.evaluate(()=>({result:globalThis.result,failure:globalThis.failure}));assert.equal(actual.failure,undefined);assert.deepEqual(errors,[]);
        const kind=c=>({status:c.status,...(c.status==='error'?{name:c.name}:{})});
        assert.deepEqual(actual.result.constructors.map(c=>({outcome:kind(c.outcome),warnings:c.warnings})),fixture.cases.map(c=>({outcome:kind(c.outcome),warnings:c.warnings})));
        record.addonRaw=structuredClone(actual.result.addon);const addon=actual.result.addon;delete addon.debug[1].message;assert.deepEqual(addon,expectedAddon);
        record.constructorMessages=Object.fromEntries([...new Set(actual.result.constructors.map(c=>c.outcome.message).filter(Boolean))].map(m=>[m,actual.result.constructors.filter(c=>c.outcome.message===m).length]));
        Object.assign(record,{status:'passed',constructors:fixture.cases.length,addonCases:4});
      }catch(e){record.status='failed';record.error=String(e.stack??e);throw e;}finally{if(browser)await browser.close();}
    }
  });report.status='passed';
}catch(e){report.status='failed';report.error=String(e.stack??e);throw e;}
finally{esbuild.stop();await fs.mkdir('results/font-cff-index',{recursive:true});await fs.writeFile('results/font-cff-index/source-browser.json',JSON.stringify(report,null,2)+'\n');await fs.rm(work,{recursive:true,force:true});}
console.log(JSON.stringify(report));
