# Instructions

- Following Playwright test failed.
- Explain why, be concise, respect Playwright best practices.
- Provide a snippet of code with the fix, if possible.

# Test info

- Name: demo-05.spec.cjs >> DOM and WebGL controls replace actual renderer ownership and retain selection
- Location: scripts\demo-e2e\demo-05.spec.cjs:119:1

# Error details

```
Error: expect(received).toBeGreaterThan(expected)

Expected: > 0
Received:   0

Call Log:
- Timeout 5000ms exceeded while waiting on the predicate
```

# Page snapshot

```yaml
- generic [active] [ref=e1]:
  - banner [ref=e2]:
    - heading "xterm Rust demo" [level=1] [ref=e3]
    - paragraph [ref=e4]: Rust parser, buffers and rendering. Full xterm.js equivalence is not proven.
  - status [ref=e5]: Action failed. See the error and lifecycle log.
  - alert [ref=e6]: "renderer: load@http://127.0.0.1:50504/demo/addons.mjs:31:50 ensure@http://127.0.0.1:50504/demo/addons.mjs:61:71 setupControls/<@http://127.0.0.1:50504/demo/controls.mjs:126:52 setupControls/on/</<@http://127.0.0.1:50504/demo/controls.mjs:48:34 action@http://127.0.0.1:50504/demo/app.mjs:53:26 action@http://127.0.0.1:50504/demo/app.mjs:110:54 setupControls/on/<@http://127.0.0.1:50504/demo/controls.mjs:48:11"
  - main [ref=e7]:
    - region "Terminal session" [ref=e8]:
      - generic [ref=e9]:
        - button "ANSI and Unicode sample" [ref=e10] [cursor=pointer]
        - button "Reset" [ref=e11] [cursor=pointer]
        - button "Dispose" [ref=e12] [cursor=pointer]
        - button "Reopen" [ref=e13] [cursor=pointer]
      - generic [ref=e15]:
        - generic [ref=e16]:
          - text: Incoming display text, or a JSON string with escapes
          - textbox "Incoming display text, or a JSON string with escapes" [ref=e17]: hello from Rust
        - button "Write" [ref=e18] [cursor=pointer]
      - generic [ref=e19]:
        - generic [ref=e20]:
          - text: Outgoing user input
          - textbox "Outgoing user input" [ref=e21]: user input
        - button "Send input" [ref=e22] [cursor=pointer]
      - group [ref=e23]:
        - generic "Live terminal state and outgoing data" [ref=e24] [cursor=pointer]
        - generic [ref=e25]: Terminal disposed
    - complementary "Live terminal controls" [ref=e26]:
      - group [ref=e27]:
        - generic "Size, style and renderer" [ref=e28] [cursor=pointer]
        - generic [ref=e29]:
          - generic [ref=e30]:
            - text: Size, COLSxROWS or auto
            - textbox "Size, COLSxROWS or auto" [ref=e31]: 80x24
          - button "Resize" [ref=e32] [cursor=pointer]
        - generic [ref=e33]:
          - text: Padding
          - spinbutton "Padding" [ref=e34]: "8"
        - generic [ref=e35]:
          - text: Theme
          - combobox "Theme" [ref=e36]:
            - option "default" [selected]
            - option "xtermjs"
            - option "sapphire"
            - option "light"
        - generic [ref=e37]:
          - text: Renderer
          - combobox "Renderer" [ref=e38]:
            - option "DOM"
            - option "WebGL" [selected]
      - group [ref=e39]:
        - generic "Options on this terminal" [ref=e40] [cursor=pointer]
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
      - group [ref=e41]:
        - generic "Search" [ref=e42] [cursor=pointer]
        - generic [ref=e43]:
          - text: Text
          - textbox "Text" [ref=e44]: Rust
        - generic [ref=e45]:
          - checkbox "Regex" [ref=e46]
          - text: Regex
        - generic [ref=e47]:
          - checkbox "Case sensitive" [ref=e48]
          - text: Case sensitive
        - generic [ref=e49]:
          - checkbox "Whole word" [ref=e50]
          - text: Whole word
        - generic [ref=e51]:
          - checkbox "Highlight all matches" [checked] [ref=e52]
          - text: Highlight all matches
        - button "Find next" [ref=e53] [cursor=pointer]
        - button "Find previous" [ref=e54] [cursor=pointer]
        - status
      - group [ref=e55]:
        - generic "Serialize" [ref=e56] [cursor=pointer]
        - button "Serialize ANSI" [ref=e57] [cursor=pointer]
        - generic [ref=e58]:
          - checkbox "Reset and write back to this terminal" [ref=e59]
          - text: Reset and write back to this terminal
        - button "Serialize HTML and copy" [ref=e60] [cursor=pointer]
      - group [ref=e61]:
        - generic "Addon lifecycle" [ref=e62] [cursor=pointer]
        - generic [ref=e63]:
          - text: Addon
          - combobox "Addon" [ref=e64]:
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
        - generic [ref=e65]:
          - text: Attach WebSocket URL
          - textbox "Attach WebSocket URL" [ref=e66]: ws://127.0.0.1:3000/socket
        - button "Load" [ref=e67] [cursor=pointer]
        - button "Unload" [ref=e68] [cursor=pointer]
        - button "Reload" [ref=e69] [cursor=pointer]
        - generic [ref=e70]: "[]"
  - group [ref=e71]:
    - generic "Action and lifecycle log" [ref=e72] [cursor=pointer]
```

# Test source

```ts
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
  99  |   await expect(page.locator('#html-output')).toContainText('HTML');
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
> 125 |   await expect.poll(() => page.locator('.xterm canvas').count()).toBeGreaterThan(canvases);
      |                                                                  ^ Error: expect(received).toBeGreaterThan(expected)
  126 |   expect(await snapshot(page)).toEqual(before);
  127 |   expect(await page.evaluate(() => demo.terminal.getSelection())).toBe('renderer');
  128 |   await change(page, 'renderer', 'dom');
  129 |   await expect.poll(() => page.locator('.xterm canvas').count()).toBe(canvases);
  130 |   await expect(page.locator('.xterm-rows')).toContainText('renderer');
  131 |   expect(await snapshot(page)).toEqual(before);
  132 | });
  133 | 
```