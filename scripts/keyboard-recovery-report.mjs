// Licensed under MIT. Retain raw keyboard evidence without hiding other failures.
// Adapted from the historical Kitty realm report, not its obsolete pass/fail rules.
import assert from 'node:assert/strict';
import { createHash } from 'node:crypto';
import { copyFileSync, mkdirSync, readFileSync, readdirSync, writeFileSync } from 'node:fs';
import { dirname, resolve } from 'node:path';

const root = resolve(import.meta.dirname, '..');
const out = resolve(root, 'results/keyboard-recovery343');
mkdirSync(out, { recursive: true });
writeFileSync(resolve(out, 'verification.json'), JSON.stringify({
  scope: 'Task343 raw keyboard recovery', rawKeyboardStatus: 'preparing'
}) + '\n');
const read = path => readFileSync(resolve(root, path));
const json = path => JSON.parse(read(path));
const hash = path => createHash('sha256').update(read(path)).digest('hex');
const receipts = {};
for (const name of ['lib', 'wasm_keyboard_live', 'wasm_input_event']) {
  const from = `results/browser-host/${name}-results.json`;
  const receipt = receipts[name] = json(from);
  copyFileSync(resolve(root, from), resolve(out, `${name}-results.json`));
  for (const browser of receipt.browsers) {
    assert.deepEqual(browser.errors, [], `${name}/${browser.name} unexpected page errors`);
    const log = resolve(dirname(receipt.artifacts.bindings.path), '..', `${name}-${browser.name}.log`);
    copyFileSync(log, resolve(out, `${name}-${browser.name}.log`));
  }
  assert.deepEqual(receipt.browsers.map(browser => browser.name), ['chromium', 'firefox', 'webkit']);
}
const rawTests = [
  'kitty_raw_lookup_values_match_source_in_this_realm',
  'win32_prototype_values_and_coercion_match_source_in_this_browser_realm'
];
for (const browser of receipts.lib.browsers) {
  for (const name of rawTests) assert.ok(browser.output.includes(name + ' ... ok'), name);
  assert.ok(browser.console.some(entry => entry.text === 'Kitty realm comparisons: 10224, failures: 0'));
  assert.ok(browser.console.some(entry => entry.text === 'Win32 comparison: 212 retained and 380 independent cases'));
}
for (const [name, count] of [['wasm_keyboard_live', 15], ['wasm_input_event', 4]]) {
  assert.equal(receipts[name].status, 'passed', name);
  for (const browser of receipts[name].browsers) {
    assert.deepEqual(browser.counts, { passed: count, failed: 0, ignored: 0, filtered: 0 });
  }
}
const nodeRoot = 'results/kitty-realm/node';
const nodeRun = readdirSync(resolve(root, nodeRoot)).sort().at(-1);
assert.ok(nodeRun);
const nodeReceipt = `${nodeRoot}/${nodeRun}/verification.json`;
const node = json(nodeReceipt);
assert.equal(node.status, 'passed');
assert.equal(node.kittyCases, 10224);
assert.equal(node.win32RetainedCases, 212);
assert.equal(node.win32IndependentCases, 380);
copyFileSync(resolve(root, nodeReceipt), resolve(out, 'node-results.json'));
copyFileSync(resolve(root, nodeRoot, nodeRun, 'node.log'), resolve(out, 'node.log'));
const baseline = json('docs/baseline/source-manifest.json');
const source = ['src/common/Types.ts', 'src/common/data/EscapeSequences.ts',
  'src/common/input/KittyKeyboard.ts', 'src/common/input/KittyKeyboard.test.ts',
  'src/common/input/Win32InputMode.ts', 'src/common/input/Win32InputMode.test.ts'].map(path => {
  const sha256 = hash('source_repo/' + path);
  assert.equal(sha256, baseline.find(entry => entry.path === path)?.sha256, path + ' baseline drift');
  return { path, sha256 };
});
const implementation = [
  ...['keyboard_event_source.rs', 'keyboard_lookup.rs', 'keyboard_map_inherited.rs',
    'kitty_keyboard.rs', 'kitty_keyboard/lookup.rs', 'kitty_keyboard/live.rs',
    'kitty_keyboard/live_fields.rs', 'kitty_keyboard/keys.rs', 'win32_input_mode.rs',
    'win32_lookup.rs', 'win32_key_codes.rs'].map(path => 'crates/xterm-core/src/input/' + path),
  ...['keyboard_lookup.rs', 'win32_lookup.rs', 'win32_tables.js', 'kitty_tables.js',
    'dom/key_event.rs', 'dom/keyboard.rs', 'dom/mod.rs'].map(path => 'crates/xterm-web/src/input/' + path),
  ...['kitty_lookup_cases.mjs', 'kitty_lookup_driver.js', 'win32-prototype-cases.mjs',
    'win32_independent_cases.mjs', 'win32_prototype_driver.js'].map(path => 'crates/xterm-web/tests/fixtures/' + path)
].map(path => ({ path, sha256: hash(path) }));
const report = {
  scope: 'Task343 raw Kitty/Win32 lookup recovery. Full input and terminal equivalence NOT PROVEN.',
  node: process.version, platform: process.platform, source, implementation,
  rawKeyboardStatus: 'passed', kittyCasesPerRealm: 10224, win32CasesPerRealm: 592,
  win32Families: { retainedRawMaps: 212, independentRawMaps: 348, changingEmptyCodepoint: 32 },
  nodeReceipt, nodeWasmSha256: node.wasmSha256,
  libraries: receipts.lib.browsers.map(({ name, version, status, counts, output }) => ({
    name, version, status, counts, rawKeyboardTestsPassed: 2,
    failures: output.split('\n').filter(line => line.endsWith(' ... FAIL'))
  })),
  dom: receipts.wasm_keyboard_live.browsers.map(({ name, version, status, counts }) => ({ name, version, status, counts })),
  inputEvent: receipts.wasm_input_event.browsers.map(({ name, version, status, counts }) => ({ name, version, status, counts })),
  artifacts: Object.fromEntries(Object.entries(receipts).map(([name, receipt]) => [name, receipt.artifacts])),
  unproven: ['Full input/accessibility family319 including admitted nonstring InputEvent',
    'Complete unchanged browser/terminal/addon suites', 'Native Linux/macOS runtime',
    'Final workspace feature/release/platform matrix', 'Canonical main integration']
};
writeFileSync(resolve(out, 'verification.json'), JSON.stringify(report, null, 2) + '\n');
console.log(JSON.stringify({ rawKeyboardStatus: report.rawKeyboardStatus,
  libraryStatuses: report.libraries.map(({ name, status, counts }) => ({ name, status, counts })),
  domTestsPerEngine: 15, inputEventTestsPerEngine: 4 }));
