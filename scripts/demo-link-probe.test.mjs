// The real JS/Rust browser checks establish semantics. These unit controls only
// keep the demo probe's physical pointer path and popup assertions intact.
import assert from 'node:assert/strict';
import { readFile } from 'node:fs/promises';
import { createRequire } from 'node:module';
import { test } from 'node:test';
import { runInNewContext } from 'node:vm';
const require = createRequire(import.meta.url);
const code = await readFile(new URL('./demo-e2e/addon-probes.cjs', import.meta.url), 'utf8');

for (const initialCell of [undefined, 1]) {
  test(`link probe enters another row before target with cached row ${initialCell}`, async () => {
    let lastRow = initialCell, pointer = false, pendingPopup;
    const moves = [], popups = [], checks = [];
    const popup = { async waitForURL(url) { popups.push(url); }, async close() { popups.push('closed'); } };
    const page = {
      context() { return { async route(pattern) { assert.equal(pattern, 'https://example.com/**'); } }; },
      locator(selector) {
        return { selector, async scrollIntoViewIfNeeded() {},
          async boundingBox() { return { x: 10, y: 20, width: 400, height: 240 }; } };
      },
      async evaluate() { return [40, 12]; },
      waitForEvent(name) {
        assert.equal(name, 'popup');
        return new Promise(resolve => { pendingPopup = resolve; });
      },
      mouse: {
        async move(x, y) {
          moves.push([x, y]);
          const row = Math.floor((y - 20) / 20) + 1;
          if (row !== lastRow) pointer = row === 1;
          lastRow = row;
        },
        async click(x, y) {
          assert.deepEqual([x, y], [40, 30]);
          assert.equal(pointer, true, 'Real hover must precede activation');
          assert(pendingPopup, 'Popup listener must precede the click');
          pendingPopup(popup);
        }
      }
    };
    const support = {
      async fresh(_, text) { assert.equal(text, 'https://example.com/'); pointer = false; },
      expect(locator) { return { async toBeVisible() {
        assert.equal(locator.selector, '.xterm-cursor-pointer');
        checks.push(pointer);
        assert.equal(pointer, true, 'Hover cannot be omitted after reload');
      } }; }
    };
    const module = { exports: {} };
    runInNewContext(code, { module, exports: module.exports,
      require: name => name === './support.cjs' ? support : require(name) });
    await module.exports.probe(page, 'web-links', 'first');
    pointer = false;
    await module.exports.probe(page, 'web-links', 'second');
    assert.deepEqual(moves, [[40, 50], [40, 30], [40, 50], [40, 30]]);
    assert.deepEqual(checks, [true, true]);
    assert.deepEqual(popups, ['https://example.com/', 'closed', 'https://example.com/', 'closed']);
  });
}
