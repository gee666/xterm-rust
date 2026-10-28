import {createWorktree,parseArgs} from './worktree-lib.mjs';
try {
  const args=parseArgs(process.argv.slice(2),['--agent','--base','--main','--copy-owned']);
  const result=await createWorktree({cwd:process.cwd(),agent:args['--agent'],base:args['--base'],main:args['--main'],copyOwned:args['--copy-owned']??[]});
  console.log(`Worktree: ${result.directory}\nBranch: ${result.branch}\nMain: ${result.main}\nCargo target: ${result.target}`);
  console.log('Work here, commit your changes, then run node scripts/worktree-merge.mjs from this worktree.');
} catch(error) {console.error(error.message);process.exitCode=1;}
