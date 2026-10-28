# Instructions

- Following Playwright test failed.
- Explain why, be concise, respect Playwright best practices.
- Provide a snippet of code with the fix, if possible.

# Test info

- Name: demo-05.spec.cjs >> HTML serialization reaches the real clipboard or reports host denial
- Location: scripts\demo-e2e\demo-05.spec.cjs:95:1

# Error details

```
Error: expect(locator).toContainText(expected) failed

Locator: locator('#html-output')
Expected substring: "HTML"
Received string:    ""
Timeout: 5000ms

Call log:
  - Expect "toContainText" with timeout 5000ms
  - waiting for locator('#html-output')
    14 × locator resolved to <pre id="html-output"></pre>
       - unexpected value ""

```

```yaml
- banner:
  - heading "xterm Rust demo" [level=1]
  - paragraph: Rust parser, buffers and rendering. Full xterm.js equivalence is not proven.
- status: Action failed. See the error and lifecycle log.
- alert: "serialize-html: Error: Rust SerializeAddon is not exported at DemoAddons.load (http://127.0.0.1:50504/demo/addons.mjs:31:50) at DemoAddons.ensure (http://127.0.0.1:50504/demo/addons.mjs:61:71) at http://127.0.0.1:50504/demo/controls.mjs:150:48 at http://127.0.0.1:50504/demo/controls.mjs:48:28 at action (http://127.0.0.1:50504/demo/app.mjs:53:26) at Object.action (http://127.0.0.1:50504/demo/app.mjs:110:48) at byId.addEventListener.signal (http://127.0.0.1:50504/demo/controls.mjs:48:11)"
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
    - group: "Live terminal state and outgoing data { \"cols\": 40, \"rows\": 12, \"cursorX\": 8, \"cursorY\": 0, \"viewportY\": 0, \"baseY\": 0, \"bufferType\": \"normal\", \"selection\": \"\", \"lines\": [ \"HTML 界😀\", \"\", \"\", \"\", \"\", \"\", \"\", \"\", \"\", \"\", \"\", \"\" ], \"addons\": [] }"
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
        - option "attach" [selected]
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
        - option "web-links"
        - option "webgl"
      - text: Attach WebSocket URL
      - textbox "Attach WebSocket URL": ws://127.0.0.1:3000/socket
      - button "Load"
      - button "Unload"
      - button "Reload"
      - text: "[]"
- group: Action and lifecycle log
```

# Test source

