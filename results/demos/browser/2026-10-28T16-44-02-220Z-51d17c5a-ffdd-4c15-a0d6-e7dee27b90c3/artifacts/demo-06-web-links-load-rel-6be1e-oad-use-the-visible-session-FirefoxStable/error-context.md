# Instructions

- Following Playwright test failed.
- Explain why, be concise, respect Playwright best practices.
- Provide a snippet of code with the fix, if possible.

# Test info

- Name: demo-06.spec.cjs >> web-links load, reload, behavior and unload use the visible session
- Location: scripts\demo-e2e\demo-06.spec.cjs:7:28

# Error details

```
Error: expect(locator).toBeVisible() failed

Locator: locator('.xterm-cursor-pointer')
Expected: visible
Timeout: 5000ms
Error: element(s) not found

Call log:
  - Expect "toBeVisible" with timeout 5000ms
  - waiting for locator('.xterm-cursor-pointer')

```

```yaml
- banner:
  - heading "xterm Rust demo" [level=1]
  - paragraph: Rust parser, buffers and rendering. Full xterm.js equivalence is not proven.
- status: Rust terminal ready
- main:
  - region "Terminal session":
    - button "ANSI and Unicode sample"
    - button "Reset"
    - button "Dispose"
    - button "Reopen"
    - textbox "Terminal input"
    - text: Incoming display text, or a JSON string with escapes
    - textbox "Incoming display text, or a JSON string with escapes": hello from Rust
    - button "Write"
    - text: Outgoing user input
    - textbox "Outgoing user input": user input
    - button "Send input"
    - group: "Live terminal state and outgoing data { \"cols\": 40, \"rows\": 12, \"cursorX\": 20, \"cursorY\": 0, \"viewportY\": 0, \"baseY\": 0, \"bufferType\": \"normal\", \"selection\": \"\", \"lines\": [ \"https://example.com/\", \"\", \"\", \"\", \"\", \"\", \"\", \"\", \"\", \"\", \"\", \"\" ], \"addons\": [ \"web-links\" ] }"
  - complementary "Live terminal controls":
    - group:
      - text: Size, style and renderer Size, COLSxROWS or auto
      - textbox "Size, COLSxROWS or auto": 80x24
      - button "Resize"
      - text: Padding
      - spinbutton "Padding": "8"
      - text: Theme
      - combobox "Theme":
        - option "default" [selected]
        - option "xtermjs"
        - option "sapphire"
        - option "light"
      - text: Renderer
      - combobox "Renderer":
        - option "DOM" [selected]
        - option "WebGL"
    - group: Options on this terminal
    - group:
      - text: Search Text
      - textbox "Text": Rust
      - checkbox "Regex"
      - text: Regex
      - checkbox "Case sensitive"
      - text: Case sensitive
      - checkbox "Whole word"
      - text: Whole word
      - checkbox "Highlight all matches" [checked]
      - text: Highlight all matches
      - button "Find next"
      - button "Find previous"
      - status
    - group:
      - text: Serialize
      - button "Serialize ANSI"
      - checkbox "Reset and write back to this terminal"
      - text: Reset and write back to this terminal
      - button "Serialize HTML and copy"
    - group:
      - text: Addon lifecycle Addon
      - combobox "Addon":
        - option "attach"
        - option "clipboard"
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
        - option "web-links" [selected]
        - option "webgl"
      - text: Attach WebSocket URL
      - textbox "Attach WebSocket URL": ws://127.0.0.1:55764/socket
      - button "Load"
      - button "Unload"
      - button "Reload"
      - text: "[\"web-links\"]"
- group: Action and lifecycle log
```

# Test source

```ts
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
  80  |     await expect.poll(() => page.evaluate(before => demo.outgoing.slice(before).join(''), before)).toBe(`\x1b]52;c;${Buffer.from(text).toString('base64')}\x07`);
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
> 105 |     await expect(page.locator('.xterm-cursor-pointer')).toBeVisible();
      |                                                         ^ Error: expect(locator).toBeVisible() failed
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