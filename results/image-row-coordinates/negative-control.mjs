// Use when checking the row regressions against exact pre515 exported wrappers.
import {readFile, writeFile} from 'node:fs/promises';
import {createHash} from 'node:crypto';
import {execFileSync} from 'node:child_process';
import {resolve} from 'node:path';
import {runCommand} from '../../scripts/quality-process.mjs';
const out=resolve('results/image-row-coordinates');
const files=['crates/xterm-web/src/image/addon.rs','crates/xterm-web/src/image/addon/bindings.rs'];
const baseline='1106d480';
const hash=b=>createHash('sha256').update(b).digest('hex');
const saved=await Promise.all(files.map(async path=>({path,bytes:await readFile(path)})));
const audit={baseline,files:[],restored:false};
try {
  for(const item of saved) {
    const before=execFileSync(process.env.GIT??'git',['show',`${baseline}:${item.path}`],{timeout:30000,maxBuffer:16*1024*1024});
    audit.files.push({path:item.path,fixed:hash(item.bytes),baseline:hash(before)});
    await writeFile(item.path,before);
  }
  const command=await runCommand(process.execPath,['results/image-row-coordinates/browser.mjs','negative'],{
    cwd:process.cwd(),env:process.env,log:resolve(out,'negative-driver.log'),timeoutMs:360000
  });
  audit.command=command;
  if(command.status!=='passed')throw Error('Negative matrix did not meet its exact assertions');
  const report=JSON.parse(await readFile(resolve(out,'negative-browser.json'),'utf8'));
  const failed=[...report.browsers[0].output.matchAll(/^test ([^\n]+) \.\.\. FAIL$/gm)].map(m=>m[1]);
  audit.failed=failed;
  if(failed.length!==3||failed.some(name=>!name.startsWith('row_coordinates::')))throw Error('Control failed inherited or missing row tests');
} finally {
  for(const item of saved)await writeFile(item.path,item.bytes);
  for(const item of saved)if(hash(await readFile(item.path))!==hash(item.bytes))throw Error('Failed exact restoration '+item.path);
  audit.restored=true;
  await writeFile(resolve(out,'negative-audit.json'),JSON.stringify(audit,null,2)+'\n');
}
console.log('Exact two-file baseline control completed and both fixed hashes restored');
