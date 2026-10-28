# Instructions

- Following Playwright test failed.
- Explain why, be concise, respect Playwright best practices.
- Provide a snippet of code with the fix, if possible.

# Test info

- Name: demo-06.spec.cjs >> unicode-graphemes load, reload, behavior and unload use the visible session
- Location: scripts\demo-e2e\demo-06.spec.cjs:7:28

# Error details

```
Error: page.evaluate: TypeError: undefined is not an object (evaluating 'demo.terminal.unicode.activeVersion')
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
        - generic [ref=e64]:
          - text: Attach WebSocket URL
          - textbox "Attach WebSocket URL" [ref=e65]: ws://127.0.0.1:3000/socket
        - button "Load" [ref=e66] [cursor=pointer]
        - button "Unload" [ref=e67] [cursor=pointer]
        - button "Reload" [ref=e68] [cursor=pointer]
        - generic [ref=e69]: "[]"
  - group [ref=e70]:
    - generic "Action and lifecycle log" [ref=e71] [cursor=pointer]
```

# Test source

```ts
  1  | const { test, expect, click, snapshot, fresh } = require('./support.cjs');
  2  | const { probe } = require('./addon-probes.cjs');
  3  | 
  4  | const addons = ['attach', 'clipboard', 'fit', 'image', 'ligatures', 'program-status', 'progress', 'search',
  5  |   'serialize', 'unicode-graphemes', 'unicode11', 'web-fonts', 'web-links', 'webgl'];
  6  | 
  7  | for (const name of addons) test(`${name} load, reload, behavior and unload use the visible session`, async ({ page, context, browserName }, info) => {
  8  |   if (name === 'clipboard' && browserName === 'chromium') await context.grantPermissions(['clipboard-read', 'clipboard-write']);
> 9  |   const previousVersion = name.startsWith('unicode') ? await page.evaluate(() => demo.terminal.unicode.activeVersion) : undefined;
     |                                                                   ^ Error: page.evaluate: TypeError: undefined is not an object (evaluating 'demo.terminal.unicode.activeVersion')
  10 |   const initialCanvases = await page.locator('.xterm canvas').count();
  11 |   if (name === 'ligatures') await page.evaluate(() => {
  12 |     window.joinerCalls = [];
  13 |     const terminal = demo.terminal, register = terminal.registerCharacterJoiner;
  14 |     terminal.registerCharacterJoiner = handler => register.call(terminal, text => {
  15 |       const ranges = handler(text); window.joinerCalls.push({ text, ranges }); return ranges;
  16 |     });
  17 |   });
  18 |   await page.locator('#addon-select').selectOption(name);
  19 |   await page.locator('#socket-url').fill(new URL('/socket', page.url()).href.replace('http:', 'ws:'));
  20 |   await click(page, 'addon-load');
  21 |   expect(await page.evaluate(() => demo.snapshot().addons)).toContain(name);
  22 |   await probe(page, name, 'first');
  23 |   await info.attach(`${name}-loaded`, { body: await page.locator('.xterm').screenshot(), contentType: 'image/png' });
  24 |   await page.evaluate(name => {
  25 |     window.oldAddon = demo.addons.get(name);
  26 |     window.oldSocket = demo.addons.loaded.get(name).socket;
  27 |   }, name);
  28 |   await click(page, 'addon-reload');
  29 |   expect(await page.evaluate(name => demo.addons.get(name) !== window.oldAddon, name)).toBe(true);
  30 |   if (name === 'attach') await expect.poll(() => page.evaluate(() => window.oldSocket.readyState)).toBe(3);
  31 |   await probe(page, name, 'second');
  32 |   const inputBefore = await page.evaluate(() => demo.outgoing.length);
  33 |   await page.locator('.xterm-helper-textarea').press('a');
  34 |   await expect.poll(() => page.evaluate(before => demo.outgoing.slice(before), inputBefore)).toEqual(['a']);
  35 |   await click(page, 'addon-unload');
  36 |   expect(await page.evaluate(() => demo.snapshot().addons)).not.toContain(name);
  37 |   if (name === 'webgl' || name === 'image') await expect.poll(() => page.locator('.xterm canvas').count()).toBe(initialCanvases);
  38 |   if (name === 'unicode11' || name === 'unicode-graphemes') {
  39 |     expect(await page.evaluate(() => demo.terminal.unicode.activeVersion)).toBe(previousVersion);
  40 |   }
  41 |   if (name === 'program-status' || name === 'progress' || name === 'clipboard') {
  42 |     const before = await page.evaluate(() => ({ events: demo.events.length, outgoing: demo.outgoing.length }));
  43 |     await page.evaluate(name => demo.write(name === 'program-status' ? '\x1b]7501;?\x07'
  44 |       : name === 'progress' ? '\x1b]9;4;1;89\x07' : '\x1b]52;c;?\x07'), name);
  45 |     const after = await page.evaluate(before => ({
  46 |       changes: demo.events.slice(before.events).filter(event => event.name.startsWith('addon:')).length,
  47 |       outgoing: demo.outgoing.slice(before.outgoing)
  48 |     }), before);
  49 |     expect(after).toEqual({ changes: 0, outgoing: [] });
  50 |   }
  51 |   await fresh(page, 'after unload 界😀');
  52 |   await expect(page.locator('.xterm-rows')).toContainText('after unload');
  53 |   const count = await page.evaluate(() => demo.outgoing.length);
  54 |   await page.locator('.xterm-helper-textarea').press('z');
  55 |   await expect.poll(() => page.evaluate(count => demo.outgoing.slice(count), count)).toEqual(['z']);
  56 | });
  57 | 
  58 | for (const order of [['fit', 'search', 'serialize'], ['serialize', 'search', 'fit']]) {
  59 |   test(`addon combination ${order.join('/')} survives dispose and reopen`, async ({ page }, info) => {
  60 |     for (const name of order) {
  61 |       await page.locator('#addon-select').selectOption(name); await click(page, 'addon-load');
  62 |     }
  63 |     await fresh(page, '\x1b[31mRust combined 界😀\x1b[0m');
  64 |     expect(await page.evaluate(() => demo.addons.get('search').findNext('combined'))).toBe(true);
  65 |     expect(await page.evaluate(() => demo.terminal.getSelection())).toBe('combined');
  66 |     expect(await page.evaluate(() => demo.addons.get('serialize').serialize())).toContain('combined');
  67 |     await page.evaluate(() => { window.retiredTerminal = demo.terminal; window.retiredAddons = demo.addons; });
  68 |     await click(page, 'dispose');
  69 |     await expect(page.locator('.xterm')).toHaveCount(0);
  70 |     await click(page, 'reopen');
  71 |     await expect(page.locator('.xterm')).toHaveCount(1);
  72 |     expect(await page.evaluate(() => demo.snapshot().addons)).toEqual([]);
  73 |     const state = await snapshot(page);
  74 |     const stale = await page.evaluate(async () => {
  75 |       const errors = [];
  76 |       for (const [name, action] of [['write', () => retiredTerminal.write('STALE')], ['input', () => retiredTerminal.input('STALE', true)],
  77 |         ['addon-dispose', () => retiredAddons.dispose()]]) {
  78 |         try { action(); } catch (error) { errors.push({ name, message: String(error) }); }
  79 |       }
  80 |       await demo.write('');
  81 |       return errors;
  82 |     });
  83 |     await info.attach('stale-handle-results', { body: JSON.stringify(stale), contentType: 'application/json' });
  84 |     expect(await snapshot(page)).toEqual(state);
  85 |     for (const name of order) {
  86 |       await page.locator('#addon-select').selectOption(name); await click(page, 'addon-load');
  87 |     }
  88 |     await fresh(page, 'fresh combination');
  89 |     expect(await page.evaluate(() => demo.addons.get('search').findNext('fresh'))).toBe(true);
  90 |     const before = await page.evaluate(() => demo.outgoing.length);
  91 |     await page.locator('.xterm-helper-textarea').press('x');
  92 |     await expect.poll(() => page.evaluate(before => demo.outgoing.slice(before), before)).toEqual(['x']);
  93 |   });
  94 | }
  95 | 
```