import {spawnSync} from 'node:child_process';
import {readFile,writeFile,mkdir,statfs} from 'node:fs/promises';
import {createHash} from 'node:crypto';
import {freemem} from 'node:os';
import {resolve} from 'node:path';
const root=process.cwd(), out=resolve(root,'results/search-highlight-bounds');
await mkdir(out,{recursive:true});await mkdir(resolve(root,'tmp/env'),{recursive:true});
const env={...process.env,TEMP:resolve(root,'tmp/env'),TMP:resolve(root,'tmp/env'),TMPDIR:resolve(root,'tmp/env'),CARGO_TARGET_DIR:resolve(root,'target'),CARGO_BUILD_JOBS:'2'};
const hash=b=>createHash('sha256').update(b).digest('hex');
const commands=[];
async function run(args,name,expected=0){
 const disk=await statfs(root,{bigint:true});if(freemem()<8*1024**3||disk.bavail*disk.bsize<10n*1024n**3n)throw Error('headroom below agreed8GiB/10GiB');
 const start=Date.now(),r=spawnSync('cargo',args,{cwd:root,env,encoding:'utf8',timeout:180000,maxBuffer:32*1024*1024});
 await writeFile(resolve(out,name+'.log'),r.stdout+'\n'+r.stderr);commands.push({args,name,status:r.status,error:r.error?.message,ms:Date.now()-start});
 await writeFile(resolve(out,'commands.json'),JSON.stringify(commands,null,2)+'\n');
 if(r.error||r.status!==expected)throw Error(`${name} unexpected exit${r.status}: ${r.error??''}`);
}
const find='crates/xterm-addons/src/search/find.rs';const fixed=await readFile(find);
const old=spawnSync('git',['show','224d1b339a43eddf899c2944d9dad734d8c44234:'+find],{cwd:root,encoding:null,timeout:30000,maxBuffer:1024**2});if(old.status!==0)throw Error('git old find');
try{
 await writeFile(find,old.stdout);
 await run(['test','--locked','-j2','-p','xterm-web','--test','search_addon','highlight_bounds::'],'native-old',101);
}finally{await writeFile(find,fixed);if(hash(await readFile(find))!==hash(fixed))throw Error('source restore mismatch');await writeFile(resolve(out,'native-restore.json'),JSON.stringify({path:find,oldSha256:hash(old.stdout),fixedSha256:hash(fixed),restored:true},null,2)+'\n');}
await run(['test','--locked','-j2','-p','xterm-web','--test','search_addon'],'native-final');
for(const test of ['search_cache','search_differential','search_integration','search_ring']) await run(['test','--locked','-j2','-p','xterm-core','--test',test],test);
await run(['clippy','--locked','-j2','-p','xterm-web','--test','search_addon','--','-D','warnings'],'clippy-native-web');
await run(['clippy','--locked','-j2','-p','xterm-core','--test','search_integration','--','-D','warnings'],'clippy-native-core');
await run(['check','--locked','-j2','--target','wasm32-unknown-unknown','-p','xterm-web','--lib'],'check-wasm');
console.log('Native proof complete');
