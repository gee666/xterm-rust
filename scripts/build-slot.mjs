#!/usr/bin/env node
import { fileURLToPath } from 'node:url';
import { runWithSlot } from './build-slot-core.mjs';

async function main() {
  const args = process.argv.slice(2);
  const split = args.indexOf('--');
  if (split < 0 || !args[split + 1]) throw new Error('Usage: node scripts/build-slot.mjs --owner NAME --lane native|wasm [--wait-ms N] [--timeout-ms N] -- COMMAND ARGS');
  const options = { waitTimeoutMs: 120_000, timeoutMs: 180_000 };
  const names = { '--owner': 'owner', '--lane': 'lane', '--wait-ms': 'waitTimeoutMs', '--timeout-ms': 'timeoutMs' };
  const seen = new Set();
  for (let index = 0; index < split; index += 2) {
    if (seen.has(args[index])) throw new Error('Duplicate build-slot flag');
    seen.add(args[index]);
    const name = names[args[index]];
    const value = args[index + 1];
    if (!name || !value || index + 1 >= split) throw new Error('Unknown or incomplete build-slot flag');
    options[name] = name.endsWith('Ms') ? Number(value) : value;
  }
  for (const key of ['waitTimeoutMs', 'timeoutMs']) {
    if (!Number.isSafeInteger(options[key]) || options[key] < 100 || options[key] > 1_800_000) throw new Error('Deadlines must be integers from100 to1800000 milliseconds');
  }
  const command = args[split + 1];
  const commandArgs = args.slice(split + 2);
  if (commandArgs.some(arg => ['--workspace', '--all-targets', '--release', '--target-dir', '--config'].includes(arg) || arg.startsWith('--target-dir=') || arg.startsWith('--config='))) {
    throw new Error('Local leases are for scoped debug checks. Full-workspace/all-target/release gates run remotely or require separate coordination.');
  }
  const controller = new AbortController();
  const interrupt = () => controller.abort(new Error('Build slot owner interrupted'));
  process.once('SIGINT', interrupt);
  process.once('SIGTERM', interrupt);
  try {
    const result = await runWithSlot({ ...options, root: fileURLToPath(new URL('../', import.meta.url)),
      command, args: commandArgs, signal: controller.signal,
      onQueued: item => console.log(`Queued ${item.owner} in ${item.lane}: ${item.id}`) });
    console.log(JSON.stringify(result, null, 2));
    if (result.status !== 'passed') process.exitCode = 1;
  } finally {
    process.removeListener('SIGINT', interrupt);
    process.removeListener('SIGTERM', interrupt);
  }
}
await main().catch(error => { console.error(error.message ?? String(error)); process.exitCode = 1; });
