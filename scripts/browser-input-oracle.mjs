#!/usr/bin/env node
// Licensed under MIT. Policy oracle, not a substitute for live browser tests.
import assert from 'node:assert/strict';
import { readFileSync, writeFileSync, mkdirSync, statfsSync } from 'node:fs';
import { fileURLToPath } from 'node:url';
import { spawnSync } from 'node:child_process';
import { createHash } from 'node:crypto';
import path from 'node:path';
import { loadSource, hex, row, referencePaths } from './browser-input-oracle-source.mjs';
import { runCommand } from './quality-process.mjs';

const rootUrl = new URL('../', import.meta.url);
const root = fileURLToPath(rootUrl);
const out = path.join(root, 'results/browser-input');
const tmp = path.join(root, 'tmp/browser-input');
mkdirSync(out, { recursive: true });
mkdirSync(tmp, { recursive: true });
const source = loadSource(rootUrl);
const original = source.runOriginal();
assert.equal(original.tests.length, 11, '8 IME and3 clipboard tests must remain discovered');
assert.equal(original.assertions, 16, 'original assertion count must not shrink');
const upstream = [...original.clipboard, ...original.traces];
const fixture = upstream.map(item => `${item.input}\t${item.expected}\t${item.name}`).join('\n') + '\n';
const fixturePath = path.join(root, 'crates/xterm-web/tests/fixtures/browser-input-upstream.tsv');
if (process.argv.includes('--generate')) writeFileSync(fixturePath, fixture);
else assert.equal(readFileSync(fixturePath, 'utf8'), fixture, 'fixture drift requires reviewed --generate');
const cases = [...upstream];
const counts = { original: upstream.length };
const seed = 0x494d4536;
let state = seed;
function random() { state = (Math.imul(state, 1664525) + 1013904223) >>> 0; return state; }
function text() {
  const units = [0, 10, 13, 27, 0x61, 0x4e00, 0xd800, 0xdc00, 0xd83d, 0xde00, 0x241b];
  return Array.from({ length: random() >>> 28 }, () => String.fromCharCode(units[(random() >>> 16) % units.length])).join('');
}
function add(action, value, mode, ignore, expected, name) {
  cases.push({ input: row(action, value, mode, ignore), expected, name });
  counts[name] = (counts[name] || 0) + 1;
}
for (let unit = 0; unit < 65536; unit++) {
  const value = `\r\n${String.fromCharCode(unit)}\x1b\n`;
  add('prepare', value, '-', '-', hex(source.clipboard.prepareTextForTerminal(value)), 'BMP line endings');
  add('bracket', value, 1, '-', hex(source.clipboard.bracketTextForPaste(value, true)), 'BMP ESC bracketing');
}
for (let i = 0; i < 10000; i++) {
  const value = text();
  for (const mode of [false, true]) for (const ignore of [false, true]) {
    const data = [];
    const textarea = { value: 'before' };
    source.clipboard.paste(value, textarea, { decPrivateModes: { bracketedPasteMode: mode }, triggerDataEvent: (value, user) => {
      assert.equal(user, true); data.push(value);
    } }, { rawOptions: { ignoreBracketedPasteMode: ignore } });
    assert.equal(data.length, 1);
    assert.equal(textarea.value, '');
    add('paste', value, Number(mode), Number(ignore), hex(data[0]), 'seeded paste flags');
  }
}
const directedStart = source.traces.length;
const directed = [
  ['insertion and repeated229', [['value', 'aba'], ['key', '', 229], ['key', '', 229], ['value', 'ababa'], ['tick']]],
  ['deletion', [['value', 'ab'], ['key', '', 229], ['value', 'a'], ['tick']]],
  ['same-length replacement', [['value', 'ab'], ['key', '', 229], ['value', '中a'], ['tick']]],
  ['no-change', [['value', 'ab'], ['key', '', 229], ['tick']]],
  ['composition cancels pendingdiff', [['key', '', 229], ['start'], ['value', '中'], ['tick'], ['end'], ['tick']]],
  ['immediate commit cancels send', [['start'], ['update', '中'], ['value', '中'], ['tick'], ['end'], ['key', '', 13], ['tick']]],
  ['deferred text before newcomposition', [['start'], ['update', 'a'], ['value', 'a'], ['tick'], ['end'], ['start'], ['update', 'b'], ['value', 'ab'], ['tick'], ['tick'], ['end'], ['tick']]],
  ['reversed selection and suffix', [['value', 'a中b'], ['startsel', '', 2], ['endsel', '', 1], ['start'], ['update', '😀'], ['value', 'a😀b'], ['endsel', '', 3], ['tick'], ['end'], ['value', 'a😀1b'], ['tick']]],
  ['lone surrogates and modifier gates', [['start'], ['update', '\ud800'], ['value', '\ud800'], ['key', '', 16], ['key', '', 17], ['key', '', 18], ['key', '', 20], ['key', '', 229], ['tick'], ['end'], ['tick']]]
];
for (const [name, steps] of directed) {
  source.apply('reset', name);
  for (const [action, value = '', number = '-'] of steps) source.apply(action, value, number);
}
const directedSteps = source.traces.slice(directedStart);
const directedFixture = directedSteps.map(item => `${item.input}\t${item.expected}\t${item.name}`).join('\n') + '\n';
const directedPath = path.join(root, 'crates/xterm-web/tests/fixtures/browser-input-directed.tsv');
if (process.argv.includes('--generate')) writeFileSync(directedPath, directedFixture);
else assert.equal(readFileSync(directedPath, 'utf8'), directedFixture, 'directed fixture drift requires review');
cases.push(...directedSteps);
counts['directed composition steps'] = directedSteps.length;
const traceStart = source.traces.length;
const actions = ['value', 'start', 'update', 'end', 'key', 'tick', 'startsel', 'endsel'];
const keys = [16, 17, 18, 20, 13, 65, 229, 0, 0xffffffff];
for (let scenario = 0; scenario < 2000; scenario++) {
  source.apply('reset', `seeded composition ${scenario}`);
  for (let step = 0; step < 35; step++) {
    const action = actions[(random() >>> 16) % actions.length];
    if (action === 'key') source.apply(action, '', keys[(random() >>> 16) % keys.length]);
    else if (action === 'value' || action === 'update') source.apply(action, text());
    else if (action === 'startsel' || action === 'endsel') source.apply(action, '', random() & 1 ? '-' : random() >>> 28);
    else if (action !== 'tick' || source.pending()) source.apply(action);
  }
  let turns = 0;
  while (source.pending()) { assert.ok(++turns < 10000); source.apply('tick'); }
}
const generated = source.traces.slice(traceStart);
cases.push(...generated);
counts['seeded composition steps'] = generated.length;
function run(command, args, options = {}) {
  const result = spawnSync(command, args, { cwd: root, encoding: 'utf8', timeout: 120000,
    maxBuffer: 128 * 1024 * 1024, ...options });
  if (result.error) throw result.error;
  assert.equal(result.status, 0, `${command} ${args.join(' ')}\n${result.stderr}\n${result.stdout}`);
  return result.stdout;
}
const disk = statfsSync(root);
const freeBytesBeforeBuild = disk.bavail * disk.bsize;
assert.ok(freeBytesBeforeBuild > 512 * 1024 * 1024, 'less than512MiB free; stop before exhausting shared build drive');
const build = await runCommand(process.env.CARGO || 'cargo', ['build', '-p', 'xterm-web', '--example', 'input_trace'], {
  cwd: root, env: process.env, log: path.join(out, 'build.log'), timeoutMs: 120000,
  resourceCheck: () => {
    const current = statfsSync(root);
    assert.ok(current.bavail * current.bsize > 512 * 1024 * 1024, 'shared drive free space fell below512MiB');
  }
});
writeFileSync(path.join(out, 'build.json'), JSON.stringify({ ...build, freeBytesBeforeBuild }, null, 2) + '\n');
if (build.status !== 'passed') {
  writeFileSync(path.join(out, 'verification.json'), JSON.stringify({ command: 'node scripts/browser-input-oracle.mjs',
    status: 'build_failed', platform: process.platform, node: process.version, seed, build,
    upstream: { discovered: original.tests.length, executed: original.tests.length, passed: original.tests.length,
      assertions: original.assertions, clipboardCalls: original.clipboard.length, compositionTraceSteps: original.traces.length },
    differential: { planned: cases.length, executed: 0, unrun: cases.length },
    scope: 'Current Rust build failed; prior successful evidence is historical, not a current pass.' }, null, 2) + '\n');
}
assert.equal(build.status, 'passed', 'Rust runner build failed; see results/browser-input/build.log and build.json');
const binary = path.join(process.env.CARGO_TARGET_DIR || path.join(root, 'target'), 'debug/examples', process.platform === 'win32' ? 'input_trace.exe' : 'input_trace');
writeFileSync(path.join(tmp, 'inputs.tsv'), cases.map(item => item.input).join('\n') + '\n');
const stdout = run(binary, [], { input: readFileSync(path.join(tmp, 'inputs.tsv'), 'utf8') });
const actual = stdout.split(/\r?\n/);
assert.equal(actual.pop(), '');
assert.equal(actual.length, cases.length);
let failures = 0;
const mismatches = [];
for (let index = 0; index < cases.length; index++) {
  if (actual[index] !== cases[index].expected) {
    failures++;
    if (mismatches.length < 20) {
      let start = index;
      while (start > 0 && !cases[start].input.startsWith('reset\t') && cases[start].name === cases[start - 1].name) start--;
      mismatches.push({ index, case: cases[index], actual: actual[index], trace: cases.slice(start, index + 1) });
    }
  }
}
const hashFiles = names => Object.fromEntries(names.map(name => [name, createHash('sha256').update(readFileSync(path.join(root, name))).digest('hex')]));
const sourceHashes = hashFiles(referencePaths);
const implementationHashes = hashFiles(['crates/xterm-web/src/input/clipboard.rs', 'crates/xterm-web/src/input/composition.rs',
  'crates/xterm-web/tests/support/input_trace.rs', 'crates/xterm-web/examples/input_trace.rs',
  'scripts/browser-input-oracle-source.mjs', 'scripts/browser-input-oracle.mjs']);
