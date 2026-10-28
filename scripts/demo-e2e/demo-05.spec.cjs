const { test, expect, fresh, click, change, snapshot } = require('./support.cjs');

test('live scalar and nested options update the visible terminal', async ({ page }) => {
  await page.locator('details').filter({ has: page.locator('#options') }).locator('summary').click();
  for (const [name, value] of Object.entries({ cursorStyle: 'bar', cursorInactiveStyle: 'underline', cursorBlink: true,
    fontFamily: 'monospace', fontSize: 19, fontWeight: 'bold', fontWeightBold: 'normal', lineHeight: 1.3, letterSpacing: 2 })) {
    await change(page, `opt-${name}`, value);
    await expect.poll(() => page.evaluate(name => demo.terminal.options[name], name)).toBe(value);
  }
  const initial = await page.evaluate(() => ({ ...demo.terminal.options.vtExtensions }));
  for (const name of ['kittyKeyboard', 'kittySgrBoldFaintControl', 'win32InputMode']) {
    await change(page, `opt-vtExtensions-${name}`, !initial[name]);
    initial[name] = !initial[name];
    expect(await page.evaluate(() => ({ ...demo.terminal.options.vtExtensions }))).toEqual(initial);
  }
  await change(page, 'opt-scrollbar-showArrows', true);
  expect(await page.evaluate(() => demo.terminal.options.scrollbar.showArrows)).toBe(true);
  const previous = await page.evaluate(() => demo.terminal.options.scrollback);
  await change(page, 'opt-scrollback', -1);
  await expect(page.locator('#error')).toBeVisible();
  expect(await page.evaluate(() => demo.terminal.options.scrollback)).toBe(previous);
  await expect(page.locator('.xterm-rows')).toContainText('Rust terminal');
});

test('themes and transparency change rendered pixels without replacing state', async ({ page }, info) => {
  const before = await snapshot(page);
  const pixels = await page.locator('.xterm-screen').screenshot();
  await change(page, 'theme', 'light');
  await expect(page.locator('#terminal-container')).toHaveCSS('background-color', 'rgb(255, 255, 255)');
  const after = await page.locator('.xterm-screen').screenshot();
  expect(pixels.equals(after), 'The theme must change actual rendered pixels').toBe(false);
  expect(await snapshot(page)).toEqual(before);
  await info.attach('light-theme', { body: after, contentType: 'image/png' });
  await page.locator('details').filter({ has: page.locator('#options') }).locator('summary').click();
  await change(page, 'opt-allowTransparency', true);
  await expect(page.locator('#terminal-container')).toHaveCSS('background-color', 'rgba(0, 0, 0, 0)');
});

test('explicit size, invalid auto-size, font and padding use real geometry', async ({ page }) => {
  await page.locator('#size').fill('60x16');
  await page.locator('#size-form button').click();
  await expect.poll(() => page.evaluate(() => [demo.terminal.cols, demo.terminal.rows])).toEqual([60, 16]);
  await expect(page.locator('.xterm-screen')).toHaveCSS('outline-style', 'solid');
  await page.locator('#size').fill('invalid');
  await page.locator('#size-form button').click();
  await expect(page.locator('#size')).toHaveValue('auto');
  await expect(page.locator('.xterm-screen')).toHaveCSS('outline-style', 'none');
  const geometry = await page.locator('.xterm-screen').boundingBox();
  const cols = await page.evaluate(() => demo.terminal.cols);
  await change(page, 'padding', 40);
  await expect.poll(() => page.evaluate(() => demo.terminal.cols)).toBeLessThan(cols);
  await page.locator('details').filter({ has: page.locator('#options') }).locator('summary').click();
  await change(page, 'opt-fontSize', 24);
  await expect.poll(() => page.locator('.xterm-screen').boundingBox()).not.toEqual(geometry);
  expect(await page.evaluate(() => demo.terminal.cols)).toBeGreaterThan(0);
});

