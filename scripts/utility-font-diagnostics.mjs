// Copyright (c) 2026 xterm-rust contributors. Licensed under MIT.
// Observe host font metrics without changing source tests or terminal options.
import { mkdir, writeFile } from 'node:fs/promises';
import { resolve } from 'node:path';
import { pathToFileURL } from 'node:url';
const playwright = await import(pathToFileURL(resolve('source_repo/node_modules/playwright/index.mjs')));
const report = { node:process.version, platform:process.platform, purpose:'source FitAddon environment diagnosis, not Rust parity', browsers:[] };
for (const name of ['chromium','firefox','webkit']) {
  let browser;
  const entry={name,status:'unrun'};report.browsers.push(entry);
  try {
    browser=await playwright[name].launch({headless:true,timeout:30000});entry.version=browser.version();
    const page=await browser.newPage();page.setDefaultTimeout(30000);
    await page.setContent('<!doctype html><meta charset="utf-8"><div id="parent" style="width:800px;height:450px"></div>');
    entry.metrics=await page.evaluate(()=>{
      const canvas=document.createElement('canvas');const context=canvas.getContext('2d');
      const fonts=[];
      for(const family of ['monospace','"Courier New"','Consolas','"Lucida Console"']) {
        context.font=`15px ${family}`;const metric=context.measureText('W');
        const span=document.createElement('span');span.style.cssText=`font:15px ${family};white-space:pre`;span.textContent='W'.repeat(32);document.body.appendChild(span);
        const width=metric.width;
        fonts.push({family,font:context.font,width,domWidth:span.getBoundingClientRect().width/32,ascent:metric.fontBoundingBoxAscent,descent:metric.fontBoundingBoxDescent,fit800:Math.max(2,Math.floor((800-14)/width)),fit1008:Math.max(2,Math.floor((1008-14)/width))});span.remove();
      }
      return {userAgent:navigator.userAgent,platform:navigator.platform,devicePixelRatio,fonts};
    });entry.status='observed';
  } catch(error){entry.status='failed';entry.error=error.stack??error.message;}
  finally{if(browser)await browser.close();}
}
await mkdir('results/utility-addons',{recursive:true});
await writeFile('results/utility-addons/font-diagnostics.json',JSON.stringify(report,null,2)+'\n');
console.log(JSON.stringify(report,null,2));
if(report.browsers.some(browser=>browser.status!=='observed'))process.exitCode=1;
