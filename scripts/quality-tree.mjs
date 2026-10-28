import { ChildProcess, spawnSync } from 'node:child_process';
import { join } from 'node:path';

/** Stop only a ChildProcess owned by this Node process. Unix children must have
 * been spawned detached so the PID identifies their private process group. */
export function stopProcessTree(child) {
  if (!(child instanceof ChildProcess)) throw new Error('A spawned ChildProcess handle is required, not an arbitrary PID');
  if (!child.pid || child.exitCode !== null || child.signalCode !== null) return null;
  if (process.platform === 'win32') {
    const kill = spawnSync(join(process.env.SystemRoot ?? 'C:\\Windows', 'System32', 'taskkill.exe'),
      ['/PID', String(child.pid), '/T', '/F'], { timeout: 10_000, windowsHide: true });
    if (kill.error || kill.status !== 0) {
      child.kill('SIGKILL');
      return kill.error?.message ?? kill.stderr.toString();
    }
  } else {
    try { process.kill(-child.pid, 'SIGKILL'); }
    catch (error) {
      if (error.code !== 'ESRCH') {
        child.kill('SIGKILL');
        return error.message;
      }
    }
  }
  return null;
}
