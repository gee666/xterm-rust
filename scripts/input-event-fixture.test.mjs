// Licensed under MIT. Keep the browser oracle's copied method body unchanged.
import assert from 'node:assert/strict';
import { createHash } from 'node:crypto';
import { readFile } from 'node:fs/promises';
import test from 'node:test';

const root = new URL('../', import.meta.url);
test('InputEvent oracle body matches the supplied upstream source', async () => {
  const source = await readFile(new URL('source_repo/src/browser/CoreBrowserTerminal.ts', root), 'utf8');
  const fixture = await readFile(new URL('crates/xterm-web/tests/fixtures/input-event-body.js', root), 'utf8');
  const body = source.replaceAll('\r\n', '\n').match(
    /protected _inputEvent\(ev: InputEvent\): boolean \{([\s\S]*?)\n  \}/)?.[1];
  assert.notEqual(body, undefined, 'upstream input method boundaries changed');
  assert.equal(fixture.replaceAll('\r\n', '\n').trim(), body.trim());
  console.log(`CoreBrowserTerminal.ts sha256=${createHash('sha256').update(source).digest('hex')}`);
  console.log(`input-event-body.js sha256=${createHash('sha256').update(fixture).digest('hex')}`);
});
