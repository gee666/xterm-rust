const { test, expect } = require('../../source_repo/node_modules/@playwright/test');
const evidence = new WeakMap();

test.beforeEach(async ({ page, browser }, info) => {
  const record = { browser: browser.version(), errors: [], console: [], failedRequests: [], wasm: [] };
  evidence.set(page, record);
  page.on('pageerror', error => record.errors.push(String(error.stack ?? error)));
  page.on('console', message => { if (message.type() === 'error') record.console.push(message.text()); });
  page.on('requestfailed', request => record.failedRequests.push({ url: request.url(), error: request.failure() }));
  page.on('response', response => {
    if (response.url().endsWith('.wasm')) record.wasm.push({ url: response.url(), status: response.status() });
    if (response.status() >= 400) record.failedRequests.push({ url: response.url(), status: response.status() });
  });
  await page.route(/source_repo|client-bundle\.js|\/lib\/xterm\./, route => route.abort('blockedbyclient'));
  await page.goto('/demo/index.html');
  await page.waitForFunction(() => window.demo);
  await page.evaluate(() => window.demo.ready);
  await expect(page.locator('#status')).toHaveText('Rust terminal ready');
  await expect(page.locator('.xterm')).toHaveCount(1);
  await expect(page.locator('.xterm-rows')).toContainText('Rust terminal');
  expect(record.wasm.length, 'The demo must instantiate the fetched Rust WASM').toBeGreaterThan(0);
  expect(record.wasm.every(response => response.status === 200)).toBe(true);
  await info.attach('initial-state', { body: JSON.stringify(await page.evaluate(() => demo.snapshot())), contentType: 'application/json' });
});

test.afterEach(async ({ page }, info) => {
  const record = evidence.get(page);
  try {
    if (!page.isClosed()) {
      const state = await page.evaluate(() => window.demo ? {
        state: demo.terminal ? demo.snapshot() : null, errors: demo.errors, events: demo.events, outgoing: demo.outgoing
      } : null);
      await info.attach('demo-state', { body: JSON.stringify(state, null, 2), contentType: 'application/json' });
      if (state) {
        await page.evaluate(() => demo.close());
        await expect(page.locator('.xterm')).toHaveCount(0);
      }
      if (info.status === info.expectedStatus) {
        expect(record.errors, 'Uncaught browser exceptions').toEqual([]);
        expect(record.console, 'Browser console errors').toEqual([]);
        expect(record.failedRequests, 'Missing or forbidden runtime assets').toEqual([]);
      }
    }
  } finally {
    await info.attach('browser-runtime', { body: JSON.stringify(record, null, 2), contentType: 'application/json' });
  }
});

async function fresh(page, text, cols = 40, rows = 12) {
  await page.evaluate(async ({ text, cols, rows }) => {
    demo.terminal.reset(); demo.terminal.resize(cols, rows);
    await demo.write(text);
  }, { text, cols, rows });
}
async function click(page, id) {
  const before = await page.evaluate(() => demo.events.length);
  await page.locator(`#${id}`).click();
  await expect.poll(() => page.evaluate(({ id, before }) => {
    const events = demo.events.slice(before);
    if (events.some(event => event.name === 'error')) throw new Error(demo.errors.at(-1));
    return events.some(event => event.name === id && event.value === 'completed');
  }, { id, before })).toBe(true);
}
async function change(page, id, value) {
  const locator = page.locator(`#${id}`);
  await locator.scrollIntoViewIfNeeded();
  if (typeof value === 'boolean') await locator.setChecked(value);
  else if (await locator.evaluate(element => element.tagName === 'SELECT')) await locator.selectOption(String(value));
  else { await locator.fill(String(value)); await locator.dispatchEvent('change'); }
}
async function snapshot(page) {
  return page.evaluate(() => {
    const terminal = demo.terminal, buffer = terminal.buffer.active;
    const methods = ['getChars', 'getCode', 'getWidth', 'getFgColorMode', 'getBgColorMode', 'getFgColor', 'getBgColor',
      'isBold', 'isItalic', 'isDim', 'isUnderline', 'isBlink', 'isInverse', 'isInvisible', 'isStrikethrough', 'isOverline'];
    const lines = [];
    for (let row = 0; row < buffer.length; row++) {
      const line = buffer.getLine(row), cells = [];
      for (let column = 0; column < terminal.cols; column++) {
        const cell = line.getCell(column);
        cells.push(Object.fromEntries(methods.map(name => [name, cell[name]()])));
      }
      lines.push({ wrapped: line.isWrapped, cells });
    }
    return { cols: terminal.cols, rows: terminal.rows, x: buffer.cursorX, y: buffer.cursorY,
      baseY: buffer.baseY, viewportY: buffer.viewportY, type: buffer.type, lines };
  });
}
module.exports = { test, expect, fresh, click, change, snapshot };
