const { test, expect, click, snapshot, fresh } = require('./support.cjs');
const { probe } = require('./addon-probes.cjs');

const addons = ['attach', 'clipboard', 'fit', 'image', 'ligatures', 'program-status', 'progress', 'search',
  'serialize', 'unicode-graphemes', 'unicode11', 'web-fonts', 'web-links', 'webgl'];

for (const name of addons) test(`${name} load, reload, behavior and unload use the visible session`, async ({ page, context, browserName }, info) => {
  if (name === 'clipboard' && browserName === 'chromium') await context.grantPermissions(['clipboard-read', 'clipboard-write']);
  const previousVersion = name.startsWith('unicode') ? await page.evaluate(() => demo.terminal.unicode.activeVersion) : undefined;
  const initialCanvases = await page.locator('.xterm canvas').count();
  if (name === 'ligatures') await page.evaluate(() => {
    window.joinerCalls = [];
    const terminal = demo.terminal, register = terminal.registerCharacterJoiner;
    terminal.registerCharacterJoiner = handler => register.call(terminal, text => {
      const ranges = handler(text); window.joinerCalls.push({ text, ranges }); return ranges;
    });
  });
  await page.locator('#addon-select').selectOption(name);
  await page.locator('#socket-url').fill(new URL('/socket', page.url()).href.replace('http:', 'ws:'));
  await click(page, 'addon-load');
  expect(await page.evaluate(() => demo.snapshot().addons)).toContain(name);
  await probe(page, name, 'first');
  await info.attach(`${name}-loaded`, { body: await page.locator('.xterm').screenshot(), contentType: 'image/png' });
  await page.evaluate(name => {
    window.oldAddon = demo.addons.get(name);
    window.oldSocket = demo.addons.loaded.get(name).socket;
  }, name);
  await click(page, 'addon-reload');
  expect(await page.evaluate(name => demo.addons.get(name) !== window.oldAddon, name)).toBe(true);
  if (name === 'attach') await expect.poll(() => page.evaluate(() => window.oldSocket.readyState)).toBe(3);
  await probe(page, name, 'second');
  const inputBefore = await page.evaluate(() => demo.outgoing.length);
  await page.locator('.xterm-helper-textarea').press('a');
  await expect.poll(() => page.evaluate(before => demo.outgoing.slice(before), inputBefore)).toEqual(['a']);
  await click(page, 'addon-unload');
  expect(await page.evaluate(() => demo.snapshot().addons)).not.toContain(name);
  if (name === 'webgl' || name === 'image') await expect.poll(() => page.locator('.xterm canvas').count()).toBe(initialCanvases);
  if (name === 'unicode11' || name === 'unicode-graphemes') {
    expect(await page.evaluate(() => demo.terminal.unicode.activeVersion)).toBe(previousVersion);
  }
  if (name === 'program-status' || name === 'progress' || name === 'clipboard') {
    const before = await page.evaluate(() => ({ events: demo.events.length, outgoing: demo.outgoing.length }));
    await page.evaluate(name => demo.write(name === 'program-status' ? '\x1b]7501;?\x07'
      : name === 'progress' ? '\x1b]9;4;1;89\x07' : '\x1b]52;c;?\x07'), name);
    const after = await page.evaluate(before => ({
      changes: demo.events.slice(before.events).filter(event => event.name.startsWith('addon:')).length,
      outgoing: demo.outgoing.slice(before.outgoing)
    }), before);
    expect(after).toEqual({ changes: 0, outgoing: [] });
  }
  await fresh(page, 'after unload 界😀');
  await expect(page.locator('.xterm-rows')).toContainText('after unload');
  const count = await page.evaluate(() => demo.outgoing.length);
  await page.locator('.xterm-helper-textarea').press('z');
  await expect.poll(() => page.evaluate(count => demo.outgoing.slice(count), count)).toEqual(['z']);
});

for (const order of [['fit', 'search', 'serialize'], ['serialize', 'search', 'fit']]) {
  test(`addon combination ${order.join('/')} survives dispose and reopen`, async ({ page }, info) => {
    for (const name of order) {
      await page.locator('#addon-select').selectOption(name); await click(page, 'addon-load');
    }
    await fresh(page, '\x1b[31mRust combined 界😀\x1b[0m');
    expect(await page.evaluate(() => demo.addons.get('search').findNext('combined'))).toBe(true);
    expect(await page.evaluate(() => demo.terminal.getSelection())).toBe('combined');
    expect(await page.evaluate(() => demo.addons.get('serialize').serialize())).toContain('combined');
    await page.evaluate(() => { window.retiredTerminal = demo.terminal; window.retiredAddons = demo.addons; });
    await click(page, 'dispose');
    await expect(page.locator('.xterm')).toHaveCount(0);
    await click(page, 'reopen');
    await expect(page.locator('.xterm')).toHaveCount(1);
    expect(await page.evaluate(() => demo.snapshot().addons)).toEqual([]);
    const state = await snapshot(page);
    const stale = await page.evaluate(async () => {
      const errors = [];
      for (const [name, action] of [['write', () => retiredTerminal.write('STALE')], ['input', () => retiredTerminal.input('STALE', true)],
        ['addon-dispose', () => retiredAddons.dispose()]]) {
        try { action(); } catch (error) { errors.push({ name, message: String(error) }); }
      }
      await demo.write('');
      return errors;
    });
    await info.attach('stale-handle-results', { body: JSON.stringify(stale), contentType: 'application/json' });
    expect(await snapshot(page)).toEqual(state);
    for (const name of order) {
      await page.locator('#addon-select').selectOption(name); await click(page, 'addon-load');
    }
    await fresh(page, 'fresh combination');
    expect(await page.evaluate(() => demo.addons.get('search').findNext('fresh'))).toBe(true);
    const before = await page.evaluate(() => demo.outgoing.length);
    await page.locator('.xterm-helper-textarea').press('x');
    await expect.poll(() => page.evaluate(before => demo.outgoing.slice(before), before)).toEqual(['x']);
  });
}
