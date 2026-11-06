// Copyright (c) 2018-2026 The xterm.js authors. All rights reserved. MIT license.
import {readFile,writeFile,copyFile} from 'node:fs/promises';
import {createHash} from 'node:crypto';
import {spawnSync} from 'node:child_process';
const file='crates/xterm-web/src/webgl/browser_frame_host.rs';
const bytes=await readFile(file),text=bytes.toString();
const needle='self.gpu.glyphs.render(model)';
if(text.split(needle).length!==2)throw Error('Expected one actual GPU draw call');
const mutant=Buffer.from(text.replace(needle,'{ self.gpu.glyphs.clear(); self.gpu.glyphs.render(model) }'));
const hash=b=>createHash('sha256').update(b).digest('hex');
const record={source:file,originalSha256:hash(bytes),mutantSha256:hash(mutant),restored:false};
try {
  await writeFile(file,mutant);
  await writeFile('results/webgl-frame/negative-mutant.rs.txt',mutant);
  const run=spawnSync(process.execPath,['scripts/browser-host-check.mjs'],{env:{...process.env,WASM_TEST:'wasm_webgl_frame',WASM_PRELUDE:'tmp/webgl-frame/source.js',WASM_TEST_TIMEOUT_MS:'180000'},encoding:'utf8',timeout:900000,maxBuffer:32*1024**2});
  record.exitCode=run.status;record.error=run.error?.message??null;
  await writeFile('results/webgl-frame/negative.log',(run.stdout??'')+(run.stderr??''));
  const report=JSON.parse(await readFile('results/browser-host/wasm_webgl_frame-results.json','utf8'));
  await copyFile('results/browser-host/wasm_webgl_frame-results.json','results/webgl-frame/browser-negative.json');
  if(run.status===0||run.error||report.browsers.length!==3||report.browsers.some(b=>b.counts?.failed<1||b.errors.length))
    throw Error('Negative control did not produce ordinary assertion failures in all three engines');
  record.counts=report.browsers.map(b=>({name:b.name,counts:b.counts,errors:b.errors}));
} finally {
  await writeFile(file,bytes);
  record.restored=hash(await readFile(file))===record.originalSha256;
  await writeFile('results/webgl-frame/negative.json',JSON.stringify(record,null,2)+'\n');
  if(!record.restored)throw Error('Source restoration SHA differs');
}
console.log('Actual GPU glyph-clear mutant failed in each engine; original source SHA restored');
