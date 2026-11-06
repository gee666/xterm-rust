// Evidence-only model. The actual hosted fmt command still decides runtime success.
import {execFileSync} from 'node:child_process';
import fs from 'node:fs';
import path from 'node:path';
const root=process.cwd();
const metadata=JSON.parse(execFileSync('cargo',['metadata','--no-deps','--format-version','1','--locked'],
  {encoding:'utf8',timeout:30000}));
const packages=metadata.packages.filter(pkg=>metadata.workspace_members.includes(pkg.id));
const groups=Object.groupBy(packages.flatMap(pkg=>pkg.targets),target=>target.edition);
const limit=32767,reserve=512;
const roots=['D:/a/q','D:/q'].map(physicalRoot=>({physicalRoot,
  groups:Object.entries(groups).map(([edition,targets])=>{
    const quotedFileArguments=targets.reduce((total,target)=>total+physicalRoot.length+1+path.relative(root,target.src_path).length+7,0);
    return {edition,roots:targets.length,quotedFileArguments,nonFileArgumentReserve:reserve,
      remainingAfterReserve:limit-reserve-quotedFileArguments,fits:quotedFileArguments<=limit-reserve};
  })}));
const workflow=fs.readFileSync('.github/workflows/rust.yml','utf8');
const expression="[IO.Path]::GetFullPath((Join-Path $env:RUNNER_TEMP '../../q'))";
if(workflow.split(expression).length!==3)throw Error('Setup and cleanup must select the same short root');
console.log(JSON.stringify({scope:'All current metadata targets, modeled hosted roots. Not actual hosted fmt execution.',
  platform:process.platform,node:process.version,windowsCommandLineLimit:limit,expression,
  setupAndCleanupMatch:true,gateCommandsUnchanged:true,roots},null,2));
