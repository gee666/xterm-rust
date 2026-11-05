// Run only under the task's compiler/browser grant, with no concurrent edits.
import {readFile, writeFile} from 'node:fs/promises';
import {createHash} from 'node:crypto';
import {runCommand} from './quality-process.mjs';
import path from 'node:path';
const root = path.resolve(import.meta.dirname, '..');
const source = path.join(root, 'crates/xterm-web/src/webgl/cursor_blink_timers.rs');
const original = await readFile(source);
const before = original.toString();
const needle = 'if positive(&remaining)? {';
if (before.split(needle).length !== 2) throw Error('Mutation needs one exact guard');
const hash = bytes => createHash('sha256').update(bytes).digest('hex');
const receipt = {before:hash(original), mutation:'Invert initial restart positive-time guard', expected:{passed:2, failed:1}};
try {
  await writeFile(source, before.replace(needle, 'if !positive(&remaining)? {'));
  const command = await runCommand(process.execPath, [path.join(root, 'scripts/browser-host-check.mjs')], {
    cwd:root, env:{...process.env, BROWSERS:'chromium', WASM_TEST:'wasm_cursor_blink'},
    log:path.join(root, 'results/cursor-blink/negative.log'), timeoutMs:180000
  });
  receipt.command = command;
  const report = JSON.parse(await readFile(path.join(root, 'results/browser-host/wasm_cursor_blink-results.json')));
  receipt.report = report;
  const counts = report.browsers[0]?.counts;
  if (command.status !== 'failed' || counts?.passed !== 2 || counts?.failed !== 1 || counts?.ignored !== 0 || counts?.filtered !== 0) {
    throw Error('Expected exactly two passing tests and one lifecycle failure');
  }
  if (!report.browsers[0].output.includes('timer_lifecycle_getters_errors_and_reentry_match_source ... FAIL')) {
    throw Error('Negative failed outside the lifecycle assertion');
  }
  receipt.verified = true;
} finally {
  await writeFile(source, original);
  receipt.after = hash(await readFile(source));
  receipt.restored = receipt.before === receipt.after;
  await writeFile(path.join(root, 'results/cursor-blink/negative.json'), JSON.stringify(receipt, null, 2) + '\n');
  if (!receipt.restored) throw Error('Source restoration hash mismatch');
}
