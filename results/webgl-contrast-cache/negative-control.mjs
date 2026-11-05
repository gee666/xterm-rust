// Mutate only this task's real production paths, under an explicit resource grant.
// The exact source bytes are restored even if a build, browser, or assertion fails.
import {readFile, writeFile, copyFile} from 'node:fs/promises';
import {createHash} from 'node:crypto';
import {fileURLToPath} from 'node:url';
import {resolve} from 'node:path';
import {runCommand} from '../../scripts/quality-process.mjs';
const root=fileURLToPath(new URL('../../',import.meta.url));
const out=resolve(root,'results/webgl-contrast-cache');
const hash=bytes=>createHash('sha256').update(bytes).digest('hex');
const nativeOnly=process.argv.includes('--native-only');
const report={date:new Date().toISOString(),nativeOnly,mutations:[],commands:[]};
async function browser(name,browsers,test='wasm_webgl_texture_atlas') {
  const result=await runCommand(process.execPath,['scripts/browser-host-check.mjs'],{
    cwd:root,env:{...process.env,CARGO_BUILD_JOBS:'2',BROWSERS:browsers,WASM_TEST:test},
    log:resolve(out,name+'.log'),timeoutMs:240000});
  report.commands.push({name,...result});
  if(result.error||result.timedOut||result.terminationError||result.signal)throw Error('Abnormal run: '+name);
  const file=resolve(out,name+'.json');
  await copyFile(resolve(root,'results/browser-host',test+'-results.json'),file);
  return {result,receipt:JSON.parse(await readFile(file,'utf8'))};
}
async function persist(){await writeFile(resolve(out,nativeOnly?'native-negative.json':'negative-control.json'),JSON.stringify(report,null,2)+'\n');}
for(const spec of [
  {name:'packed-contrast-keys',path:'crates/xterm-web/src/webgl/atlas_colors.rs',
    from:'cache_bg: colors[0],\n            cache_fg: colors[1],',
    to:'cache_bg: i64::from(attrs.bg),\n            cache_fg: i64::from(attrs.fg),'},
  {name:'old-rasterizer-forwarding',path:'crates/xterm-web/src/webgl/atlas_rasterizer.rs',
    from:'AtlasColors::paint_with_colors(&attrs, [colors[0], colors[1]])',to:'AtlasColors::paint(&attrs)'},
] .filter((_,index)=>!nativeOnly||index===0)) {
  const file=resolve(root,spec.path),original=await readFile(file),text=original.toString('utf8');
  if(text.split(spec.from).length!==2)throw Error('Expected one mutation site: '+spec.name);
  const mutant=text.replace(spec.from,spec.to),entry={...spec,before:hash(original),mutant:hash(mutant)};
  report.mutations.push(entry);
  await writeFile(resolve(out,spec.name+'.rs.txt'),mutant);
  try {
    await writeFile(file,mutant);
    if(nativeOnly) {
      const result=await runCommand(process.env.CARGO??'cargo',['test','--locked','-j2','-p','xterm-web','--test','webgl_atlas_colors'],{
        cwd:root,env:process.env,log:resolve(out,'native-mutant.log'),timeoutMs:180000});
      report.commands.push(result);
      const log=await readFile(resolve(out,'native-mutant.log'),'utf8');
      if(result.exitCode!==101||result.timedOut||result.error||
        !log.includes('1 passed; 1 failed; 0 ignored')||!log.includes('seeded color identity: signed-'))
        throw Error('Unexpected native mutant outcome');
      entry.counts={passed:1,failed:1,ignored:0,filtered:0};
    } else {
      const {result,receipt}=await browser(spec.name,'chromium');
      const counts=receipt.browsers[0]?.counts;
      if(result.exitCode!==1||counts?.passed!==18||counts?.failed!==3||counts?.ignored!==0||counts?.filtered!==0)
        throw Error('Unexpected mutant outcome: '+spec.name+' '+JSON.stringify(counts));
      entry.counts=counts;
      entry.failures=(receipt.browsers[0].output??'').split('\n').filter(line=>/^test .* \.\.\. FAIL$/.test(line));
      const failedNames=entry.failures.join('\n');
      for(const expected of ['signed_full_contrast_cache_colors_and_null_reach_real_atlas_pixels',
        'signed_half_contrast_cache_colors_and_null_reach_real_atlas_pixels',
        'actual_gpu_preserves_signed_contrast_keys_and_source_atlas_pixels'])
        if(!failedNames.includes(expected))throw Error('Missing expected mutant failure: '+expected);
    }
  } finally {
    await writeFile(file,original);
    entry.restored=hash(await readFile(file));
    await persist();
    if(entry.restored!==entry.before)throw Error('Source restore mismatch: '+spec.path);
  }
}
if(!nativeOnly)for(const [name,test,count] of [['final-atlas','wasm_webgl_texture_atlas',21],['final-gpu','wasm_webgl_glyphs',8]]) {
  const {result,receipt}=await browser(name,'chromium,firefox,webkit',test);
  if(result.exitCode!==0||receipt.browsers.length!==3||receipt.browsers.some(b=>
    b.counts?.passed!==count||b.counts?.failed!==0||b.counts?.ignored!==0||b.counts?.filtered!==0))
    throw Error('Restored suite failed: '+name);
}
report.finishedAt=new Date().toISOString();await persist();
console.log(nativeOnly?'Packed contrast-key native mutant1PASS1FAIL, exactrestore':'Two real production mutants each18PASS3FAIL; exactrestore; final21atlas+8GPU PASS each3engines');
