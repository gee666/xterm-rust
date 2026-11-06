import { join } from 'node:path';
import { fileURLToPath } from 'node:url';

/** A private Windows Job closes orphan browser helpers when its wrapper exits.
 * Other commands retain the existing direct-spawn policy and process-tree tests.
 * No handle/PID guessing or machine-wide executable-name cleanup is involved. */
export function ownedCommand(command, args, { cwd, env, containDescendants }, platform = process.platform) {
  if (!containDescendants || platform !== 'win32') return { command, args, env };
  const executable = join(env.SystemRoot ?? 'C:\\Windows', 'System32/WindowsPowerShell/v1.0/powershell.exe');
  return { command: executable,
    args: ['-NoProfile', '-NonInteractive', '-ExecutionPolicy', 'Bypass', '-File',
      fileURLToPath(new URL('./windows-owned-job.ps1', import.meta.url))],
    env: { ...env, XTERM_OWNED_JOB_COMMAND: JSON.stringify({ executable: command, arguments: args, cwd }) } };
}
