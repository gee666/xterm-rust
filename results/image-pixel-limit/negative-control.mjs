// Use when proving the new assertions fail the exact original production code.
import {readFile,writeFile,lstat,realpath,mkdir,copyFile} from 'node:fs/promises';
import {execFileSync} from 'node:child_process';
import {resolve,relative} from 'node:path';
import {createHash} from 'node:crypto';
import {runCommand} from '../../scripts/quality-process.mjs';
const root=resolve('.'),out=resolve('results/image-pixel-limit');
if(!root.replaceAll('\\','/').endsWith('/tmp/worktrees/BryanImagePixels'))
 throw Error('Control is limited to the owned worktree');
const git=(...args)=>execFileSync(process.env.GIT??'git',args,{cwd:root,timeout:30000});
const base='0442e3578a7f57d35c2ae6ff20d7186820983065';
const files=git('diff','--name-only','--diff-filter=M',base,'--','crates/xterm-addons/src/image','crates/xterm-web/src/image').toString().trim().split(/\r?\n/).filter(Boolean);
const saved=[];
const hash=b=>createHash('sha256').update(b).digest('hex');
for(const file of files){
 const target=resolve(file);
 if(relative(root,target).startsWith('..')||!(await lstat(target)).isFile()||resolve(await realpath(target))!==target)
  throw Error('Unowned or linked production input');
 const fixed=await readFile(target),original=git('show',base+':'+file);
 saved.push({file,fixed,original,fixedSHA:hash(fixed),baseSHA:hash(original)});
}
await mkdir(out,{recursive:true});
await mkdir('tmp/pixel-temp',{recursive:true});
const env={...process.env,CARGO_BUILD_JOBS:'2',CARGO_TARGET_DIR:resolve('target'),WASM_TEST:'image_addon_browser',
 BROWSERS:'chromium',TEMP:resolve('tmp/pixel-temp'),TMP:resolve('tmp/pixel-temp'),TMPDIR:resolve('tmp/pixel-temp')};
let record,report;
try{
 for(const item of saved)await writeFile(item.file,item.original);
 record=await runCommand('node',['scripts/image-canvas-check.mjs'],{cwd:root,env,log:resolve(out,'control.log'),timeoutMs:600000});
 await copyFile('results/browser-host/image_addon_browser-results.json',resolve(out,'control-browser.json'));
 await copyFile('results/image-addon/verification.json',resolve(out,'control-verification.json'));
 await copyFile('tmp/image-canvas/image-comparators.ts',resolve(out,'control-comparators.ts'));
 await copyFile('tmp/image-canvas/source-renderer.js',resolve(out,'control-source-renderer.js'));
 report=JSON.parse(await readFile(resolve(out,'control-browser.json'),'utf8'));
 if(record.status==='passed'||report.browsers.length!==1)throw Error('Control unexpectedly passed or lacked browser evidence');
 const counts=report.browsers[0].counts;
 if(counts?.passed!==64||counts?.failed!==12||counts?.ignored!==0||counts?.filtered!==0)
  throw Error('Control must retain all64 inherited passes and fail exactly12 new functions: '+JSON.stringify(counts));
}finally{
 for(const item of saved)await writeFile(item.file,item.fixed);
 const pins=[];
 for(const item of saved){
  const after=hash(await readFile(item.file));
  pins.push({file:item.file,fixedSHA:item.fixedSHA,baseSHA:item.baseSHA,afterSHA:after,restored:after===item.fixedSHA});
 }
 await writeFile(resolve(out,'control-restore.json'),JSON.stringify({base,record,pins},null,2)+'\n');
 if(pins.some(p=>!p.restored))throw Error('Control failed to restore exact fixed bytes');
}
