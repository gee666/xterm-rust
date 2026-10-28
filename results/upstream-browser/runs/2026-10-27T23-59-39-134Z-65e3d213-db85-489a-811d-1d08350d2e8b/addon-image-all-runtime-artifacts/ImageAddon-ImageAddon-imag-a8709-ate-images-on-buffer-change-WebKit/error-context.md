# Instructions

- Following Playwright test failed.
- Explain why, be concise, respect Playwright best practices.
- Provide a snippet of code with the fix, if possible.

# Test info

- Name: ImageAddon.test.js >> ImageAddon >> image lifecycle & eviction >> clear alternate images on buffer change
- Location: addons\addon-image\test\ImageAddon.test.ts:260:5

# Error details

```
AssertionError: The expression evaluated to a falsy value:

  (Math.abs((await ctx.page.evaluate("window.imageAddon.storageUsage")) - 640 * 80 * 4 / 1e6) < 0.05)

```

# Page snapshot

```yaml
- generic [ref=e6]:
  - textbox "Terminal input" [ref=e7]
  - text: WWWWWWWWWWWWWWWWWWWWWWWWWWWWWWWW
```

# Test source

```ts
  163 |       }, { opts: customSettings });
  164 |       deepStrictEqual(await ctx.page.evaluate(`window.imageAddonCustom._opts`), customSettings);
  165 |     });
  166 |   });
  167 | 
  168 |   test.describe('scrolling & cursor modes', () => {
  169 |     test('testdata default (scrolling with VT240 cursor pos)', async () => {
  170 |       const dim = await getDimensions();
  171 |       await ctx.proxy.write(SIXEL_SEQ_0);
  172 |       deepStrictEqual(await getCursor(), [0, Math.floor(TESTDATA.height/dim.cellHeight)]);
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
> 263 |       ok(Math.abs((await ctx.page.evaluate<number>('window.imageAddon.storageUsage')) - 640 * 80 * 4 / 1000000) < 0.05);
      |       ^ AssertionError: The expression evaluated to a falsy value:
  264 |       await ctx.proxy.write('\x1b[?1049l');
  265 |       strictEqual(await ctx.page.evaluate('window.imageAddon.storageUsage'), 0);
  266 |     });
  267 |     test('evict tiles by in-place overwrites (only full overwrite tested)', async () => {
  268 |       await timeout(50);
  269 |       await ctx.proxy.write('\x1b[H' + SIXEL_SEQ_0 + '\x1b[100;100H');
  270 |       await timeout(50);
  271 |       let usage = await ctx.page.evaluate('window.imageAddon.storageUsage');
  272 |       while (usage === 0) {
  273 |         await timeout(50);
  274 |         usage = await ctx.page.evaluate('window.imageAddon.storageUsage');
  275 |       }
  276 |       await ctx.proxy.write('\x1b[H' + SIXEL_SEQ_0 + '\x1b[100;100H');
  277 |       await timeout(200); // wait some time and re-check
  278 |       strictEqual(await ctx.page.evaluate('window.imageAddon.storageUsage'), usage);
  279 |     });
  280 |     test('manual eviction on alternate buffer must not miss images', async () => {
  281 |       await ctx.proxy.write('\x1b[?1049h');
  282 |       await ctx.proxy.write(SIXEL_SEQ_0 + SIXEL_SEQ_0 + SIXEL_SEQ_0 + SIXEL_SEQ_0 + SIXEL_SEQ_0 + SIXEL_SEQ_0);
  283 |       await timeout(100);
  284 |       const usage: number = await ctx.page.evaluate('window.imageAddon.storageUsage');
  285 |       await ctx.proxy.write(SIXEL_SEQ_0 + SIXEL_SEQ_0 + SIXEL_SEQ_0 + SIXEL_SEQ_0 + SIXEL_SEQ_0 + SIXEL_SEQ_0);
  286 |       await ctx.proxy.write(SIXEL_SEQ_0 + SIXEL_SEQ_0 + SIXEL_SEQ_0 + SIXEL_SEQ_0 + SIXEL_SEQ_0 + SIXEL_SEQ_0);
  287 |       await timeout(100);
  288 |       const newUsage: number = await ctx.page.evaluate('window.imageAddon.storageUsage');
  289 |       strictEqual(newUsage, usage);
  290 |     });
  291 |   });
  292 | 
  293 |   test.describe('IIP support', () => {
  294 |     test('size params is optional/informative', async () => {
  295 |       // too small size param
  296 |       await ctx.proxy.write(`\x1b]1337;File=inline=1;size=5:${PALETTE_PNG_BASE64}\x07`);
  297 |       deepStrictEqual(await getOrigSize(1), [640, 80]);
  298 |       // no size aparm at all
  299 |       await ctx.proxy.write(`\x1b[10H\x1b]1337;File=inline=1:${PALETTE_QOI_BASE64}\x07`);
  300 |       deepStrictEqual(await getOrigSize(2), [640, 80]);
  301 |     });
  302 |     test.skip('FilePart support - bytewise', async () => {
  303 |       // opener
  304 |       await ctx.proxy.write(`\x1b]1337;MultipartFile=inline=1\x07`);
  305 |       // byte-wise
  306 |       for (let i = 0; i < PALETTE_PNG_BASE64.length; ++i) {
  307 |         await ctx.proxy.write(`\x1b]1337;FilePart=${PALETTE_PNG_BASE64[i]}\x07`);
  308 |       }
  309 |       // finalizer
  310 |       await ctx.proxy.write(`\x1b]1337;FileEnd\x07`);
  311 |       deepStrictEqual(await getOrigSize(1), [640, 80]);
  312 |     });
  313 |     test('FilePart support - chunks', async () => {
  314 |       // opener
  315 |       await ctx.proxy.write(`\x1b]1337;MultipartFile=inline=1\x07`);
  316 |       // chunks
  317 |       for (let i = 0; i < PALETTE_PNG_BASE64.length; i += 256) {
  318 |         await ctx.proxy.write(`\x1b]1337;FilePart=${PALETTE_PNG_BASE64.slice(i, i + 256)}\x07`);
  319 |       }
  320 |       // finalizer
  321 |       await ctx.proxy.write(`\x1b]1337;FileEnd\x07`);
  322 |       deepStrictEqual(await getOrigSize(1), [640, 80]);
  323 |     });
  324 |     test('FilePart support - edgecases', async () => {
  325 |       // multiple opener do not harm
  326 |       for (let i = 0; i < 10; ++i) {
  327 |         await ctx.proxy.write(`\x1b]1337;MultipartFile=inline=1\x07`);
  328 |       }
  329 |       // multiple finalizer do not harm
  330 |       for (let i = 0; i < 5; ++i) {
  331 |         await ctx.proxy.write(`\x1b]1337;FileEnd\x07`);
  332 |       }
  333 |       // chunks w'o opener are ignored
  334 |       for (let i = 0; i < PALETTE_PNG_BASE64.length; i += 256) {
  335 |         await ctx.proxy.write(`\x1b]1337;FilePart=${PALETTE_PNG_BASE64.slice(i, i + 256)}\x07`);
  336 |       }
  337 |       await ctx.proxy.write(`\x1b]1337;FileEnd\x07`);
  338 |       pollFor(ctx.page, 'window.imageAddon._storage._images.size', 0);
  339 |     });
  340 |     test('FilePart support - other IIP sequence cancels chunked image', async () => {
  341 |       // opener
  342 |       await ctx.proxy.write(`\x1b]1337;MultipartFile=inline=1\x07`);
  343 |       // chunks
  344 |       for (let i = 0; i < PALETTE_PNG_BASE64.length; i += 256) {
  345 |         await ctx.proxy.write(`\x1b]1337;FilePart=${PALETTE_PNG_BASE64.slice(i, i + 256)}\x07`);
  346 |       }
  347 |       // finalizer skipped
  348 |       // write spinfox.png
  349 |       await ctx.proxy.write(TESTDATA_IIP[1][0]);
  350 |       deepStrictEqual(await getOrigSize(1), TESTDATA_IIP[1][1]);
  351 |     });
  352 |   });
  353 | 
  354 |   test.describe('IIP support - testimages', () => {
  355 |     test('palette.png', async () => {
  356 |       await ctx.proxy.write(TESTDATA_IIP[0][0]);
  357 |       deepStrictEqual(await getOrigSize(1), TESTDATA_IIP[0][1]);
  358 |     });
  359 |     test('spinfox.png', async () => {
  360 |       await ctx.proxy.write(TESTDATA_IIP[1][0]);
  361 |       deepStrictEqual(await getOrigSize(1), TESTDATA_IIP[1][1]);
  362 |     });
  363 |     test('w3c gif', async () => {
```