// Use when reproducing the native/dependent quality gates for phased image rows.
import {mkdir, readFile, writeFile, statfs} from 'node:fs/promises';
import {resolve} from 'node:path';
import {freemem} from 'node:os';
import {runCommand} from '../../scripts/quality-process.mjs';
const out = resolve('results/image-row-coordinates');
const scratch = resolve('tmp/native');
await mkdir(scratch, {recursive:true});
const env = {...process.env, TEMP:scratch, TMP:scratch, CARGO_BUILD_JOBS:'2', CARGO_TARGET_DIR:resolve('target')};
const commands = [];
async function headroom() {
  const disk = await statfs(process.cwd());
  const state = {RAMavailable:freemem(), diskFree:Number(disk.bavail)*Number(disk.bsize)};
  if (state.RAMavailable < 8*2**30 || state.diskFree < 10*2**30) throw Error('Insufficient resource headroom');
  return state;
}
await writeFile(resolve(out,'headroom-native.json'), JSON.stringify(await headroom(),null,2)+'\n');
const selection = process.argv[2] ?? 'native';
const cases = selection === 'native' ? [
  ['phased-core', ['test','--locked','-j2','-p','xterm-core','--test','circular_list_phased_reads']],
  ['workspace-tests', ['test','--locked','-j2','--workspace','--all-targets','--all-features']],
  ['workspace-clippy', ['clippy','--locked','-j2','--workspace','--all-targets','--all-features','--','-D','warnings']],
  ['wasm-lib-clippy', ['clippy','--locked','-j2','--workspace','--lib','--all-features','--target','wasm32-unknown-unknown','--','-D','warnings']],
  ['wasm-image-clippy', ['clippy','--locked','-j2','-p','xterm-web','--lib','--test','image_addon_browser','--all-features','--target','wasm32-unknown-unknown','--','-D','warnings']],
] : selection === 'format' ? [
  ['fmt-all', ['fmt','--all','--','--check']],
  ...['xterm-core','xterm-addons','xterm-web','xterm-font-addons','xterm-wasm-support'].map(p=>['fmt-'+p,['fmt','-p',p,'--','--check']])
] : (()=>{throw Error('Pass native or format');})();
for (const [name,args] of cases) {
  await headroom();
  const command = await runCommand(process.env.CARGO ?? 'cargo', args, {
    cwd:process.cwd(), env, log:resolve(out,name+'.log'), timeoutMs:480000, resourceCheck:headroom
  });
  commands.push({name,...command});
  await writeFile(resolve(out,selection+'-commands.json'),JSON.stringify(commands,null,2)+'\n');
  console.log(name, command.status);
  if (command.status !== 'passed' && !(selection==='format' && name==='fmt-all')) throw Error('Failed '+name);
}
if (selection === 'native') {
  const log = await readFile(resolve(out,'workspace-tests.log'),'utf8');
  const summaries = [...log.matchAll(/test result: ok\. (\d+) passed; (\d+) failed; (\d+) ignored; (\d+) measured; (\d+) filtered out/g)];
  const keys = ['passed','failed','ignored','measured','filtered'];
  const summary = {summaries:summaries.length, ...Object.fromEntries(keys.map((k,i)=>[k,summaries.reduce((n,m)=>n+Number(m[i+1]),0)]))};
  if (!summary.summaries || summary.failed || summary.ignored || summary.filtered) throw Error('Unexpected workspace test counters');
  await writeFile(resolve(out,'native-summary.json'),JSON.stringify(summary,null,2)+'\n');
  console.log(JSON.stringify(summary));
}
