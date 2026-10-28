# Instructions

- Following Playwright test failed.
- Explain why, be concise, respect Playwright best practices.
- Provide a snippet of code with the fix, if possible.

# Test info

- Name: demo-06.spec.cjs >> clipboard load, reload, behavior and unload use the visible session
- Location: scripts\demo-e2e\demo-06.spec.cjs:7:28

# Error details

```
Error: expect(received).toBe(expected) // Object.is equality

Expected: ""
Received: ""

Call Log:
- Timeout 5000ms exceeded while waiting on the predicate
```

# Page snapshot

```yaml
- generic [active] [ref=e1]:
  - banner [ref=e2]:
    - heading "xterm Rust demo" [level=1] [ref=e3]
    - paragraph [ref=e4]: Rust parser, buffers and rendering. Full xterm.js equivalence is not proven.
  - status [ref=e5]: Rust terminal ready
  - main [ref=e6]:
    - region "Terminal session" [ref=e7]:
      - generic [ref=e8]:
        - button "ANSI and Unicode sample" [ref=e9] [cursor=pointer]
        - button "Reset" [ref=e10] [cursor=pointer]
        - button "Dispose" [ref=e11] [cursor=pointer]
        - button "Reopen" [ref=e12] [cursor=pointer]
      - generic [ref=e14]:
        - generic [ref=e15]:
          - text: Incoming display text, or a JSON string with escapes
          - textbox "Incoming display text, or a JSON string with escapes" [ref=e16]: hello from Rust
        - button "Write" [ref=e17] [cursor=pointer]
      - generic [ref=e18]:
        - generic [ref=e19]:
          - text: Outgoing user input
          - textbox "Outgoing user input" [ref=e20]: user input
        - button "Send input" [ref=e21] [cursor=pointer]
      - group [ref=e22]:
        - generic "Live terminal state and outgoing data" [ref=e23] [cursor=pointer]
        - generic [ref=e24]: Terminal disposed
    - complementary "Live terminal controls" [ref=e25]:
      - group [ref=e26]:
        - generic "Size, style and renderer" [ref=e27] [cursor=pointer]
        - generic [ref=e28]:
          - generic [ref=e29]:
            - text: Size, COLSxROWS or auto
            - textbox "Size, COLSxROWS or auto" [ref=e30]: 80x24
          - button "Resize" [ref=e31] [cursor=pointer]
        - generic [ref=e32]:
          - text: Padding
          - spinbutton "Padding" [ref=e33]: "8"
        - generic [ref=e34]:
          - text: Theme
          - combobox "Theme" [ref=e35]:
            - option "default" [selected]
            - option "xtermjs"
            - option "sapphire"
            - option "light"
        - generic [ref=e36]:
          - text: Renderer
          - combobox "Renderer" [ref=e37]:
            - option "DOM" [selected]
            - option "WebGL"
      - group [ref=e38]:
        - generic "Options on this terminal" [ref=e39] [cursor=pointer]
        - option "block" [selected]
        - option "underline"
        - option "bar"
        - option "outline" [selected]
        - option "block"
        - option "bar"
        - option "underline"
        - option "none"
        - option "normal" [selected]
        - option "bold"
        - option "100"
        - option "200"
        - option "300"
        - option "400"
        - option "500"
        - option "600"
        - option "700"
        - option "800"
        - option "900"
        - option "normal"
        - option "bold" [selected]
        - option "100"
        - option "200"
        - option "300"
        - option "400"
        - option "500"
        - option "600"
        - option "700"
        - option "800"
        - option "900"
        - option "trace"
        - option "debug"
        - option "info" [selected]
        - option "warn"
        - option "error"
        - option "off"
      - group [ref=e40]:
        - generic "Search" [ref=e41] [cursor=pointer]
        - generic [ref=e42]:
          - text: Text
          - textbox "Text" [ref=e43]: Rust
        - generic [ref=e44]:
          - checkbox "Regex" [ref=e45]
          - text: Regex
        - generic [ref=e46]:
          - checkbox "Case sensitive" [ref=e47]
          - text: Case sensitive
        - generic [ref=e48]:
          - checkbox "Whole word" [ref=e49]
          - text: Whole word
        - generic [ref=e50]:
          - checkbox "Highlight all matches" [checked] [ref=e51]
          - text: Highlight all matches
        - button "Find next" [ref=e52] [cursor=pointer]
        - button "Find previous" [ref=e53] [cursor=pointer]
        - status
      - group [ref=e54]:
        - generic "Serialize" [ref=e55] [cursor=pointer]
        - button "Serialize ANSI" [ref=e56] [cursor=pointer]
        - generic [ref=e57]:
          - checkbox "Reset and write back to this terminal" [ref=e58]
          - text: Reset and write back to this terminal
        - button "Serialize HTML and copy" [ref=e59] [cursor=pointer]
      - group [ref=e60]:
        - generic "Addon lifecycle" [ref=e61] [cursor=pointer]
        - generic [ref=e62]:
          - text: Addon
          - combobox "Addon" [ref=e63]:
            - option "attach"
            - option "clipboard" [selected]
            - option "fit"
            - option "image"
            - option "ligatures"
            - option "program-status"
            - option "progress"
            - option "search"
            - option "serialize"
            - option "unicode-graphemes"
            - option "unicode11"
            - option "web-fonts"
            - option "web-links"
            - option "webgl"
        - generic [ref=e64]:
          - text: Attach WebSocket URL
          - textbox "Attach WebSocket URL" [ref=e65]: ws://127.0.0.1:55764/socket
        - button "Load" [ref=e66] [cursor=pointer]
        - button "Unload" [ref=e67] [cursor=pointer]
        - button "Reload" [ref=e68] [cursor=pointer]
        - generic [ref=e69]: "[]"
  - group [ref=e70]:
    - generic "Action and lifecycle log" [ref=e71] [cursor=pointer]
```

