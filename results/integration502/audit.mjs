// Reuses integration476 bounded immutable-object audit.
// Reuses integration463's immutable Git-object audit and batched blob reads.
// Expected shared files are composed from preserved producers, not this checkout.
import fs from 'node:fs';
import crypto from 'node:crypto';
import {execFileSync} from 'node:child_process';
const cache = new Map();
function git(...args) {
  const key = JSON.stringify(args);
  if (!cache.has(key)) cache.set(key, execFileSync(process.env.GIT ?? 'git', args,
    {timeout: 30000, maxBuffer: 32 * 1024 * 1024}));
  return cache.get(key);
}
const text = (...args) => git(...args).toString().trim();
const trees = new Map();
function exists(ref, file) {
  if (!trees.has(ref)) trees.set(ref, new Set(text('ls-tree', '-r', '--name-only', ref).split('\n')));
  return trees.get(ref).has(file);
}
function prime(specs) {
  const unique = [...new Set(specs)];
  // Producer receipts include WASM binaries. Bound each batch rather than
  // buffering every historical browser build in one child-process result.
  if (unique.length > 8) {
    for (let index = 0; index < unique.length; index += 8) prime(unique.slice(index, index + 8));
    return;
  }
  const output = execFileSync(process.env.GIT ?? 'git', ['cat-file', '--batch'],
    {input: unique.join('\n') + '\n', timeout: 30000, maxBuffer: 128 * 1024 * 1024});
  let cursor = 0;
  for (const spec of unique) {
    const end = output.indexOf(10, cursor), header = output.subarray(cursor, end).toString();
    const match = /^\S+ blob (\d+)$/.exec(header);
    if (!match) throw Error(`Invalid object ${spec}: ${header}`);
    const size = Number(match[1]), start = end + 1;
    cache.set(JSON.stringify(['show', spec]), output.subarray(start, start + size));
    cursor = start + size + 1;
  }
  if (cursor !== output.length) throw Error('Unexpected batch object suffix');
}
const sha = bytes => crypto.createHash('sha256').update(bytes).digest('hex');
const plan = JSON.parse(fs.readFileSync('results/integration502/plan.json'));
const blob = (ref, file) => git('show', `${ref}:${file}`).toString();
function replace(value, before, after) {
  if (value.split(before).length !== 2) throw Error(`Ambiguous replacement: ${before}`);
  return value.replace(before, after);
}
function segment(value, start, end) {
  const from = value.indexOf(start), to = value.indexOf(end, from + start.length);
  if (from < 0 || to < 0) throw Error(`Missing producer segment ${start}`);
  return value.slice(from, to);
}
const combined = new Map();
// 477's frame resolver and 486's timer adapter own disjoint sections.
const debouncePath = 'crates/xterm-web/src/links/browser_debounce.rs';
const timerStart = '/// Reusable browser timeout/clock adapter.';
const baseDebounce = blob(plan.base, debouncePath);
const timerDebounce = blob('f049ce29', debouncePath);
const framePrefix = replace(baseDebounce.slice(0, baseDebounce.indexOf(timerStart)),
  'type TimerCallback = Closure<dyn FnMut() -> Result<(), JsValue>>;',
  'type TimerCallback = Closure<dyn Fn() -> Result<(), JsValue>>;');
combined.set(debouncePath, Buffer.from(framePrefix + timerDebounce.slice(timerDebounce.indexOf(timerStart))));
// Keep 479's disposal contract while adding 490's lossless name errors.
const mouseApi = 'crates/xterm-web/src/terminal/mouse_api.rs';
const producerApi = blob('78355f37', mouseApi), baseApi = blob(plan.base, mouseApi);
const withImport = replace(producerApi, 'use xterm_core::{\n', 'use xterm_core::{\n    lifecycle::Disposable,\n');
const disposeStart = '\n    /// Only the registered protocol emitter is disposed.';
const disposeTail = baseApi.slice(baseApi.indexOf(disposeStart));
if (!disposeTail.startsWith(disposeStart)) throw Error('Missing base mouse disposal');
combined.set(mouseApi, Buffer.from(withImport.slice(0, withImport.lastIndexOf('}\n')) + disposeTail));
const mouseRunner = 'scripts/mouse-mode-source.mjs';
const familyEnd = "  disposal: ['browser_mouse_dispose_cases.js', 'mouse-disposal'],\n};";
combined.set(mouseRunner, Buffer.from(replace(blob(plan.base, mouseRunner), familyEnd,
  familyEnd.replace('};', "  'name-errors': ['browser_mouse_name_cases.js', 'mouse-name-errors'],\n};"))));