const report = { command: 'node scripts/browser-input-oracle.mjs', status: failures ? 'failed' : 'passed', platform: process.platform, arch: process.arch,
  node: process.version, rustc: run(process.env.RUSTC || 'rustc', ['--version']).trim(), seed, freeBytesBeforeBuild,
  sourceHashes, implementationHashes, build, upstream: { discovered: original.tests.length, executed: original.tests.length,
    passed: original.tests.length, failed: 0, skipped: 0, unrun: 0, assertions: original.assertions,
    clipboardCalls: original.clipboard.length, compositionTraceSteps: original.traces.length, names: original.tests },
  differential: { executed: cases.length, passed: cases.length - failures, failed: failures, retainedMismatches: mismatches.length, counts },
  scope: 'Clipboard string policy and CompositionHelper deferred/state/output policy only. Geometry, actual DOM events, browser timing and full terminal equivalence NOT PROVEN.' };
writeFileSync(path.join(out, 'verification.json'), JSON.stringify(report, null, 2) + '\n');
if (mismatches.length) writeFileSync(path.join(out, 'counterexamples.json'), JSON.stringify({ seed, mismatches }, null, 2) + '\n');
assert.equal(failures, 0, `input differential failed; retained ${mismatches.length} counterexamples in results/browser-input`);
console.log(JSON.stringify({ tests: original.tests.length, assertions: original.assertions, comparisons: cases.length, seed }));
