// Trusted Playwright input against the actual Rust/WASM scrollbar fixture.
import {readFile,writeFile,mkdir} from 'node:fs/promises';
import {resolve,dirname} from 'node:path';
import {pathToFileURL} from 'node:url';
import assert from 'node:assert/strict';
import {withArtifactServer} from './browser-artifact-server.mjs';

process.env.WASM_TEST='wasm_scrollbars';
await import('./browser-host-check.mjs');
const root=resolve(import.meta.dirname,'..');
const component=JSON.parse(await readFile(resolve(root,'results/browser-host/wasm_scrollbars-results.json'),'utf8'));
const work=dirname(component.artifacts.bindings.path);
const css=await readFile(resolve(root,component.artifacts.css.path));
const results=resolve(root,'results/scrollbars');await mkdir(results,{recursive:true});
const playwright=await import(pathToFileURL(process.env.PLAYWRIGHT_MODULE??resolve(root,'source_repo/node_modules/playwright/index.mjs')));
const report={kind:'trusted-rust-scrollbar-drag',browsers:[]};
try{
 await withArtifactServer({directory:work,css,html:'<!doctype html><meta charset="utf-8"><link rel="stylesheet" href="/xterm.css"><body></body>'},async url=>{
  for(const name of ['chromium','firefox','webkit']){
    const entry={name,status:'failed',orientations:[]};report.browsers.push(entry);let browser;
    try{
      browser=await playwright[name].launch({headless:true,timeout:30000});entry.version=browser.version();
      const page=await browser.newPage();page.setDefaultTimeout(10000);
      await page.goto(url);
      await page.evaluate(async()=>{const module=await import('/suite.js');await module.default();window.scrollbarModule=module;});
      for(const horizontal of [false,true]){
        await page.evaluate(horizontal=>{
          window.fixture=new window.scrollbarModule.TrustedScrollbarFixture(horizontal);
          window.fixture.slider.id='drag-slider';window.pointer=null;
          window.fixture.slider.addEventListener('pointerdown',event=>{window.pointer={id:event.pointerId,trusted:event.isTrusted};});
        },horizontal);
        try{
          await page.waitForFunction(()=>window.fixture.element.classList.contains('xterm-visible'));
          const box=await page.locator('#drag-slider').boundingBox();assert.ok(box,'slider has no layout box');
          const x=box.x+Math.min(5,box.width/2),y=box.y+Math.min(5,box.height/2);
          await page.mouse.move(x,y);await page.mouse.down();
          const down=await page.evaluate(()=>({starts:window.fixture.starts,trusted:window.pointer?.trusted,captured:window.fixture.slider.hasPointerCapture(window.pointer.id)}));
          assert.equal(down.trusted,true,'input was not a trusted browser event');assert.equal(down.starts,1);assert.equal(down.captured,true,'setPointerCapture did not activate');
          // Both coordinates leave the thin scrollbar, staying below the source
          // 140px orthogonal Windows snap-back threshold.
          await page.mouse.move(x+40,y+40);await page.mouse.up();
          const after=await page.evaluate(()=>({position:window.fixture.position,starts:window.fixture.starts,ends:window.fixture.ends,active:window.fixture.slider.classList.contains('xterm-active'),captured:window.fixture.slider.hasPointerCapture(window.pointer.id)}));
          assert.ok(after.position>0,'outside-node drag did not move Rust scroll state');assert.equal(after.starts,1);assert.equal(after.ends,1);assert.equal(after.active,false);assert.equal(after.captured,false);
          await page.evaluate(()=>window.fixture.dispose());
          const next=await page.locator('#drag-slider').boundingBox();assert.ok(next);
          await page.mouse.move(next.x+Math.min(5,next.width/2),next.y+Math.min(5,next.height/2));await page.mouse.down();await page.mouse.up();
          assert.equal(await page.evaluate(()=>window.fixture.starts),1,'disposed scrollbar retained its pointer listener');
          entry.orientations.push({axis:horizontal?'horizontal':'vertical',status:'passed',position:after.position});
        }finally{
          await page.mouse.up();await page.evaluate(()=>{if(window.fixture){window.fixture.dispose();window.fixture.free();window.fixture=null;}});
        }
      }
      entry.status='passed';console.log(`${name} ${entry.version}: trusted horizontal/vertical drag passed`);
    }catch(error){entry.error=error.stack??error.message;console.error(`${name}: ${error.message}`);}
    finally{if(browser)await browser.close();}
  }

 });
}finally{
 await writeFile(resolve(results,'trusted-drag.json'),JSON.stringify(report,null,2)+'\n');
}
if(report.browsers.some(browser=>browser.status!=='passed'))process.exitCode=1;