```ts
  1   | const { test, expect, fresh, click, change, snapshot } = require('./support.cjs');
  2   | 
  3   | test('live scalar and nested options update the visible terminal', async ({ page }) => {
  4   |   await page.locator('details').filter({ has: page.locator('#options') }).locator('summary').click();
  5   |   for (const [name, value] of Object.entries({ cursorStyle: 'bar', cursorInactiveStyle: 'underline', cursorBlink: true,
  6   |     fontFamily: 'monospace', fontSize: 19, fontWeight: 'bold', fontWeightBold: 'normal', lineHeight: 1.3, letterSpacing: 2 })) {
  7   |     await change(page, `opt-${name}`, value);
  8   |     await expect.poll(() => page.evaluate(name => demo.terminal.options[name], name)).toBe(value);
  9   |   }
  10  |   const initial = await page.evaluate(() => ({ ...demo.terminal.options.vtExtensions }));
  11  |   for (const name of ['kittyKeyboard', 'kittySgrBoldFaintControl', 'win32InputMode']) {
  12  |     await change(page, `opt-vtExtensions-${name}`, !initial[name]);
  13  |     initial[name] = !initial[name];
  14  |     expect(await page.evaluate(() => ({ ...demo.terminal.options.vtExtensions }))).toEqual(initial);
  15  |   }
  16  |   await change(page, 'opt-scrollbar-showArrows', true);
  17  |   expect(await page.evaluate(() => demo.terminal.options.scrollbar.showArrows)).toBe(true);
  18  |   const previous = await page.evaluate(() => demo.terminal.options.scrollback);
  19  |   await change(page, 'opt-scrollback', -1);
  20  |   await expect(page.locator('#error')).toBeVisible();
  21  |   expect(await page.evaluate(() => demo.terminal.options.scrollback)).toBe(previous);
  22  |   await expect(page.locator('.xterm-rows')).toContainText('Rust terminal');
  23  | });
  24  | 
  25  | test('themes and transparency change rendered pixels without replacing state', async ({ page }, info) => {
  26  |   const before = await snapshot(page);
  27  |   const pixels = await page.locator('.xterm-screen').screenshot();
  28  |   await change(page, 'theme', 'light');
  29  |   await expect(page.locator('#terminal-container')).toHaveCSS('background-color', 'rgb(255, 255, 255)');
  30  |   const after = await page.locator('.xterm-screen').screenshot();
  31  |   expect(pixels.equals(after), 'The theme must change actual rendered pixels').toBe(false);
  32  |   expect(await snapshot(page)).toEqual(before);
  33  |   await info.attach('light-theme', { body: after, contentType: 'image/png' });
  34  |   await page.locator('details').filter({ has: page.locator('#options') }).locator('summary').click();
  35  |   await change(page, 'opt-allowTransparency', true);
  36  |   await expect(page.locator('#terminal-container')).toHaveCSS('background-color', 'rgba(0, 0, 0, 0)');
  37  | });
  38  | 
  39  | test('explicit size, invalid auto-size, font and padding use real geometry', async ({ page }) => {
  40  |   await page.locator('#size').fill('60x16');
  41  |   await page.locator('#size-form button').click();
  42  |   await expect.poll(() => page.evaluate(() => [demo.terminal.cols, demo.terminal.rows])).toEqual([60, 16]);
  43  |   await expect(page.locator('.xterm-screen')).toHaveCSS('outline-style', 'solid');
  44  |   await page.locator('#size').fill('invalid');
  45  |   await page.locator('#size-form button').click();
  46  |   await expect(page.locator('#size')).toHaveValue('auto');
  47  |   await expect(page.locator('.xterm-screen')).toHaveCSS('outline-style', 'none');
  48  |   const geometry = await page.locator('.xterm-screen').boundingBox();
  49  |   const cols = await page.evaluate(() => demo.terminal.cols);
  50  |   await change(page, 'padding', 40);
  51  |   await expect.poll(() => page.evaluate(() => demo.terminal.cols)).toBeLessThan(cols);
  52  |   await page.locator('details').filter({ has: page.locator('#options') }).locator('summary').click();
  53  |   await change(page, 'opt-fontSize', 24);
  54  |   await expect.poll(() => page.locator('.xterm-screen').boundingBox()).not.toEqual(geometry);
  55  |   expect(await page.evaluate(() => demo.terminal.cols)).toBeGreaterThan(0);
  56  | });
  57  | 
  58  | test('search flags, wrapped Unicode ranges, direction and failed search are live', async ({ page }) => {
  59  |   await fresh(page, '\x1b[31mRust 界😀\x1b[0m\r\nrust test\r\nRust Rusty\r\n123456789Rust', 12, 8);
  60  |   await page.locator('#search-text').fill('Rust');
  61  |   await change(page, 'search-case', true);
  62  |   await change(page, 'search-word', true);
  63  |   await click(page, 'find-next');
  64  |   let result = JSON.parse(await page.locator('#search-results').textContent());
  65  |   expect(result.found).toBe(true); expect(result.selection).toBe('Rust');
  66  |   expect(result.result.resultCount).toBe(2);
  67  |   const first = await page.evaluate(() => demo.terminal.getSelectionPosition());
  68  |   expect(first).toEqual({ start: { x: 0, y: 0 }, end: { x: 4, y: 0 } });
  69  |   await click(page, 'find-next');
  70  |   expect(await page.evaluate(() => demo.terminal.getSelectionPosition())).toEqual({ start: { x: 0, y: 2 }, end: { x: 4, y: 2 } });
  71  |   await click(page, 'find-previous');
  72  |   expect(await page.evaluate(() => demo.terminal.getSelectionPosition())).toEqual(first);
  73  |   await change(page, 'search-word', false);
  74  |   await change(page, 'search-regex', true);
  75  |   await page.locator('#search-text').fill('9Rust');
  76  |   await click(page, 'find-next');
  77  |   expect(await page.evaluate(() => demo.terminal.getSelection())).toBe('9Rust');
  78  |   expect(await page.evaluate(() => demo.terminal.getSelectionPosition())).toEqual({ start: { x: 8, y: 3 }, end: { x: 1, y: 4 } });
  79  |   await page.locator('#search-text').fill('does-not-exist');
  80  |   await click(page, 'find-next');
  81  |   result = JSON.parse(await page.locator('#search-results').textContent());
  82  |   expect(result.found).toBe(false); expect(result.result.resultCount).toBe(0);
  83  | });
  84  | 
  85  | test('ANSI serialization write-back preserves complete cells, wraps and cursor', async ({ page }) => {
  86  |   await fresh(page, '\x1b[1;3;4;38;2;255;0;0mRust 界😀e\u0301 abcdefghijklmnop\x1b[0m\r\nsecond\x1b[2D', 16, 8);
  87  |   const before = await snapshot(page);
  88  |   await page.locator('#write-back').check();
  89  |   await click(page, 'serialize');
  90  |   const output = JSON.parse(await page.locator('#serialize-output').textContent());
  91  |   expect(output).toContain('Rust'); expect(output).toContain('\x1b[');
  92  |   expect(await snapshot(page)).toEqual(before);
  93  | });
  94  | 
  95  | test('HTML serialization reaches the real clipboard or reports host denial', async ({ page, context, browserName }) => {
  96  |   if (browserName === 'chromium') await context.grantPermissions(['clipboard-read', 'clipboard-write']);
  97  |   await fresh(page, '\x1b[38;2;255;0;0mHTML 界😀\x1b[0m');
  98  |   await page.locator('#serialize-html').click();
> 99  |   await expect(page.locator('#html-output')).toContainText('HTML');
      |                                              ^ Error: expect(locator).toContainText(expected) failed
  100 |   const html = await page.locator('#html-output').textContent();
  101 |   expect(html).toContain('<'); expect(html).toContain('255');
  102 |   await expect.poll(() => page.evaluate(() => demo.events.some(event => event.name === 'serialize:html-copied') || demo.errors.length > 0)).toBe(true);
  103 |   const copied = await page.evaluate(() => demo.events.some(event => event.name === 'serialize:html-copied'));
  104 |   if (browserName === 'chromium') expect(copied, 'Granted Chromium clipboard permission must copy actual HTML').toBe(true);
  105 |   if (copied) {
  106 |     const actual = await page.evaluate(async () => {
  107 |       const items = await navigator.clipboard.read();
  108 |       const item = items.find(item => item.types.includes('text/html'));
  109 |       if (!item) throw new Error('HTML clipboard item is missing');
  110 |       return (await item.getType('text/html')).text();
  111 |     });
  112 |     expect(actual).toContain('HTML'); expect(actual).toContain('界😀');
  113 |   } else {
  114 |     await expect(page.locator('#error')).toBeVisible();
  115 |     expect(await page.evaluate(() => demo.errors.at(-1))).toMatch(/clipboard|copy|permission|denied|allowed/i);
  116 |   }
  117 | });
  118 | 
  119 | test('DOM and WebGL controls replace actual renderer ownership and retain selection', async ({ page }) => {
  120 |   await fresh(page, 'renderer 界😀');
  121 |   await page.evaluate(() => demo.terminal.select(0, 0, 8));
  122 |   const before = await snapshot(page);
  123 |   const canvases = await page.locator('.xterm canvas').count();
  124 |   await change(page, 'renderer', 'webgl');
  125 |   await expect.poll(() => page.locator('.xterm canvas').count()).toBeGreaterThan(canvases);
  126 |   expect(await snapshot(page)).toEqual(before);
  127 |   expect(await page.evaluate(() => demo.terminal.getSelection())).toBe('renderer');
  128 |   await change(page, 'renderer', 'dom');
  129 |   await expect.poll(() => page.locator('.xterm canvas').count()).toBe(canvases);
  130 |   await expect(page.locator('.xterm-rows')).toContainText('renderer');
  131 |   expect(await snapshot(page)).toEqual(before);
  132 | });
  133 | 
```