# Test source

```ts
  1   | const { readFileSync } = require('node:fs');
  2   | const { resolve } = require('node:path');
  3   | const { expect, fresh } = require('./support.cjs');
  4   | 
  5   | async function probe(page, name, phase) {
  6   |   if (name === 'attach') {
  7   |     await expect.poll(() => page.evaluate(() => demo.addons.loaded.get('attach').socket.readyState)).toBe(1);
  8   |     await fresh(page, '');
  9   |     const text = `socket-${phase}`;
  10  |     await page.evaluate(text => demo.terminal.input(text, true), text);
  11  |     await expect.poll(() => page.evaluate(() => demo.terminal.buffer.active.getLine(0).translateToString(true))).toBe(text);
  12  |   } else if (name === 'fit') {
  13  |     await page.evaluate(() => demo.addons.get('fit').fit());
  14  |     expect(await page.evaluate(() => demo.addons.get('fit').proposeDimensions())).toEqual(
  15  |       await page.evaluate(() => ({ cols: demo.terminal.cols, rows: demo.terminal.rows })));
  16  |     const cols = await page.evaluate(() => demo.terminal.cols);
  17  |     await page.locator('#terminal-container').evaluate(element => { element.style.width = '300px'; });
  18  |     await page.evaluate(() => demo.addons.get('fit').fit());
  19  |     expect(await page.evaluate(() => demo.terminal.cols)).toBeLessThan(cols);
  20  |     await page.locator('#terminal-container').evaluate(element => { element.style.width = ''; });
  21  |   } else if (name === 'progress' || name === 'program-status') {
  22  |     const event = `addon:${name}:onChange`;
  23  |     const before = await page.evaluate(event => demo.events.filter(item => item.name === event).length, event);
  24  |     await page.evaluate(async name => {
  25  |       await demo.write(name === 'progress' ? '\x1b]9;4;1;42\x07' : '\x1b]7501;state=working:id=e2e:app=rust\x1b\\');
  26  |     }, name);
  27  |     expect(await page.evaluate(event => demo.events.filter(item => item.name === event).length, event)).toBe(before + 1);
  28  |     if (name === 'progress') expect(await page.evaluate(() => demo.addons.get('progress').progress)).toEqual({ state: 1, value: 42 });
  29  |     else expect(await page.evaluate(() => demo.addons.get('program-status').records)).toEqual([{ id: 'e2e', state: 'working', app: 'rust', effectiveApp: 'rust' }]);
  30  |   } else if (name === 'unicode11' || name === 'unicode-graphemes') {
  31  |     const version = name === 'unicode11' ? '11' : '15-graphemes';
  32  |     expect(await page.evaluate(() => demo.terminal.unicode.versions)).toContain(version);
  33  |     await page.evaluate(version => { demo.terminal.unicode.activeVersion = version; }, version);
  34  |     await fresh(page, 'A界😀');
  35  |     expect(await page.evaluate(() => demo.terminal.buffer.active.cursorX)).toBe(5);
  36  |     expect(await page.evaluate(() => [demo.terminal.buffer.active.getLine(0).getCell(1).getWidth(), demo.terminal.buffer.active.getLine(0).getCell(3).getWidth()])).toEqual([2, 2]);
  37  |     if (name === 'unicode-graphemes') {
  38  |       await fresh(page, '👩\u200d🚀');
  39  |       expect(await page.evaluate(() => demo.terminal.buffer.active.cursorX)).toBe(2);
  40  |     }
  41  |   } else if (name === 'search') {
  42  |     await fresh(page, `probe ${phase} 界😀`);
  43  |     expect(await page.evaluate(phase => demo.addons.get('search').findNext(phase), phase)).toBe(true);
  44  |     expect(await page.evaluate(() => demo.terminal.getSelection())).toBe(phase);
  45  |   } else if (name === 'serialize') {
  46  |     await fresh(page, '\x1b[31mserialize 界😀\x1b[0m');
  47  |     expect(await page.evaluate(() => demo.addons.get('serialize').serialize())).toContain('serialize 界😀');
  48  |     expect(await page.evaluate(() => demo.addons.get('serialize').serializeAsHTML())).toContain('serialize');
  49  |   } else if (name === 'image') {
  50  |     await fresh(page, '');
  51  |     await page.evaluate(async () => {
  52  |       window.imageEvents = 0;
  53  |       window.imageHandle?.dispose();
  54  |       window.imageHandle = demo.addons.get('image').onImageAdded(() => window.imageEvents++);
  55  |       await demo.write('\x1b_Ga=T,f=32,s=1,v=1,i=71;/wAA/w==\x1b\\');
  56  |     });
  57  |     await expect.poll(() => page.evaluate(() => window.imageEvents)).toBe(1);
  58  |     expect(await page.evaluate(() => demo.addons.get('image').storageUsage)).toBeGreaterThan(0);
  59  |     const image = await page.evaluate(() => {
  60  |       const canvas = demo.addons.get('image').getImageAtBufferCell(0, 0);
  61  |       if (!canvas) throw new Error('Actual image buffer cell is missing');
  62  |       return { width: canvas.width, height: canvas.height, pixel: [...canvas.getContext('2d').getImageData(0, 0, 1, 1).data] };
  63  |     });
  64  |     expect(image).toEqual({ width: 1, height: 1, pixel: [255, 0, 0, 255] });
  65  |     await expect(page.locator('.xterm canvas')).not.toHaveCount(0);
  66  |   } else if (name === 'webgl') {
  67  |     await fresh(page, '\x1b[38;2;255;0;0mGPU 界😀\x1b[0m');
  68  |     await expect(page.locator('.xterm canvas')).not.toHaveCount(0);
  69  |     const atlas = await page.evaluate(() => {
  70  |       const canvas = demo.addons.get('webgl').textureAtlas;
  71  |       if (!canvas) throw new Error('Actual WebGL texture atlas is missing');
  72  |       return { width: canvas.width, height: canvas.height };
  73  |     });
  74  |     expect(atlas.width).toBeGreaterThan(0); expect(atlas.height).toBeGreaterThan(0);
  75  |   } else if (name === 'clipboard') {
  76  |     const text = `clipboard-${phase}`;
  77  |     await page.evaluate(text => navigator.clipboard.writeText(text), text);
  78  |     const before = await page.evaluate(() => demo.outgoing.length);
  79  |     await page.evaluate(() => demo.write('\x1b]52;c;?\x07'));
> 80  |     await expect.poll(() => page.evaluate(before => demo.outgoing.slice(before).join(''), before)).toBe(`\x1b]52;c;${Buffer.from(text).toString('base64')}\x07`);
      |                                                                                                    ^ Error: expect(received).toBe(expected) // Object.is equality
  81  |   } else if (name === 'web-fonts') {
  82  |     const font = readFileSync(resolve(__dirname, '../../source_repo/addons/addon-ligatures/fonts/FiraCode-Regular.otf')).toString('base64');
  83  |     const loaded = await page.evaluate(async font => {
  84  |       const face = new FontFace('E2EFira', `url(data:font/otf;base64,${font})`);
  85  |       demo.terminal.options.fontFamily = '"E2EFira", monospace';
  86  |       const fonts = await demo.addons.get('web-fonts').loadFonts([face]);
  87  |       await demo.addons.get('web-fonts').relayout();
  88  |       return fonts.map(font => ({ family: font.family, status: font.status }));
  89  |     }, font);
  90  |     expect(loaded).toEqual([{ family: 'E2EFira', status: 'loaded' }]);
  91  |     await fresh(page, 'font => !=');
  92  |     await expect(page.locator('.xterm-rows')).toContainText('font');
  93  |   } else if (name === 'ligatures') {
  94  |     await fresh(page, '-> != ===');
  95  |     await expect.poll(() => page.evaluate(() => window.joinerCalls.filter(call => call.ranges.length > 0).length)).toBeGreaterThan(0);
  96  |     expect(await page.evaluate(() => window.joinerCalls.some(call => call.text.includes('->') && call.ranges.some(range => range[0] === 0 && range[1] === 2)))).toBe(true);
  97  |   } else if (name === 'web-links') {
  98  |     await page.context().route('https://example.com/**', route => route.fulfill({ body: 'E2E link target' }));
  99  |     await fresh(page, 'https://example.com/');
  100 |     await page.locator('.xterm-screen').scrollIntoViewIfNeeded();
  101 |     const box = await page.locator('.xterm-screen').boundingBox();
  102 |     const dimensions = await page.evaluate(() => [demo.terminal.cols, demo.terminal.rows]);
  103 |     const x = box.x + box.width / dimensions[0] * 3, y = box.y + box.height / dimensions[1] / 2;
  104 |     await page.mouse.move(x, y);
  105 |     await expect(page.locator('.xterm-cursor-pointer')).toBeVisible();
  106 |     const popupPromise = page.waitForEvent('popup');
  107 |     await page.mouse.click(x, y);
  108 |     const popup = await popupPromise;
  109 |     await popup.waitForURL('https://example.com/');
  110 |     await popup.close();
  111 |   } else throw new Error(`No real behavior probe for ${name}`);
  112 | }
  113 | module.exports = { probe };
  114 | 
```