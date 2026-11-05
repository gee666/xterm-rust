// Licensed under MIT. Keep the IME oracle method bodies unchanged.
import assert from 'node:assert/strict';
import { createHash } from 'node:crypto';
import { readFile } from 'node:fs/promises';
import test from 'node:test';
const root = new URL('../', import.meta.url);
for (const [name, signature] of [
  ['start', 'public compositionstart(): void {'],
  ['keydown', 'public keydown(ev: KeyboardEvent): boolean {'],
  ['textarea', 'private _handleAnyTextareaChanges(): void {'],
  ['end', 'public compositionend(): void {'],
  ['finalize', 'private _finalizeComposition(waitForPropagation: boolean): void {'],
]) {
  test(`composition ${name} matches the unchanged reference body`, async () => {
    const raw = await readFile(new URL('source_repo/src/browser/input/CompositionHelper.ts', root), 'utf8');
    const source = raw.replaceAll('\r\n', '\n');
    const offset = source.indexOf(signature);
    assert.ok(offset >= 0, 'reference signature must exist');
    const start = offset + signature.length;
    const end = source.indexOf('\n  }', start);
    assert.ok(end > start);
    const fixture = await readFile(new URL(`crates/xterm-web/tests/fixtures/composition-${name}-body.js`, root), 'utf8');
    assert.equal(fixture.replaceAll('\r\n', '\n').trim(), source.slice(start, end).trim());
    console.log(`CompositionHelper.ts sha256=${createHash('sha256').update(raw).digest('hex')}`);
    console.log(`${name} sha256=${createHash('sha256').update(fixture).digest('hex')}`);
  });
}
