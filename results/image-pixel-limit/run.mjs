// Use when rerunning task539's bounded native, lint and browser checks.
import {mkdir,writeFile,copyFile,statfs} from 'node:fs/promises';
import {freemem} from 'node:os';
import {resolve} from 'node:path';
import {runCommand} from '../../scripts/quality-process.mjs';
const root=process.cwd(),out=resolve('results/image-pixel-limit'),mode=process.argv[2];
await mkdir(out,{recursive:true});await mkdir('tmp/pixel-temp',{recursive:true});
const env={...process.env,CARGO_BUILD_JOBS:'2',CARGO_TARGET_DIR:resolve('target'),
 TEMP:resolve('tmp/pixel-temp'),TMP:resolve('tmp/pixel-temp'),TMPDIR:resolve('tmp/pixel-temp')};
const records=[];
async function resource(){
 const disk=await statfs(root);
 if(freemem()<8*1024**3 || Number(disk.bavail)*Number(disk.bsize)<10*1024**3)
  throw Error('Insufficient RAM/disk');
}
async function command(name,cmd,args,allowFailure=false){
 await resource();
 const r=await runCommand(cmd,args,{cwd:root,env,log:resolve(out,name+'.log'),timeoutMs:600000,resourceCheck:resource});
 records.push({name,cmd,args,...r});console.log(name,r.status);
 await writeFile(resolve(out,mode+'-commands.json'),JSON.stringify(records,null,2)+'\n');
 if(r.status!=='passed'&&!allowFailure)throw Error(name+' failed');return r;
}
if(mode==='native'){
 await command('native-phases','cargo',['test','--locked','-j2','-p','xterm-addons','--test','image_iip_pixel_phases','--test','image_iip','--test','kitty_geometry']);
 await command('native-addons','cargo',['test','--locked','-j2','-p','xterm-addons','--all-targets','--all-features']);
 await command('clippy-native','cargo',['clippy','--locked','-j2','-p','xterm-addons','--all-targets','--all-features','--','-D','warnings']);
 await command('clippy-wasm','cargo',['clippy','--locked','-j2','-p','xterm-web','--target','wasm32-unknown-unknown','--lib','--test','image_addon_browser','--','-D','warnings']);
}else if(mode==='format'){
 await command('fmt-all','cargo',['fmt','--all','--','--check'],true);
 for(const pkg of ['xterm-core','xterm-addons','xterm-web','xterm-wasm-support','xterm-font-addons'])
  await command('fmt-'+pkg,'cargo',['fmt','-p',pkg,'--','--check']);
}else if(['initial','final','iip','sixel','kitty'].includes(mode)){
 const test=['iip','sixel','kitty'].includes(mode)?'image_'+mode+'_terminal_browser':'image_addon_browser';
 env.WASM_TEST=test;
 const r=await command(mode,'node',['scripts/image-canvas-check.mjs'],true);
 await copyFile('results/browser-host/'+test+'-results.json',resolve(out,mode+'-browser.json'));
 const label=test==='image_addon_browser'?'image-addon':'image-'+mode+'-terminal';
 await copyFile('results/'+label+'/verification.json',resolve(out,mode+'-verification.json'));
 await copyFile('tmp/image-canvas/image-comparators.ts',resolve(out,mode+'-comparators.ts'));
 await copyFile('tmp/image-canvas/source-renderer.js',resolve(out,mode+'-source-renderer.js'));
 if(r.status!=='passed')throw Error(mode+' browser run failed');
}else throw Error('Expected native, format, initial, final, iip, sixel, kitty');
