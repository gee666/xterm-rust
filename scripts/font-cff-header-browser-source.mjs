// Browser baseline bundles unchanged reference modules only. Never used by the Rust product.
import fs from 'node:fs/promises';
import path from 'node:path';
import {createRequire} from 'node:module';
import {pathToFileURL} from 'node:url';
import {createHash} from 'node:crypto';
import assert from 'node:assert/strict';
import {withArtifactServer} from './browser-artifact-server.mjs';
const root=process.cwd(),work=path.join(root,'tmp/cff-header-source'),require=createRequire(process.env.SOURCE_PACKAGE_JSON??path.join(root,'source_repo/package.json'));
const esbuild=require('esbuild'),source=path.dirname(require.resolve('./package.json')),addon=path.join(source,'addons/addon-ligatures/src/LigaturesAddon.ts'),parse=require.resolve('opentype.js');
const helper=path.join(root,'crates/xterm-font-addons/tests/support/font_cff_header_input.mjs'),scenarios=path.join(root,'crates/xterm-font-addons/tests/support/font_cff_header_addon.mjs');
const sha=x=>createHash('sha256').update(x).digest('hex'),fixtureBytes=await fs.readFile('crates/xterm-addons/tests/fixtures/font-cff-header.json'),fixture=JSON.parse(fixtureBytes);
const addonExpected=JSON.parse(await fs.readFile('crates/xterm-font-addons/tests/fixtures/font-cff-header-addon.json'));
const fonts={};for(const [key,name]of [['otf','FiraCode-Regular.otf'],['ttf','UbuntuMono-Regular.ttf']])fonts[key]=[...await fs.readFile(`crates/xterm-addons/tests/fixtures/ligature-fonts/${name}`)];
const normalize=o=>o.status==='error'?{status:o.status,name:o.name,...(/^(Unsupported CFF|Data error|Font doesn't)/.test(o.message)?{message:o.message}:{})}:o;
const report={node:process.version,status:'running',fixtureSha256:sha(fixtureBytes),sourceSha256:sha(await fs.readFile(parse)),scope:'Unchanged source constructor and addon baseline only, not Rust execution',browsers:[]};
await fs.mkdir(work,{recursive:true});
try {
  await esbuild.build({stdin:{contents:`import {LigaturesAddon} from ${JSON.stringify(addon)};import {parse} from ${JSON.stringify(parse)};import {cffHeaderInput,mutateCffHeader} from ${JSON.stringify(helper)};import {cffHeaderAddonScenarios} from ${JSON.stringify(scenarios)};
    export async function run(fonts,fixture){for(const k of Object.keys(fonts))fonts[k]=new Uint8Array(fonts[k]);const inputs={},constructors=[];
    const normalize=${normalize.toString()};
    for(const c of fixture.cases){const bytes=cffHeaderInput(fonts[c.base??'otf'],c,{...fixture.woffTemplates[c.base??'otf'],...c.encoded}),warnings=[],warn=console.warn,error=console.error,sentinel={object:{},number:42,null:null,undefined:undefined}[c.throws];let outcome;
      try{console.warn=(...args)=>{warnings.push(['warn',...args.map(String)]);if(c.mutation)mutateCffHeader(bytes,c);if(c.throws)throw sentinel;};console.error=(...args)=>warnings.push(['error',...args.map(String)]);parse(bytes.buffer);outcome={status:'ok'};}
      catch(e){outcome=c.throws&&Object.is(e,sentinel)?{status:'warning'}:{status:'error',name:e.name,message:e.message};}finally{console.warn=warn;console.error=error;}
      constructors.push({outcome:normalize(outcome),warnings});
      if(c.data?.length===2&&c.data[0]===3&&c.data[1]===7&&c.remove?.[0]==='hmtx'&&!c.woff)inputs[c.tag==='CFF '?'BadCff':'BadCff2']=bytes;
      if(JSON.stringify(c.header)==='[1,0,4]'&&!c.mutation&&c.tag==='CFF '&&!c.woff)inputs.GoodCff=bytes;
      if(JSON.stringify(c.header)==='[1,0,4]'&&!c.mutation&&c.tag==='CFF2'&&c.woff)inputs.GoodCff2=bytes;
    }return {constructors,addon:await cffHeaderAddonScenarios(o=>new LigaturesAddon(o),inputs)};}`,
    resolveDir:root,sourcefile:'cff-header-source-entry.mjs'},bundle:true,format:'esm',platform:'browser',outfile:path.join(work,'source.js'),logLevel:'silent'});
  report.bundleSha256=sha(await fs.readFile(path.join(work,'source.js')));
  const playwright=await import(pathToFileURL(process.env.PLAYWRIGHT_MODULE??require.resolve('playwright/index.mjs')));
  await withArtifactServer({directory:work,css:'',html:'<!doctype html><meta charset="utf-8"><title>Source CFF header baseline</title>'},async url=>{
    for(const name of ['chromium','firefox','webkit']) {
      let browser;const record={name,status:'unrun'};report.browsers.push(record);
      try {
        browser=await playwright[name].launch({headless:true,timeout:30000});record.version=browser.version();
        const page=await browser.newPage();page.setDefaultTimeout(180000);const errors=[];page.on('pageerror',e=>errors.push(String(e)));await page.goto(url);
        await page.evaluate(({fonts,fixture})=>{import('/source.js').then(m=>m.run(fonts,fixture)).then(result=>{globalThis.result=result;},e=>{globalThis.failure=String(e.stack??e);});},{fonts,fixture});
        await page.waitForFunction(()=>globalThis.result!==undefined||globalThis.failure!==undefined);
        const actual=await page.evaluate(()=>({result:globalThis.result,failure:globalThis.failure}));
        assert.equal(actual.failure,undefined);assert.deepEqual(errors,[]);
        assert.deepEqual(actual.result.constructors,fixture.cases.map(c=>({outcome:normalize(c.outcome),warnings:c.warnings})));
        assert.deepEqual(actual.result.addon,addonExpected);Object.assign(record,{status:'passed',constructors:fixture.cases.length,addonCases:4});
      }catch(e){record.status='failed';record.error=String(e.stack??e);throw e;}finally{if(browser)await browser.close();}
    }
  });report.status='passed';
}catch(e){report.status='failed';report.error=String(e.stack??e);throw e;}
finally{esbuild.stop();await fs.mkdir('results/font-cff-header',{recursive:true});await fs.writeFile('results/font-cff-header/source-browser.json',JSON.stringify(report,null,2)+'\n');await fs.rm(work,{recursive:true,force:true});}
console.log(JSON.stringify(report));
