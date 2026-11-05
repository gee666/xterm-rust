// Licensed under MIT. Source-only negative controls, not Rust coverage.
import assert from 'node:assert/strict';
import { createHash } from 'node:crypto';
import { mkdir, readFile, writeFile } from 'node:fs/promises';
import { stripTypeScriptTypes } from 'node:module';
import { resolve } from 'node:path';
import vm from 'node:vm';
import { compareIndependentWin32 } from '../crates/xterm-web/tests/fixtures/win32_independent_cases.mjs';

const root = resolve(import.meta.dirname, '..');
const sourceFiles = ['src/common/Types.ts', 'src/common/data/EscapeSequences.ts',
  'src/common/input/Win32InputMode.ts'];
const sources = await Promise.all(sourceFiles.map(path => readFile(resolve(root, 'source_repo', path))));
const strip = text => stripTypeScriptTypes(text.replace(/^import .*;\r?$/gm, '')
  .replace(/^export /gm, ''), { mode: 'transform' });
const resultEnum = sources[0].toString().match(/export const enum KeyboardResultType\s*\{[^}]*\}/)?.[0];
assert.ok(resultEnum);
const Source = vm.runInNewContext([strip(resultEnum), strip(sources[1].toString()),
  strip(sources[2].toString()), 'Win32InputMode'].join('\n'), {}, { timeout: 5000 });
const identity = compareIndependentWin32(Source, Source);
assert.equal(identity.cases, 348);
assert.deepEqual(identity.failures, []);

class ConstantOutput extends Source {
  evaluateKeyboardEvent() { return { type: 0, cancel: true, key: '\x1b[0;0;0;1;0;1_' }; }
}
class EarlyLookups extends Source {
  evaluateKeyboardEvent(event, down) {
    for (const field of ['_codeToVk', '_codeToScancode', '_keyToControlChar']) this[field] = { ...this[field] };
    return super.evaluateKeyboardEvent(event, down);
  }
}
class NumericCoercion extends Source {
  evaluateKeyboardEvent(event, down) {
    const result = super.evaluateKeyboardEvent(event, down);
    result.key = result.key.replace(/^\x1b\[([^;]*);([^;]*);([^;]*);/,
      (_, vk, sc, uc) => `\x1b[${Number(vk)};${Number(sc)};${Number(uc)};`);
    return result;
  }
}
class LossyUtf16 extends Source {
  evaluateKeyboardEvent(event, down) {
    const result = super.evaluateKeyboardEvent(event, down);
    result.key = result.key.toWellFormed();
    return result;
  }
}
class ReplacedException extends Source {
  evaluateKeyboardEvent(event, down) {
    try { return super.evaluateKeyboardEvent(event, down); }
    catch { throw new Error('Replaced original thrown value'); }
  }
}
const controls = [];
for (const Mutated of [ConstantOutput, EarlyLookups, NumericCoercion, LossyUtf16, ReplacedException]) {
  const result = compareIndependentWin32(Source, Mutated);
  assert.ok(result.failures.length > 0, `${Mutated.name} escaped detection`);
  controls.push({ mutation: Mutated.name, detected: result.failures.length,
    firstCounterexample: result.failures[0] });
}
const output = resolve(root, 'results/win32-independent');
await mkdir(output, { recursive: true });
const report = { scope: 'Source-only harness negative controls. Not Rust execution.',
  node: process.version, platform: process.platform, cases: identity.cases, controls,
  source: sources.map((bytes, index) => ({ path: sourceFiles[index],
    sha256: createHash('sha256').update(bytes).digest('hex') })) };
await writeFile(resolve(output, 'negative-controls.json'), JSON.stringify(report, null, 2) + '\n');
console.log('Win32 independent harness detects all five deliberate mutations.');
