const { readFileSync } = require('node:fs');
const { resolve } = require('node:path');
const { expect, fresh } = require('./support.cjs');

async function probe(page, name, phase) {
  if (name === 'attach') {
    await expect.poll(() => page.evaluate(() => demo.addons.loaded.get('attach').socket.readyState)).toBe(1);
    await fresh(page, '');
    const text = `socket-${phase}`;
    await page.evaluate(text => demo.terminal.input(text, true), text);
    await expect.poll(() => page.evaluate(() => demo.terminal.buffer.active.getLine(0).translateToString(true))).toBe(text);
  } else if (name === 'fit') {
    await page.evaluate(() => demo.addons.get('fit').fit());
    expect(await page.evaluate(() => demo.addons.get('fit').proposeDimensions())).toEqual(
      await page.evaluate(() => ({ cols: demo.terminal.cols, rows: demo.terminal.rows })));
    const cols = await page.evaluate(() => demo.terminal.cols);
    await page.locator('#terminal-container').evaluate(element => { element.style.width = '300px'; });
    await page.evaluate(() => demo.addons.get('fit').fit());
    expect(await page.evaluate(() => demo.terminal.cols)).toBeLessThan(cols);
    await page.locator('#terminal-container').evaluate(element => { element.style.width = ''; });
  } else if (name === 'progress' || name === 'program-status') {
    const event = `addon:${name}:onChange`;
    const before = await page.evaluate(event => demo.events.filter(item => item.name === event).length, event);
    await page.evaluate(async name => {
      await demo.write(name === 'progress' ? '\x1b]9;4;1;42\x07' : '\x1b]7501;state=working:id=e2e:app=rust\x1b\\');
    }, name);
    expect(await page.evaluate(event => demo.events.filter(item => item.name === event).length, event)).toBe(before + 1);
    if (name === 'progress') expect(await page.evaluate(() => demo.addons.get('progress').progress)).toEqual({ state: 1, value: 42 });
    else expect(await page.evaluate(() => demo.addons.get('program-status').records)).toEqual([{ id: 'e2e', state: 'working', app: 'rust', effectiveApp: 'rust' }]);
  } else if (name === 'unicode11' || name === 'unicode-graphemes') {
    const version = name === 'unicode11' ? '11' : '15-graphemes';
    expect(await page.evaluate(() => demo.terminal.unicode.versions)).toContain(version);
    await page.evaluate(version => { demo.terminal.unicode.activeVersion = version; }, version);
    await fresh(page, 'A界😀');
    expect(await page.evaluate(() => demo.terminal.buffer.active.cursorX)).toBe(5);
    expect(await page.evaluate(() => [demo.terminal.buffer.active.getLine(0).getCell(1).getWidth(), demo.terminal.buffer.active.getLine(0).getCell(3).getWidth()])).toEqual([2, 2]);
    if (name === 'unicode-graphemes') {
      await fresh(page, '👩\u200d🚀');
      expect(await page.evaluate(() => demo.terminal.buffer.active.cursorX)).toBe(2);
    }
  } else if (name === 'search') {
    await fresh(page, `probe ${phase} 界😀`);
    expect(await page.evaluate(phase => demo.addons.get('search').findNext(phase), phase)).toBe(true);
    expect(await page.evaluate(() => demo.terminal.getSelection())).toBe(phase);
  } else if (name === 'serialize') {
    await fresh(page, '\x1b[31mserialize 界😀\x1b[0m');
    expect(await page.evaluate(() => demo.addons.get('serialize').serialize())).toContain('serialize 界😀');
    expect(await page.evaluate(() => demo.addons.get('serialize').serializeAsHTML())).toContain('serialize');
  } else if (name === 'image') {
    await fresh(page, '');
    await page.evaluate(async () => {
      window.imageEvents = 0;
      window.imageHandle?.dispose();
      window.imageHandle = demo.addons.get('image').onImageAdded(() => window.imageEvents++);
      await demo.write('\x1b_Ga=T,f=32,s=1,v=1,i=71;/wAA/w==\x1b\\');
    });
    await expect.poll(() => page.evaluate(() => window.imageEvents)).toBe(1);
    expect(await page.evaluate(() => demo.addons.get('image').storageUsage)).toBeGreaterThan(0);
    const image = await page.evaluate(() => {
      const canvas = demo.addons.get('image').getImageAtBufferCell(0, 0);
      if (!canvas) throw new Error('Actual image buffer cell is missing');
      return { width: canvas.width, height: canvas.height, pixel: [...canvas.getContext('2d').getImageData(0, 0, 1, 1).data] };
    });
    expect(image).toEqual({ width: 1, height: 1, pixel: [255, 0, 0, 255] });
    await expect(page.locator('.xterm canvas')).not.toHaveCount(0);
  } else if (name === 'webgl') {
    await fresh(page, '\x1b[38;2;255;0;0mGPU 界😀\x1b[0m');
    await expect(page.locator('.xterm canvas')).not.toHaveCount(0);
    const atlas = await page.evaluate(() => {
      const canvas = demo.addons.get('webgl').textureAtlas;
      if (!canvas) throw new Error('Actual WebGL texture atlas is missing');
      return { width: canvas.width, height: canvas.height };
    });
    expect(atlas.width).toBeGreaterThan(0); expect(atlas.height).toBeGreaterThan(0);
  } else if (name === 'clipboard') {
    const text = `clipboard-${phase}`;
    await page.evaluate(text => navigator.clipboard.writeText(text), text);
    const before = await page.evaluate(() => demo.outgoing.length);
    await page.evaluate(() => demo.write('\x1b]52;c;?\x07'));
    await expect.poll(() => page.evaluate(before => demo.outgoing.slice(before).join(''), before)).toBe(`\x1b]52;c;${Buffer.from(text).toString('base64')}\x07`);
  } else if (name === 'web-fonts') {
    const font = readFileSync(resolve(__dirname, '../../source_repo/addons/addon-ligatures/fonts/FiraCode-Regular.otf')).toString('base64');
    const loaded = await page.evaluate(async font => {
      const face = new FontFace('E2EFira', `url(data:font/otf;base64,${font})`);
      demo.terminal.options.fontFamily = '"E2EFira", monospace';
      const fonts = await demo.addons.get('web-fonts').loadFonts([face]);
      await demo.addons.get('web-fonts').relayout();
      return fonts.map(font => ({ family: font.family, status: font.status }));
    }, font);
    expect(loaded).toEqual([{ family: 'E2EFira', status: 'loaded' }]);
    await fresh(page, 'font => !=');
    await expect(page.locator('.xterm-rows')).toContainText('font');
  } else if (name === 'ligatures') {
    await fresh(page, '-> != ===');
    await expect.poll(() => page.evaluate(() => window.joinerCalls.filter(call => call.ranges.length > 0).length)).toBeGreaterThan(0);
    expect(await page.evaluate(() => window.joinerCalls.some(call => call.text.includes('->') && call.ranges.some(range => range[0] === 0 && range[1] === 2)))).toBe(true);
  } else if (name === 'web-links') {
    await page.context().route('https://example.com/**', route => route.fulfill({ body: 'E2E link target' }));
    await fresh(page, 'https://example.com/');
    await page.locator('.xterm-screen').scrollIntoViewIfNeeded();
    const box = await page.locator('.xterm-screen').boundingBox();
    const dimensions = await page.evaluate(() => [demo.terminal.cols, demo.terminal.rows]);
    const x = box.x + box.width / dimensions[0] * 3, y = box.y + box.height / dimensions[1] / 2;
    await page.mouse.move(x, y);
    await expect(page.locator('.xterm-cursor-pointer')).toBeVisible();
    const popupPromise = page.waitForEvent('popup');
    await page.mouse.click(x, y);
    const popup = await popupPromise;
    await popup.waitForURL('https://example.com/');
    await popup.close();
  } else throw new Error(`No real behavior probe for ${name}`);
}
module.exports = { probe };
