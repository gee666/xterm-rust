import { spawnSync } from 'node:child_process';
import { basename, join } from 'node:path';

/** Conservative inventory only. Never terminate an unknown user's process. */
export function observeBuildProcesses(target) {
  const names = new Set(['cargo', 'rustc', 'rustdoc', 'clippy-driver', 'wasm-bindgen', 'wasm-bindgen-test-runner', 'wasm-pack', 'wasm-opt', 'rust-lld', 'lld-link', 'esbuild', 'robocopy', 'link']);
  if (process.platform === 'win32') {
    const script = '$ErrorActionPreference="Stop"; Get-CimInstance Win32_Process | Where-Object { $_.Name -in @("cargo.exe","rustc.exe","rustdoc.exe","clippy-driver.exe","wasm-bindgen.exe","wasm-bindgen-test-runner.exe","wasm-pack.exe","wasm-opt.exe","rust-lld.exe","lld-link.exe","esbuild.exe","robocopy.exe","link.exe") -or ($_.ExecutablePath -and $_.ExecutablePath.StartsWith($env:XTERM_TARGET_ROOT,[System.StringComparison]::OrdinalIgnoreCase)) } | Select-Object ProcessId,Name | ConvertTo-Json -Compress';
    const result = spawnSync(join(process.env.SystemRoot, 'System32', 'WindowsPowerShell', 'v1.0', 'powershell.exe'),
      ['-NoProfile', '-NonInteractive', '-Command', script], {
        env: { ...process.env, XTERM_TARGET_ROOT: target + '\\' }, encoding: 'utf8', timeout: 30_000
      });
    if (result.error || result.status !== 0) throw new Error('Windows idle-process inventory failed; operation refused');
    const entries = result.stdout.trim() ? JSON.parse(result.stdout) : [];
    return (Array.isArray(entries) ? entries : [entries]).map(item => ({ pid: item.ProcessId, name: item.Name }));
  }
  const result = spawnSync('ps', ['-A', '-o', 'pid=', '-o', 'args='], { encoding: 'utf8', timeout: 5000 });
  if (result.error || result.status !== 0) throw new Error('Unix idle-process inventory failed; operation refused');
  const active = [];
  for (const row of result.stdout.trim().split('\n')) {
    const match = row.trim().match(/^(\d+)\s+(\S+)(.*)$/);
    if (!match) throw new Error('Unrecognized process inventory; operation refused');
    const name = basename(match[2]);
    if (names.has(name) || row.includes(target + '/') || /(?:^|\s)(?:\.\/)?target\//.test(match[2] + match[3])) active.push({ pid: Number(match[1]), name });
  }
  return active;
}
