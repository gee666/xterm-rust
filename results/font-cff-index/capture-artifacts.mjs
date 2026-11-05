import fs from 'node:fs/promises';
import path from 'node:path';
import {createHash} from 'node:crypto';
const sha=bytes=>createHash('sha256').update(bytes).digest('hex'),root=process.cwd();
const report=JSON.parse(await fs.readFile('results/font-cff-index/rust-browser.json'));
if(report.status!=='passed'||report.browsers.length!==3||report.browsers.some(b=>b.status!=='passed'||b.counts.passed!==2||b.counts.failed!==0))throw new Error('Incomplete final browser run');
const work=path.dirname(report.artifacts.wasm.path),run=path.dirname(work),pins={};
for(const [name,entry]of Object.entries(report.artifacts))if(entry&&typeof entry==='object'&&entry.path) {
 const bytes=await fs.readFile(path.resolve(entry.path));if(sha(bytes)!==entry.sha256)throw new Error(`Artifact changed ${name}`);
 pins[name]={path:entry.path,bytes:bytes.length,sha256:entry.sha256,preserved:name==='compiler'||name==='css'};
}
for(const name of ['commands.json','font_cff_index_browser-build.log','font_cff_index_browser-bindgen-version.log','font_cff_index_browser-bindgen.log',...['chromium','firefox','webkit'].map(b=>`font_cff_index_browser-${b}.log`)])await fs.copyFile(path.join(run,name),`results/font-cff-index/final-${name}`);
for(const file of await fs.readdir('target/debug/deps'))if(/^(font_cff_index-|font_cff_header-|xterm_addons-).*\.exe$/.test(file)) {
 const name=path.join('target/debug/deps',file),bytes=await fs.readFile(name);pins[name]={path:path.resolve(name),bytes:bytes.length,sha256:sha(bytes),preserved:false};
}
await fs.writeFile('results/font-cff-index/final-artifacts.json',JSON.stringify({root,pins,note:'Compiler WASM retained under acceptance-artifacts; generated binding/converted WASM/native EXEs are owned temporary outputs, inspected during review then removed. Recreate with recorded commands, not release distributions.'},null,2)+'\n');
console.log(JSON.stringify({pins:Object.keys(pins).length,compiler:report.artifacts.compiler.path}));
