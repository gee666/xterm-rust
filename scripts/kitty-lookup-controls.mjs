// Licensed under MIT. Source-only comparator negative controls, not Rust coverage.
import assert from 'node:assert/strict';
import { createHash } from 'node:crypto';
import { mkdirSync, readFileSync, writeFileSync } from 'node:fs';
import { stripTypeScriptTypes } from 'node:module';
import { resolve } from 'node:path';
import { compareKittyLookups } from '../crates/xterm-web/tests/fixtures/kitty_lookup_cases.mjs';
const root = resolve(import.meta.dirname, '..');
const read = name => readFileSync(resolve(root, name), 'utf8');
const erase = text => stripTypeScriptTypes(text.replace(/^import .*;\r?$/gm, '').replace(/^export /gm, ''), { mode: 'transform' });
const enumeration = read('source_repo/src/common/Types.ts').match(/export const enum KeyboardResultType\s*\{[^}]*\}/)?.[0];
assert.ok(enumeration);
const raw = read('source_repo/src/common/input/KittyKeyboard.ts');
const prefix = erase(enumeration) + '\n' + erase(read('source_repo/src/common/data/EscapeSequences.ts')) + '\n';
const make = source => new Function(prefix + erase(source) + '\nreturn KittyKeyboard;')();
const Source = make(raw);
const fields = ['_functionalKeyCodes', '_csiTildeKeys', '_csiLetterKeys', '_ss3FunctionKeys'];
class NoPrototype extends Source {
  constructor() { super(); for (const field of fields) Object.setPrototypeOf(this[field], null); }
}
class Lossy extends Source {
  evaluate(...args) {
    const value = super.evaluate(...args);
    if (value.key !== undefined) value.key = value.key.replace(/[\ud800-\udfff]/g, '\ufffd');
    return value;
  }
}
class ReplacedError extends Source {
  evaluate(...args) {
    try { return super.evaluate(...args); } catch { throw new Error('replacement'); }
  }
}
const hintMutation = raw.replaceAll("+ letter", "+ String(letter)").replaceAll("seq += letter", "seq += String(letter)");
assert.notEqual(hintMutation, raw);
const numericMutation = raw.replaceAll('keyCode ===', 'Number(keyCode) ===');
assert.notEqual(numericMutation, raw);
const controls = [['no-prototype', NoPrototype], ['string-hint', make(hintMutation)],
  ['numeric-coercion', make(numericMutation)], ['lossy-utf16', Lossy], ['replaced-exceptions', ReplacedError]];
const baseline = compareKittyLookups(Source, Source);
assert.equal(baseline.failures.length, 0, 'The comparator must agree with itself');
const results = controls.map(([name, Mutated]) => {
  const result = compareKittyLookups(Source, Mutated);
  assert.ok(result.failures.length > 0, 'Comparator missed ' + name);
  return { name, cases: result.cases, detected: result.failures.length, firstFailure: result.failures[0] };
});
const directory = resolve(root, 'results/kitty-realm');
mkdirSync(directory, { recursive: true });
const report = { scope: 'Source-only deliberate faults; not Rust coverage', node: process.version,
  sourceSha256: createHash('sha256').update(raw).digest('hex'), baselineCases: baseline.cases, results };
writeFileSync(resolve(directory, 'negative-controls.json'), JSON.stringify(report, null, 2) + '\n');
console.log(results.map(result => result.name + ': ' + result.detected).join('\n'));
