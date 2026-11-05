// Licensed under MIT. Verify copied method bodies against the read-only source.
import assert from 'node:assert/strict';
import { createHash } from 'node:crypto';
import { readFile } from 'node:fs/promises';
import test from 'node:test';
const root = new URL('../', import.meta.url);
test('browser platform selectors match the supplied source', async () => {
  const source = await readFile(new URL('source_repo/src/common/Platform.ts', root), 'utf8');
  const fixture = await readFile(new URL('crates/xterm-web/tests/fixtures/keypress-cases.js', root), 'utf8');
  for (const name of ['isMac', 'isWindows']) {
    const expression = source.match(new RegExp('export const ' + name + ' = ([^;]+);'))?.[1];
    assert.ok(expression);
    assert.ok(fixture.includes(name + ': ' + expression.replace('includes(platform)', 'includes(navigator.platform)')));
  }
  console.log('Platform.ts sha256=' + createHash('sha256').update(source).digest('hex'));
});
for (const [signature, name] of [
  ['protected _keyPress(ev: KeyboardEvent): boolean {', 'keypress-body.js'],
  ['private _isThirdLevelShift(browser: IBrowser, ev: KeyboardEvent): boolean {', 'third-level-shift-body.js'],
]) {
  test(`${name} matches unchanged source`, async () => {
    const source = await readFile(new URL('source_repo/src/browser/CoreBrowserTerminal.ts', root), 'utf8');
    const normalized = source.replaceAll('\r\n', '\n');
    const start = normalized.indexOf(signature);
    assert.ok(start >= 0);
    const body = start + signature.length;
    const end = normalized.indexOf('\n  }', body);
    assert.ok(end > body);
    const fixture = await readFile(new URL(`crates/xterm-web/tests/fixtures/${name}`, root), 'utf8');
    assert.equal(fixture.replaceAll('\r\n', '\n').trim(), normalized.slice(body, end).trim());
    console.log(`source sha256=${createHash('sha256').update(source).digest('hex')}`);
    console.log(`${name} sha256=${createHash('sha256').update(fixture).digest('hex')}`);
  });
}
