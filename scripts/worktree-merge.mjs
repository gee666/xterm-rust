// Tested integration entry point for the canonical worktree-lib implementation.
import {mergeWorktree,parseArgs} from './worktree-lib.mjs';
try {
  const args=parseArgs(process.argv.slice(2),['--main']);
  const result=await mergeWorktree({cwd:process.cwd(),main:args['--main']});
  console.log(`Merged ${result.branch} into ${result.main}: ${result.commit}`);
  console.log(`Changed crates tested: ${result.packages.join(', ') || 'none'}; private target removed.`);
} catch(error) {console.error(error.message);process.exitCode=1;}
