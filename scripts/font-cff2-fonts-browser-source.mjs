// Unchanged full constructors and actual source addon. Resource hazards stay Node-only.
import fs from 'node:fs/promises';
import path from 'node:path';
import {createRequire} from 'node:module';
import {pathToFileURL} from 'node:url';
import {createHash} from 'node:crypto';
import assert from 'node:assert/strict';
import {withArtifactServer} from './browser-artifact-server.mjs';
const root=process.cwd(),work=path.join(root,'tmp/cff2-fonts-source'),require=createRequire(process.env.SOURCE_PACKAGE_JSON??path.join(root,'source_repo/package.json'));
const esbuild=require('esbuild'),source=path.dirname(require.resolve('./package.json')),addon=path.join(source,'addons/addon-ligatures/src/LigaturesAddon.ts'),parse=require.resolve('opentype.js');
const helper=path.join(root,'crates/xterm-font-addons/tests/support/font_cff_header_input.mjs'),scenarios=path.join(root,'crates/xterm-font-addons/tests/support/font_cff_header_addon.mjs');
const sha=b=>createHash('sha256').update(b).digest('hex'),fixtureBytes=await fs.readFile('crates/xterm-addons/tests/fixtures/font-cff2-fonts.json'),fixture=JSON.parse(fixtureBytes);
const excluded=fixture.cases.filter(c=>c.resourceFailure).map(c=>({name:c.name,reason:'bounded source resource failure'}));
fixture.cases=fixture.cases.filter(c=>!excluded.some(e=>e.name===c.name));
const expected=JSON.parse(await fs.readFile('crates/xterm-font-addons/tests/fixtures/font-cff-header-addon.json'));
const base=[...await fs.readFile('crates/xterm-addons/tests/fixtures/ligature-fonts/FiraCode-Regular.otf')];
const report={node:process.version,status:'running',fixtureSha256:sha(fixtureBytes),sourceSha256:sha(await fs.readFile(parse)),excluded,scope:'Complete eager font/private/Subrs constructors and actual addon lifecycle. Resource failures excluded explicitly, not passed. Later342/equivalence NOT PROVEN.',browsers:[]};
await fs.mkdir(work,{recursive:true});
try {
  await esbuild.build({stdin:{contents:`import {LigaturesAddon} from ${JSON.stringify(addon)};import {parse} from ${JSON.stringify(parse)};import {cffHeaderInput} from ${JSON.stringify(helper)};import {cffHeaderAddonScenarios} from ${JSON.stringify(scenarios)};
    export async function run(base,fixture){base=new Uint8Array(base);const inputs={},constructors=[];
    for(const c of fixture.cases){const bytes=cffHeaderInput(base,c,{...fixture.woffTemplates.otf,...c.encoded}),warnings=[],warn=console.warn,error=console.error;let outcome;
      try{console.warn=(...a)=>warnings.push(['warn',...a.map(String)]);console.error=(...a)=>warnings.push(['error',...a.map(String)]);parse(bytes.buffer);outcome={status:'ok'};}
      catch(e){outcome={status:'error',name:e.name,message:e.message};}finally{console.warn=warn;console.error=error;}
      constructors.push({outcome,warnings});
      const bad=c.addonBad===true;
      if(bad&&c.tag==='CFF '&&!c.woff)inputs.BadCff=bytes;
      if(bad&&c.tag==='CFF2'&&c.woff)inputs.BadCff2=bytes;
      if(!c.data){if(c.tag==='CFF '&&!c.woff)inputs.GoodCff=bytes;if(c.tag==='CFF2'&&c.woff)inputs.GoodCff2=bytes;}
    }if(Object.keys(inputs).length!==4)throw Error('Missing addon input recipe');
    return {constructors,addon:await cffHeaderAddonScenarios(o=>new LigaturesAddon(o),inputs)};}`,
    resolveDir:root,sourcefile:'cff2-fonts-source-entry.mjs'},bundle:true,format:'esm',platform:'browser',outfile:path.join(work,'source.js'),logLevel:'silent'});
  report.bundleSha256=sha(await fs.readFile(path.join(work,'source.js')));
  const playwright=await import(pathToFileURL(process.env.PLAYWRIGHT_MODULE??require.resolve('playwright/index.mjs')));
  await withArtifactServer({directory:work,css:'',html:'<!doctype html><meta charset="utf-8"><title>Source CFF2 font/private DICT</title>'},async url=>{
    for(const name of ['chromium','firefox','webkit']) {
      let browser;const record={name,status:'unrun'};report.browsers.push(record);
      try {
        browser=await playwright[name].launch({headless:true,timeout:30000});record.version=browser.version();
        const page=await browser.newPage();page.setDefaultTimeout(600000);const errors=[];page.on('pageerror',e=>errors.push(String(e)));await page.goto(url);
        await page.evaluate(({base,fixture})=>{import('/source.js').then(m=>m.run(base,fixture)).then(result=>{globalThis.result=result;},e=>{globalThis.failure=String(e.stack??e);});},{base,fixture});
        await page.waitForFunction(()=>globalThis.result!==undefined||globalThis.failure!==undefined);
        const actual=await page.evaluate(()=>({result:globalThis.result,failure:globalThis.failure}));assert.equal(actual.failure,undefined);assert.deepEqual(errors,[]);
        for(const [i,c]of fixture.cases.entries()) {
          const raw=actual.result.constructors[i];assert.deepEqual(raw.warnings,c.warnings);
          assert.equal(raw.outcome.status,c.outcome.status);assert.equal(raw.outcome.name,c.outcome.name);
          if(c.outcome.message?.startsWith('Invalid b0')||c.outcome.message?.startsWith('Object ')||c.outcome.message?.startsWith('This is a CFF2 font'))assert.equal(raw.outcome.message,c.outcome.message);
        }
        record.constructorsRaw=actual.result.constructors;
        record.addonRaw=actual.result.addon;
        const badCases=fixture.cases.map((c,i)=>({c,i})).filter(({c})=>c.addonBad===true);
        assert.equal(badCases.length,4);
        const bounds=actual.result.constructors[badCases[0].i].outcome;
        assert.deepEqual(bounds,{status:'error',name:'Error',message:'Object [object Object] already has key 0'});
        for(const {i}of badCases)assert.deepEqual(actual.result.constructors[i].outcome,bounds);
        record.addonConstructorCases=badCases.map(({c})=>c.name);
        record.addonConstructorOutcome=bounds;
        const engineExpected=structuredClone(expected);engineExpected.debug[1]={name:bounds.name,message:bounds.message};
        assert.deepEqual(record.addonRaw,engineExpected);
        record.constructorMessages=Object.fromEntries([...new Set(actual.result.constructors.map(c=>c.outcome.message).filter(Boolean))].map(m=>[m,actual.result.constructors.filter(c=>c.outcome.message===m).length]));
        Object.assign(record,{status:'passed',constructors:fixture.cases.length,addonCases:4});
      }catch(e){record.status='failed';record.error=String(e.stack??e);throw e;}finally{if(browser)await browser.close();}
    }
  });report.status='passed';
}catch(e){report.status='failed';report.error=String(e.stack??e);throw e;}
finally{esbuild.stop();await fs.mkdir('results/font-cff2-fonts',{recursive:true});await fs.writeFile('results/font-cff2-fonts/source-browser.json',JSON.stringify(report,null,2)+'\n');await fs.rm(work,{recursive:true,force:true});}
console.log(JSON.stringify(report));
