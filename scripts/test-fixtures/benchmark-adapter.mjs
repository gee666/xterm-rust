import { readFileSync, writeFileSync } from 'node:fs';
import { result } from './benchmark-data.mjs';
const [mode, engine, ...args] = process.argv.slice(2);
if (mode === 'nonzero') process.exit(7);
if (mode === 'signal') process.kill(process.pid, 'SIGTERM');
// Deliberately nonterminating negative fixtures. The runner owns and kills them.
if (mode === 'timeout') setInterval(() => {}, 1000);
else {
  const input = JSON.parse(readFileSync(args[args.indexOf('--benchmark-manifest') + 1], 'utf8'));
  const target = args[args.indexOf('--benchmark-output') + 1];
  if (mode.startsWith('partial-')) {
    writeFileSync(target, '{partial-result}');
    if (mode === 'partial-nonzero') process.exit(7);
    setInterval(() => {}, 1000);
  } else if (mode.startsWith('oversized')) {
    writeFileSync(1, 'x'.repeat(9 * 1024 * 1024));
    if (mode === 'oversized') process.exit(7);
    setInterval(() => {}, 1000);
  } else if (mode === 'malformed') writeFileSync(target, '{broken');
  else if (mode !== 'missing') {
    const receipt = result(input, engine);
    if (mode === 'pending') receipt.cases[0].measurements[0].pendingOperations = 1;
    writeFileSync(target, JSON.stringify(receipt));
  }
}
