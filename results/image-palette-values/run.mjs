// Use when rerunning the scoped raw-palette gates in an owned O: worktree.
import {mkdir,readFile,writeFile,copyFile,statfs} from 'node:fs/promises';
import {freemem} from 'node:os';
import {resolve} from 'node:path';
import {runCommand} from '../../scripts/quality-process.mjs';
const out='results/image-palette-values',mode=process.argv[2],root=process.cwd();
await mkdir(out,{recursive:true});await mkdir('tmp/palette-temp',{recursive:true});
const env={...process.env,CARGO_BUILD_JOBS:'2',CARGO_TARGET_DIR:resolve('target'),
  TEMP:resolve('tmp/palette-temp'),TMP:resolve('tmp/palette-temp')};
const records=[];
async function resource(){
  const disk=await statfs(root);
  if(freemem()<8*1024**3 || Number(disk.bavail)*Number(disk.bsize)<10*1024**3)throw Error('Insufficient RAM/disk');
}
async function command(name,cmd,args,allowFailure=false){
  await resource();
  const record=await runCommand(cmd,args,{cwd:root,env,log:resolve(out,name+'.log'),timeoutMs:600000,resourceCheck:resource});
  records.push({name,...record});console.log(name,record.status);
  await writeFile(resolve(out,mode+'-commands.json'),JSON.stringify(records,null,2)+'\n');
  if(record.status!=='passed'&&!allowFailure)throw Error(name+' failed');
  return record;
}
if(mode==='native'){
  await command('native-sixel','cargo',['test','--locked','-j2','-p','xterm-addons','--lib','image::sixel']);
  await command('native-addons','cargo',['test','--locked','-j2','-p','xterm-addons','--all-targets','--all-features']);
  await command('clippy-native','cargo',['clippy','--locked','-j2','-p','xterm-addons','--all-targets','--all-features','--','-D','warnings']);
  await command('clippy-wasm','cargo',['clippy','--locked','-j2','-p','xterm-web','--target','wasm32-unknown-unknown','--lib','--test','image_addon_browser','--test','image_sixel_terminal_browser','--','-D','warnings']);
}else if(mode==='format'){
  await command('fmt-all','cargo',['fmt','--all','--','--check'],true);
  for(const pkg of ['xterm-core','xterm-addons','xterm-web','xterm-wasm-support','xterm-font-addons'])
    await command('fmt-'+pkg,'cargo',['fmt','-p',pkg,'--','--check']);
}else if(['initial','final','producer'].includes(mode)){
  const test=mode==='producer'?'image_sixel_terminal_browser':'image_addon_browser';
  env.WASM_TEST=test;
  const record=await command(mode,'node',['scripts/image-canvas-check.mjs'],true);
  for(const [from,to]of [[`results/browser-host/${test}-results.json`,mode+'-browser.json'],
    [`results/${mode==='producer'?'image-sixel-terminal':'image-addon'}/verification.json`,mode+'-verification.json']])
    await copyFile(from,resolve(out,to));
  if(record.status!=='passed')throw Error(mode+' browser run failed');
}else throw Error('Expected native, format, initial, final or producer');
