// Licensed under MIT. Keep the browser oracle's copied method bodies unchanged.
import assert from 'node:assert/strict';
import { createHash } from 'node:crypto';
import { readFile } from 'node:fs/promises';
import test from 'node:test';

const root = new URL('../', import.meta.url);
for (const [name, path, signature, fixtureName] of [
  ['InputEvent', 'src/browser/CoreBrowserTerminal.ts',
    'protected _inputEvent(ev: InputEvent): boolean {', 'input-event-body.js'],
  ['CompositionHelper keydown', 'src/browser/input/CompositionHelper.ts',
    'public keydown(ev: KeyboardEvent): boolean {', 'composition-keydown-body.js'],
]) {
  test(`${name} oracle body matches the supplied upstream source`, async () => {
    const source = await readFile(new URL(`source_repo/${path}`, root), 'utf8');
    const fixture = await readFile(new URL(`crates/xterm-web/tests/fixtures/${fixtureName}`, root), 'utf8');
    const normalized = source.replaceAll('\r\n', '\n');
    const start = normalized.indexOf(signature);
    assert.ok(start >= 0, 'upstream method signature changed');
    const bodyStart = start + signature.length;
    const end = normalized.indexOf('\n  }', bodyStart);
    assert.ok(end > bodyStart, 'upstream method boundary changed');
    assert.equal(fixture.replaceAll('\r\n', '\n').trim(), normalized.slice(bodyStart, end).trim());
    console.log(`${path} sha256=${createHash('sha256').update(source).digest('hex')}`);
    console.log(`${fixtureName} sha256=${createHash('sha256').update(fixture).digest('hex')}`);
  });
}
