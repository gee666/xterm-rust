// Run only under an explicit compiler/browser allocation. Restores production
// bytes even if an assertion or browser launch fails. No source reference edits.
import {readFile, writeFile, copyFile} from 'node:fs/promises';
import {createHash} from 'node:crypto';
import {fileURLToPath} from 'node:url';
import {resolve} from 'node:path';
import {runCommand} from '../../scripts/quality-process.mjs';
const root = fileURLToPath(new URL('../../', import.meta.url));
const out = resolve(root, 'results/webgl-glyph-backgrounds');
const file = resolve(root, 'crates/xterm-web/src/webgl/glyphs.rs');
const original = await readFile(file);
const hash = bytes => createHash('sha256').update(bytes).digest('hex');
const before = hash(original);
const from = 'numbers.current[0] != numbers.previous_background';
const to = '(numbers.current[0] as u32) != (numbers.previous_background as u32)';
const text = original.toString('utf8');
if (text.split(from).length !== 2) throw Error('Expected exactly one fixed comparison');
const mutant = text.replace(from, to);
const report = {before, mutant: hash(mutant), from, to, commands: []};
const env = {...process.env, CARGO_BUILD_JOBS: '2', WASM_TEST: 'wasm_webgl_glyphs'};
async function run(name, command, args, overrides = {}) {
  const result = await runCommand(command, args, {cwd: root, env: {...env, ...overrides},
    log: resolve(out, name + '.log'), timeoutMs: 240000});
  report.commands.push({name, ...result});
  if (result.error || result.timedOut || result.terminationError || result.signal) {
    throw Error('Command did not complete normally: ' + name);
  }
  return result;
}
async function browser(name, browsers) {
  const result = await run(name, process.execPath, ['scripts/browser-host-check.mjs'], {BROWSERS: browsers});
  const src = resolve(root, 'results/browser-host/wasm_webgl_glyphs-results.json');
  const saved = resolve(out, name + '.json');
  await copyFile(src, saved);
  return {result, receipt: JSON.parse(await readFile(saved, 'utf8'))};
}
try {
  await writeFile(file, mutant);
  const native = await run('negative-native', 'cargo', ['test', '-j2', '-p', 'xterm-web', '--test', 'webgl_glyph_backgrounds']);
  const log = await readFile(resolve(out, 'negative-native.log'), 'utf8');
  if (native.exitCode !== 101 || !log.includes('0 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out')) {
    throw Error('Expected exactly one native assertion failure');
  }
  const negative = await browser('negative-browser', 'chromium');
  const counts = negative.receipt.browsers[0]?.counts;
  if (negative.result.exitCode !== 1 || counts?.passed !== 7 || counts?.failed !== 1 || counts?.ignored !== 0 || counts?.filtered !== 0) {
    throw Error('Expected exactly 7 passing GPU tests and one signed-clipping failure');
  }
} finally {
  await writeFile(file, original);
  report.restored = hash(await readFile(file));
  await writeFile(resolve(out, 'negative-control.json'), JSON.stringify(report, null, 2) + '\n');
  if (report.restored !== before) throw Error('Production source was not restored byte-identically');
}
const native = await run('restored-native', 'cargo', ['test', '-j2', '-p', 'xterm-web', '--test', 'webgl_glyph_backgrounds']);
if (native.exitCode !== 0) throw Error('Restored native regression failed');
const fixed = await browser('final-browser', 'chromium,firefox,webkit');
if (fixed.result.exitCode !== 0 || fixed.receipt.browsers.length !== 3 || fixed.receipt.browsers.some(b =>
  b.counts?.passed !== 8 || b.counts?.failed !== 0 || b.counts?.ignored !== 0 || b.counts?.filtered !== 0)) {
  throw Error('Expected all eight GPU assertions to pass in all three browsers');
}
report.finalSource = hash(await readFile(file));
if (report.finalSource !== before) throw Error('Final source changed unexpectedly');
await writeFile(resolve(out, 'negative-control.json'), JSON.stringify(report, null, 2) + '\n');
console.log('Native negative1FAIL, GPU negative7PASS1FAIL, exactrestore, finalnative1PASS/GPU8PASSx3');
