// Bundles unchanged source for baseline execution, never for the Rust product.
import fs from 'node:fs/promises';
import path from 'node:path';
import {createRequire} from 'node:module';
import {pathToFileURL} from 'node:url';
import {createHash} from 'node:crypto';
import assert from 'node:assert/strict';
import {withArtifactServer} from './browser-artifact-server.mjs';
const root = process.cwd(), work = path.join(root,'tmp/hint-source');
const require = createRequire(process.env.SOURCE_PACKAGE_JSON ?? path.join(root,'source_repo/package.json'));
const source = path.dirname(require.resolve('./package.json'));
const addon = path.join(source,'addons/addon-ligatures/src/LigaturesAddon.ts');
const parse = require.resolve('opentype.js');
const helper = path.join(root,'crates/xterm-font-addons/tests/support/font_hint_tables.mjs');
const sha = bytes => createHash('sha256').update(bytes).digest('hex');
const fixture = JSON.parse(await fs.readFile('crates/xterm-addons/tests/fixtures/font-hint-tables.json','utf8'));
const base = [...await fs.readFile('crates/xterm-addons/tests/fixtures/ligature-fonts/UbuntuMono-Regular.ttf')];
const report = {node:process.version,scope:'Unchanged source browser baseline only, not Rust execution',browsers:[]};
await fs.mkdir(work,{recursive:true});
try {
  await require('esbuild').build({stdin:{contents:`import {LigaturesAddon} from ${JSON.stringify(addon)};import {parse} from ${JSON.stringify(parse)};import {buildHintFont,hintAddonScenarios} from ${JSON.stringify(helper)};
    export async function run(base,cases){const bytes=new Uint8Array(base);const constructors=cases.map(c=>{try{parse(buildHintFont(bytes,c).buffer);return{status:'ok'}}catch(e){return{status:'error',name:e.name}}});const addon=await hintAddonScenarios(o=>new LigaturesAddon(o),bytes,cases);return{constructors,addon};}`,
    resolveDir:root,sourcefile:'hint-source-entry.mjs'},bundle:true,format:'esm',platform:'browser',outfile:path.join(work,'source.js'),logLevel:'silent'});
  const bundle = await fs.readFile(path.join(work,'source.js'));
  report.bundleSha256 = sha(bundle);
  const playwright = await import(pathToFileURL(process.env.PLAYWRIGHT_MODULE ?? require.resolve('playwright/index.mjs')));
  await withArtifactServer({directory:work,css:'',html:'<!doctype html><meta charset="utf-8"><title>Source font baseline</title>'},async url => {
    for (const name of ['chromium','firefox','webkit']) {
      let browser;
      const record = {name,status:'unrun'}; report.browsers.push(record);
      try {
        browser = await playwright[name].launch({headless:true,timeout:30000});
        record.version = browser.version();
        const page = await browser.newPage(); page.setDefaultTimeout(180000);
        const errors = []; page.on('pageerror',error=>errors.push(String(error)));
        await page.goto(url);
        await page.evaluate(({base,cases})=>{
          globalThis.resultPromise=import('/source.js').then(module=>module.run(base,cases));
          globalThis.resultPromise.then(result=>{globalThis.result=result;},error=>{globalThis.failure=String(error.stack??error);});
        },{base,cases:fixture.cases});
        await page.waitForFunction(()=>globalThis.result!==undefined||globalThis.failure!==undefined);
        const actual = await page.evaluate(()=>({result:globalThis.result,failure:globalThis.failure}));
        assert.equal(actual.failure,undefined); assert.deepEqual(errors,[]);
        assert.deepEqual(actual.result.constructors,fixture.cases.map(c=>c.outcome.status==='ok'?{status:'ok'}:{status:'error',name:c.outcome.name}));
        assert.deepEqual(actual.result.addon,fixture.addon);
        Object.assign(record,{status:'passed',constructors:fixture.cases.length,addonCases:fixture.addon.length});
      } catch(error) {record.status='failed';record.error=String(error.stack??error);throw error;}
      finally {if(browser)await browser.close();}
    }
  });
} finally {
  await fs.mkdir('results/font-hint-tables',{recursive:true});
  await fs.writeFile('results/font-hint-tables/source-browser.json',JSON.stringify(report,null,2)+'\n');
  await fs.rm(work,{recursive:true,force:true});
}
console.log(JSON.stringify(report));
