#!/usr/bin/env node
// Run in the agent's worktree. Source traces first, then the actual Rust DOM
// tests in all three engines. The browser runner retains its own normal logs.
import { mkdir } from 'node:fs/promises';
import { fileURLToPath } from 'node:url';
import path from 'node:path';
import { runCommand } from './quality-process.mjs';

const root = fileURLToPath(new URL('../', import.meta.url));
await mkdir(path.join(root, 'tmp/mouse-live'), { recursive: true });
for (const [name, script, timeoutMs] of [
  ['source', 'scripts/mouse-live-oracle.mjs', 30000],
  ['browser', 'scripts/browser-host-check.mjs', 240000],
]) {
  const result = await runCommand(process.execPath, [script], {
    cwd: root,
    env: { ...process.env, WASM_TEST: 'wasm_mouse', BROWSERS: 'chromium,firefox,webkit' },
    log: path.join(root, 'tmp/mouse-live', `${name}.log`), timeoutMs,
  });
  console.log(`${name}: ${result.status}`);
  if (result.status !== 'passed') {
    console.error(`See tmp/mouse-live/${name}.log`);
    process.exitCode = 1;
    break;
  }
}