const source = file => ['crates/', 'scripts/', '.github/', '.pi/swarm/skills/'].some(p => file.startsWith(p));
const expected = new Map(), owners = new Map();
const report = {base: plan.base, assembly: text('rev-parse', 'HEAD'), batches: [], sourcePins: [],
  receiptPins: [], preservedBasePins: [], pointerExceptions: ['results/demos/build/latest.json']};
for (const batch of plan.batches) {
  const chain = batch.chain.map(ref => text('rev-parse', ref));
  const paths = [...new Set(chain.flatMap(ref => text('diff-tree', '--root', '--no-commit-id',
    '--name-only', '-r', ref).split('\n').filter(Boolean)))];
  if (batch.receiptOnly && paths.some(source)) throw Error(`Receipt-only task${batch.task} changes source`);
  report.batches.push({...batch, chain, paths: paths.length});
  for (const file of paths) {
    if (file === 'results/demos/build/latest.json') continue;
    expected.set(file, {task: batch.task, ref: chain.at(-1), present: exists(chain.at(-1), file)});
    if (source(file)) owners.set(file, [...new Set([...(owners.get(file) ?? []), batch.task])]);
  }
}
const baseline = JSON.parse(git('show', `${plan.base}:results/integration494/manifest.json`));
const oldFiles = [...new Set([...baseline.sourcePins, ...baseline.preservedBasePins].map(pin => pin.path))];
prime([...expected].filter(([file, entry]) => entry.present && (source(file) || file.startsWith('results/')))
  .flatMap(([file, entry]) => [`${entry.ref}:${file}`, `HEAD:${file}`])
  .concat(oldFiles.flatMap(file => [`${plan.base}:${file}`, `HEAD:${file}`])));
for (const [file, entry] of expected) {
  if (!source(file) && !file.startsWith('results/')) continue;
  if (!entry.present) {
    report.receiptPins.push({path: file, deleted: true, matches: !exists('HEAD', file)});
    continue;
  }
  const producer = git('show', `${entry.ref}:${file}`), wanted = combined.get(file) ?? producer;
  const actual = git('show', `HEAD:${file}`);
  const pin = {task: entry.task, path: file, producerSha256: sha(producer), expectedSha256: sha(wanted),
    headSha256: sha(actual), composed: combined.has(file), matches: wanted.equals(actual)};
  if (source(file)) {
    const working = fs.readFileSync(file);
    pin.checkoutEolOnly = !wanted.equals(working) && file.endsWith('.md') &&
      wanted.toString().replaceAll('\r\n', '\n') === working.toString().replaceAll('\r\n', '\n');
    pin.matches &&= wanted.equals(working) || pin.checkoutEolOnly;
    pin.owners = owners.get(file);
    report.sourcePins.push(pin);
  } else report.receiptPins.push(pin);
}
for (const file of oldFiles) {
  const updated = report.sourcePins.find(pin => pin.path === file);
  const wanted = updated?.expectedSha256 ?? sha(git('show', `${plan.base}:${file}`));
  const actual = sha(git('show', `HEAD:${file}`));
  report.preservedBasePins.push({path: file, updated: updated?.task ?? null, expectedSha256: wanted,
    headSha256: actual, matches: wanted === actual});
}
report.receiptPins.push({path: 'results/demos/build/latest.json', preservedMainPointer: true,
  matches: git('show', `${plan.base}:results/demos/build/latest.json`).equals(git('show', 'HEAD:results/demos/build/latest.json'))});
const unexpectedSource = text('diff', '--name-only', plan.base, 'HEAD').split('\n')
  .filter(file => source(file) && !expected.has(file));
const mismatches = [...report.sourcePins, ...report.receiptPins, ...report.preservedBasePins].filter(p => !p.matches);
report.unexpectedSource = unexpectedSource;
report.status = mismatches.length || unexpectedSource.length ? 'failed' : 'passed';
if (process.argv.includes('--write')) fs.writeFileSync('results/integration502/manifest.json', JSON.stringify(report, null, 2) + '\n');
console.log(JSON.stringify({status: report.status, sources: report.sourcePins.length, composed: combined.size,
  receipts: report.receiptPins.length, preserved: report.preservedBasePins.length, unexpectedSource, mismatches}));
if (report.status !== 'passed') process.exitCode = 1;