test('search flags, wrapped Unicode ranges, direction and failed search are live', async ({ page }) => {
  await fresh(page, '\x1b[31mRust 界😀\x1b[0m\r\nrust test\r\nRust Rusty\r\n123456789Rust', 12, 8);
  await page.locator('#search-text').fill('Rust');
  await change(page, 'search-case', true);
  await change(page, 'search-word', true);
  await click(page, 'find-next');
  let result = JSON.parse(await page.locator('#search-results').textContent());
  expect(result.found).toBe(true); expect(result.selection).toBe('Rust');
  expect(result.result.resultCount).toBe(2);
  const first = await page.evaluate(() => demo.terminal.getSelectionPosition());
  expect(first).toEqual({ start: { x: 0, y: 0 }, end: { x: 4, y: 0 } });
  await click(page, 'find-next');
  expect(await page.evaluate(() => demo.terminal.getSelectionPosition())).toEqual({ start: { x: 0, y: 2 }, end: { x: 4, y: 2 } });
  await click(page, 'find-previous');
  expect(await page.evaluate(() => demo.terminal.getSelectionPosition())).toEqual(first);
  await change(page, 'search-word', false);
  await change(page, 'search-regex', true);
  await page.locator('#search-text').fill('9Rust');
  await click(page, 'find-next');
  expect(await page.evaluate(() => demo.terminal.getSelection())).toBe('9Rust');
  expect(await page.evaluate(() => demo.terminal.getSelectionPosition())).toEqual({ start: { x: 8, y: 3 }, end: { x: 1, y: 4 } });
  await page.locator('#search-text').fill('does-not-exist');
  await click(page, 'find-next');
  result = JSON.parse(await page.locator('#search-results').textContent());
  expect(result.found).toBe(false); expect(result.result.resultCount).toBe(0);
});

test('ANSI serialization write-back preserves complete cells, wraps and cursor', async ({ page }) => {
  await fresh(page, '\x1b[1;3;4;38;2;255;0;0mRust 界😀e\u0301 abcdefghijklmnop\x1b[0m\r\nsecond\x1b[2D', 16, 8);
  const before = await snapshot(page);
  await page.locator('#write-back').check();
  await click(page, 'serialize');
  const output = JSON.parse(await page.locator('#serialize-output').textContent());
  expect(output).toContain('Rust'); expect(output).toContain('\x1b[');
  expect(await snapshot(page)).toEqual(before);
});

test('HTML serialization reaches the real clipboard or reports host denial', async ({ page, context, browserName }) => {
  if (browserName === 'chromium') await context.grantPermissions(['clipboard-read', 'clipboard-write']);
  await fresh(page, '\x1b[38;2;255;0;0mHTML 界😀\x1b[0m');
  await page.locator('#serialize-html').click();
  await expect(page.locator('#html-output')).toContainText('HTML');
  const html = await page.locator('#html-output').textContent();
  expect(html).toContain('<'); expect(html).toContain('255');
  await expect.poll(() => page.evaluate(() => demo.events.some(event => event.name === 'serialize:html-copied') || demo.errors.length > 0)).toBe(true);
  const copied = await page.evaluate(() => demo.events.some(event => event.name === 'serialize:html-copied'));
  if (browserName === 'chromium') expect(copied, 'Granted Chromium clipboard permission must copy actual HTML').toBe(true);
  if (copied) {
    const actual = await page.evaluate(async () => {
      const items = await navigator.clipboard.read();
      const item = items.find(item => item.types.includes('text/html'));
      if (!item) throw new Error('HTML clipboard item is missing');
      return (await item.getType('text/html')).text();
    });
    expect(actual).toContain('HTML'); expect(actual).toContain('界😀');
  } else {
    await expect(page.locator('#error')).toBeVisible();
    expect(await page.evaluate(() => demo.errors.at(-1))).toMatch(/clipboard|copy|permission|denied|allowed/i);
  }
});

test('DOM and WebGL controls replace actual renderer ownership and retain selection', async ({ page }) => {
  await fresh(page, 'renderer 界😀');
  await page.evaluate(() => demo.terminal.select(0, 0, 8));
  const before = await snapshot(page);
  const canvases = await page.locator('.xterm canvas').count();
  await change(page, 'renderer', 'webgl');
  await expect.poll(() => page.locator('.xterm canvas').count()).toBeGreaterThan(canvases);
  expect(await snapshot(page)).toEqual(before);
  expect(await page.evaluate(() => demo.terminal.getSelection())).toBe('renderer');
  await change(page, 'renderer', 'dom');
  await expect.poll(() => page.locator('.xterm canvas').count()).toBe(canvases);
  await expect(page.locator('.xterm-rows')).toContainText('renderer');
  expect(await snapshot(page)).toEqual(before);
});
