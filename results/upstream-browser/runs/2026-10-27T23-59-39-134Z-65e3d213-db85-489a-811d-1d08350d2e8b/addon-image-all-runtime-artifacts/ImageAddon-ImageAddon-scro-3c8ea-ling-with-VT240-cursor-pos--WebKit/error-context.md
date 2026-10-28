# Instructions

- Following Playwright test failed.
- Explain why, be concise, respect Playwright best practices.
- Provide a snippet of code with the fix, if possible.

# Test info

- Name: ImageAddon.test.js >> ImageAddon >> scrolling & cursor modes >> testdata default (scrolling with VT240 cursor pos)
- Location: addons\addon-image\test\ImageAddon.test.ts:169:5

# Error details

```
AssertionError: Expected values to be strictly deep-equal:
+ actual - expected

  [
    0,
+   0
-   4
  ]

```

# Page snapshot

```yaml
- generic [ref=e6]:
  - textbox "Terminal input"
  - text: WWWWWWWWWWWWWWWWWWWWWWWWWWWWWWWW
```

# Test source

```ts
  72  |   [readFileSync('./addons/addon-image/fixture/iip/palette.iip', { encoding: 'utf-8' }), [640, 80]],
  73  |   [readFileSync('./addons/addon-image/fixture/iip/spinfox.iip', { encoding: 'utf-8' }), [148, 148]],
  74  |   [readFileSync('./addons/addon-image/fixture/iip/w3c_gif.iip', { encoding: 'utf-8' }), [72, 48]],
  75  |   [readFileSync('./addons/addon-image/fixture/iip/w3c_jpg.iip', { encoding: 'utf-8' }), [72, 48]],
  76  |   [readFileSync('./addons/addon-image/fixture/iip/w3c_png.iip', { encoding: 'utf-8' }), [72, 48]]
  77  | ];
  78  | const PALETTE_PNG_BASE64 = readFileSync('./addons/addon-image/fixture/palette.png').toString('base64');
  79  | const qoiEnc = new QoiEncoder(1024*1024);
  80  | const qoiData = qoiEnc.encode(TESTDATA.bytes, 640, 80);
  81  | const PALETTE_QOI_BASE64 = Buffer.from(qoiData).toString('base64');
  82  | 
  83  | let ctx: ITestContext;
  84  | test.beforeAll(async ({ browser }) => {
  85  |   ctx = await createTestContext(browser);
  86  |   await openTerminal(ctx, { cols: 80, rows: 24 });
  87  | });
  88  | test.afterAll(async () => await ctx.page.close());
  89  | 
  90  | test.describe('ImageAddon', () => {
  91  | 
  92  |   test.beforeEach(async ({}, testInfo) => {
  93  |     await ctx.page.evaluate(`
  94  |       window.term.reset()
  95  |       window.imageAddon?.dispose();
  96  |       window.imageAddon = new ImageAddon({ sixelPaletteLimit: 512 });
  97  |       window.term.loadAddon(window.imageAddon);
  98  |     `);
  99  |   });
  100 | 
  101 |   test('test for private accessors', async () => {
  102 |     // terminal privates
  103 |     const accessors = [
  104 |       '_core',
  105 |       '_core._renderService',
  106 |       '_core._inputHandler',
  107 |       '_core._inputHandler._parser',
  108 |       '_core._inputHandler._curAttrData',
  109 |       '_core._inputHandler._dirtyRowTracker',
  110 |       '_core._themeService.colors',
  111 |       '_core._coreBrowserService'
  112 |     ];
  113 |     for (const prop of accessors) {
  114 |       strictEqual(
  115 |         await ctx.page.evaluate('(() => { const v = window.term.' + prop + '; return v !== undefined && v !== null; })()'),
  116 |         true, `problem at ${prop}`
  117 |       );
  118 |     }
  119 |     // bufferline privates
  120 |     strictEqual(await ctx.page.evaluate('window.term._core.buffer.lines.get(0)._data instanceof Uint32Array'), true);
  121 |     strictEqual(await ctx.page.evaluate('window.term._core.buffer.lines.get(0)._extendedAttrs instanceof Object'), true);
  122 |     // inputhandler privates
  123 |     strictEqual(await ctx.page.evaluate('window.term._core._inputHandler._curAttrData.constructor.name'), '_AttributeData');
  124 |     strictEqual(await ctx.page.evaluate('window.term._core._inputHandler._parser.constructor.name'), 'EscapeSequenceParser');
  125 |   });
  126 | 
  127 |   test.describe('ctor options', () => {
  128 |     test('empty settings should load defaults', async () => {
  129 |       const DEFAULT_OPTIONS: IImageAddonOptions = {
  130 |         enableSizeReports: true,
  131 |         pixelLimit: 16777216,
  132 |         sixelSupport: true,
  133 |         sixelScrolling: true,
  134 |         sixelPaletteLimit: 512,  // set to 512 to get example image working
  135 |         sixelSizeLimit: 33554432,
  136 |         storageLimit: 128,
  137 |         showPlaceholder: true,
  138 |         iipSupport: true,
  139 |         iipSizeLimit: 33554432,
  140 |         kittySupport: true,
  141 |         kittySizeLimit: 33554432
  142 |       };
  143 |       deepStrictEqual(await ctx.page.evaluate(`window.imageAddon._opts`), DEFAULT_OPTIONS);
  144 |     });
  145 |     test('custom settings should overload defaults', async () => {
  146 |       const customSettings: IImageAddonOptions = {
  147 |         enableSizeReports: false,
  148 |         pixelLimit: 5,
  149 |         sixelSupport: false,
  150 |         sixelScrolling: false,
  151 |         sixelPaletteLimit: 1024,
  152 |         sixelSizeLimit: 1000,
  153 |         storageLimit: 10,
  154 |         showPlaceholder: false,
  155 |         iipSupport: false,
  156 |         iipSizeLimit: 1000,
  157 |         kittySupport: false,
  158 |         kittySizeLimit: 1000
  159 |       };
  160 |       await ctx.page.evaluate(opts => {
  161 |         (window as any).imageAddonCustom = new ImageAddon(opts.opts);
  162 |         (window as any).term.loadAddon((window as any).imageAddonCustom);
  163 |       }, { opts: customSettings });
  164 |       deepStrictEqual(await ctx.page.evaluate(`window.imageAddonCustom._opts`), customSettings);
  165 |     });
  166 |   });
  167 | 
  168 |   test.describe('scrolling & cursor modes', () => {
  169 |     test('testdata default (scrolling with VT240 cursor pos)', async () => {
  170 |       const dim = await getDimensions();
  171 |       await ctx.proxy.write(SIXEL_SEQ_0);
> 172 |       deepStrictEqual(await getCursor(), [0, Math.floor(TESTDATA.height/dim.cellHeight)]);
      |       ^ AssertionError: Expected values to be strictly deep-equal:
  173 |       // moved to right by 10 cells
  174 |       await ctx.proxy.write('#'.repeat(10) + SIXEL_SEQ_0);
  175 |       deepStrictEqual(await getCursor(), [10, Math.floor(TESTDATA.height/dim.cellHeight) * 2]);
  176 |     });
  177 |     test('write testdata noScrolling', async () => {
  178 |       await ctx.proxy.write('\x1b[?80h' + SIXEL_SEQ_0);
  179 |       deepStrictEqual(await getCursor(), [0, 0]);
  180 |       // second draw does not change anything
  181 |       await ctx.proxy.write(SIXEL_SEQ_0);
  182 |       deepStrictEqual(await getCursor(), [0, 0]);
  183 |     });
  184 |     test('testdata cursor always at VT240 pos', async () => {
  185 |       const dim = await getDimensions();
  186 |       // offset 0
  187 |       await ctx.proxy.write(SIXEL_SEQ_0);
  188 |       deepStrictEqual(await getCursor(), [0, Math.floor(TESTDATA.height/dim.cellHeight)]);
  189 |       // moved to right by 10 cells
  190 |       await ctx.proxy.write('#'.repeat(10) + SIXEL_SEQ_0);
  191 |       deepStrictEqual(await getCursor(), [10, Math.floor(TESTDATA.height/dim.cellHeight) * 2]);
  192 |       // moved by 30 cells (+10 prev)
  193 |       await ctx.proxy.write('#'.repeat(30) + SIXEL_SEQ_0);
  194 |       deepStrictEqual(await getCursor(), [10 + 30, Math.floor(TESTDATA.height/dim.cellHeight) * 3]);
  195 |     });
  196 |   });
  197 | 
  198 |   test.describe('image lifecycle & eviction', () => {
  199 |     test('onImageAdded fires for each image', async () => {
  200 |       await ctx.page.evaluate(`
  201 |         window._imageAddedCount = 0;
  202 |         window.imageAddon.onImageAdded(() => { window._imageAddedCount++; });
  203 |       `);
  204 |       await ctx.proxy.write(SIXEL_SEQ_0);
  205 |       await pollFor(ctx.page, 'window._imageAddedCount', 1);
  206 |       await ctx.proxy.write(SIXEL_SEQ_0);
  207 |       await pollFor(ctx.page, 'window._imageAddedCount', 2);
  208 |       await ctx.proxy.write(SIXEL_SEQ_0);
  209 |       await pollFor(ctx.page, 'window._imageAddedCount', 3);
  210 |     });
  211 |     test('delete image once scrolled off', async () => {
  212 |       await ctx.proxy.write(SIXEL_SEQ_0);
  213 |       pollFor(ctx.page, 'window.imageAddon._storage._images.size', 1);
  214 |       // scroll to scrollback + rows - 1
  215 |       await ctx.page.evaluate(
  216 |         scrollback => new Promise(res => (window as any).term.write('\n'.repeat(scrollback), res)),
  217 |         (await getScrollbackPlusRows() - 1)
  218 |       );
  219 |       // wait here, as we have to make sure, that eviction did not yet occur
  220 |       await timeout(100);
  221 |       pollFor(ctx.page, 'window.imageAddon._storage._images.size', 1);
  222 |       // scroll one further should delete the image
  223 |       await ctx.page.evaluate(() => new Promise(res => (window as any).term.write('\n', res)));
  224 |       pollFor(ctx.page, 'window.imageAddon._storage._images.size', 0);
  225 |     });
  226 |     test('get storageUsage', async () => {
  227 |       strictEqual(await ctx.page.evaluate('window.imageAddon.storageUsage'), 0);
  228 |       await ctx.proxy.write(SIXEL_SEQ_0);
  229 |       ok(Math.abs((await ctx.page.evaluate<number>('window.imageAddon.storageUsage')) - 640 * 80 * 4 / 1000000) < 0.05);
  230 |     });
  231 |     test('get/set storageLimit', async () => {
  232 |       strictEqual(await ctx.page.evaluate('window.imageAddon.storageLimit'), 128);
  233 |       strictEqual(await ctx.page.evaluate('window.imageAddon.storageLimit = 1'), 1);
  234 |       strictEqual(await ctx.page.evaluate('window.imageAddon.storageLimit'), 1);
  235 |     });
  236 |     test('remove images by storage limit pressure', async () => {
  237 |       strictEqual(await ctx.page.evaluate('window.imageAddon.storageLimit = 1'), 1);
  238 |       // never go beyond storage limit
  239 |       await ctx.proxy.write(SIXEL_SEQ_0);
  240 |       await ctx.proxy.write(SIXEL_SEQ_0);
  241 |       await ctx.proxy.write(SIXEL_SEQ_0);
  242 |       await ctx.proxy.write(SIXEL_SEQ_0);
  243 |       await timeout(100);
  244 |       const usage = await ctx.page.evaluate('window.imageAddon.storageUsage');
  245 |       await ctx.proxy.write(SIXEL_SEQ_0);
  246 |       await ctx.proxy.write(SIXEL_SEQ_0);
  247 |       await ctx.proxy.write(SIXEL_SEQ_0);
  248 |       await ctx.proxy.write(SIXEL_SEQ_0);
  249 |       await timeout(100);
  250 |       strictEqual(await ctx.page.evaluate('window.imageAddon.storageUsage'), usage);
  251 |       strictEqual(usage as number < 1, true);
  252 |     });
  253 |     test('set storageLimit removes images synchronously', async () => {
  254 |       await ctx.proxy.write(SIXEL_SEQ_0 + SIXEL_SEQ_0 + SIXEL_SEQ_0);
  255 |       const usage: number = await ctx.page.evaluate('window.imageAddon.storageUsage');
  256 |       const newUsage: number = await ctx.page.evaluate('window.imageAddon.storageLimit = 0.5; window.imageAddon.storageUsage');
  257 |       strictEqual(newUsage < usage, true);
  258 |       strictEqual(newUsage < 0.5, true);
  259 |     });
  260 |     test('clear alternate images on buffer change', async () => {
  261 |       strictEqual(await ctx.page.evaluate('window.imageAddon.storageUsage'), 0);
  262 |       await ctx.proxy.write('\x1b[?1049h' + SIXEL_SEQ_0);
  263 |       ok(Math.abs((await ctx.page.evaluate<number>('window.imageAddon.storageUsage')) - 640 * 80 * 4 / 1000000) < 0.05);
  264 |       await ctx.proxy.write('\x1b[?1049l');
  265 |       strictEqual(await ctx.page.evaluate('window.imageAddon.storageUsage'), 0);
  266 |     });
  267 |     test('evict tiles by in-place overwrites (only full overwrite tested)', async () => {
  268 |       await timeout(50);
  269 |       await ctx.proxy.write('\x1b[H' + SIXEL_SEQ_0 + '\x1b[100;100H');
  270 |       await timeout(50);
  271 |       let usage = await ctx.page.evaluate('window.imageAddon.storageUsage');
  272 |       while (usage === 0) {
```