import { readFile, writeFile } from 'node:fs/promises';
import { spawnSync } from 'node:child_process';
const files = ['scripts/e2e/upstream-config.mjs', 'scripts/e2e-upstream.mjs', 'scripts/demo-e2e.mjs'];
const saved = await Promise.all(files.map(file => readFile(file)));
try {
  for (const file of files) {
    const old = spawnSync('C:/Program Files/Git/cmd/git.exe', ['show', `14139449:${file}`], { timeout: 10000 });
    if (old.error || old.status !== 0) throw old.error ?? new Error(`git show failed ${file}`);
    await writeFile(file, old.stdout);
  }
  const result = spawnSync(process.execPath, ['--experimental-vm-modules', '--test', 'scripts/e2e/runner-report.test.mjs'],
    { encoding: 'utf8', timeout: 30000, windowsHide: true });
  await writeFile('results/e2e-report583/old-runner-controls-final.log', `${result.stdout}\n${result.stderr}`);
  if (result.error || result.status !== 1) throw result.error ?? new Error(`Expected regression exit1, got ${result.status}`);
  console.log('Retained actual original entrypoint negative-control exit1. See raw log.');
} finally {
  for (let index = 0; index < files.length; index++) await writeFile(files[index], saved[index]);
}
