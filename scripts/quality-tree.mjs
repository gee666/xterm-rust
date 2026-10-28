import { ChildProcess, spawnSync } from 'node:child_process';
import { join } from 'node:path';

/** Stop only a ChildProcess owned by this Node process, including descendant
 * groups detached by managed servers. Unix uses the OS PID/PPID inventory. */
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
    return null;
  }
  const inventory = spawnSync('ps', ['-A', '-o', 'pid=', '-o', 'ppid='], { encoding: 'utf8', timeout: 5000 });
  const errors = [];
  const children = new Map();
  if (inventory.error || inventory.status !== 0) errors.push(inventory.error?.message ?? 'Cannot enumerate owned Unix descendants');
  else {
    for (const row of inventory.stdout.trim().split('\n')) {
      const match = row.trim().match(/^(\d+)\s+(\d+)$/);
      if (!match) { errors.push('Invalid Unix process inventory'); continue; }
      const pid = Number(match[1]);
      const parent = Number(match[2]);
      const list = children.get(parent) ?? [];
      list.push(pid);
      children.set(parent, list);
    }
  }
  const descendants = [];
  const seen = new Set([child.pid]);
  const queue = [child.pid];
  for (let i = 0; i < queue.length; i++) {
    for (const pid of children.get(queue[i]) ?? []) {
      if (seen.has(pid)) continue;
      if (pid === process.pid) { errors.push('Refusing to stop the process owner'); continue; }
      seen.add(pid);
      descendants.push(pid);
      queue.push(pid);
    }
  }
  // Inventory is taken while the owned parent is live. Kill descendants first,
  // then its private group to catch ordinary children spawned during inventory.
  for (const pid of descendants.reverse()) {
    try { process.kill(pid, 'SIGKILL'); }
    catch (error) { if (error.code !== 'ESRCH') errors.push(error.message); }
  }
  try { process.kill(-child.pid, 'SIGKILL'); }
  catch (error) {
    if (error.code !== 'ESRCH') errors.push(error.message);
    child.kill('SIGKILL');
  }
  return errors.length ? errors.join('; ') : null;
}
