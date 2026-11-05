import {readFile,writeFile,mkdir,copyFile} from 'node:fs/promises';
import {spawnSync} from 'node:child_process';
import {createHash} from 'node:crypto';
import {resolve} from 'node:path';
import {runCommand} from '../scripts/quality-process.mjs';
const root=resolve(import.meta.dirname,'..');
const out=resolve(root,'results/mouse-name-errors');
const source=resolve(root,'crates/xterm-web/src/terminal/mouse_api.rs');
const hash=bytes=>createHash('sha256').update(bytes).digest('hex');
const fixed=await readFile(source);
const git=spawnSync('git',['show','735e114a:crates/xterm-web/src/terminal/mouse_api.rs'],{cwd:root,timeout:10000,maxBuffer:1024*1024});
if(git.status!==0)throw Error('cannot read exact old mouse source');
const old=git.stdout;
if(hash(old)===hash(fixed))throw Error('old and fixed source must differ');
const report={oldCommit:'735e114a',oldSourceSha256:hash(old),fixedSourceSha256:hash(fixed),
  status:'failed',commands:[],restored:false};
async function command(exe,args,log){
  const r=await runCommand(exe,args,{cwd:root,env:process.env,log:resolve(out,log),timeoutMs:180000});
  report.commands.push(r);
  if(r.status!=='passed')throw Error('command failed: '+log);
}
try{
  await writeFile(source,old);
  await command('cargo',['test','--locked','-j2','-p','xterm-web','--target','wasm32-unknown-unknown','--test','wasm_terminal_mouse_names','--no-run','--message-format=json'],'negative-build.log');
  const messages=(await readFile(resolve(out,'negative-build.log'),'utf8')).split('\n').filter(x=>x.startsWith('{')).map(JSON.parse);
  const artifact=messages.find(x=>x.reason==='compiler-artifact'&&x.target.name==='wasm_terminal_mouse_names'&&x.profile.test);
  if(!artifact||artifact.fresh)throw Error('baseline must be a real fresh=false compiler build');
  const file=artifact.filenames.find(x=>x.endsWith('.wasm'));
  if(!file)throw Error('missing baseline WASM');
  const compiler=await readFile(file);
  report.compiler={fresh:artifact.fresh,bytes:compiler.length,sha256:hash(compiler),originalPath:file,path:'results/mouse-name-errors/artifacts/old-compiler.wasm'};
  const packageDir=resolve(out,'artifacts/old');
  await mkdir(packageDir,{recursive:true});
  await writeFile(resolve(root,report.compiler.path),compiler);
  await command(process.env.WASM_BINDGEN,[file,'--target','web','--out-dir',packageDir,'--out-name','suite'],'negative-bindgen.log');
  await command(process.execPath,['scripts/mouse-name-control.mjs'],'negative-control.log');
  report.status='passed';
}catch(error){report.error=error.stack;process.exitCode=1;}
finally{
  await writeFile(source,fixed);
  report.restored=hash(await readFile(source))===report.fixedSourceSha256;
  if(!report.restored){process.exitCode=1;report.status='failed';}
  await writeFile(resolve(out,'negative-build.json'),JSON.stringify(report,null,2)+'\n');
}
console.log(JSON.stringify(report,null,2));
