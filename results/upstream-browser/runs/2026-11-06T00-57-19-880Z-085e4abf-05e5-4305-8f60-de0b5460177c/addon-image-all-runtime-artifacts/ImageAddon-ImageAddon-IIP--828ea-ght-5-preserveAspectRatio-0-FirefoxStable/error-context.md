# Instructions

- Following Playwright test failed.
- Explain why, be concise, respect Playwright best practices.
- Provide a snippet of code with the fix, if possible.

# Test info

- Name: ImageAddon.test.js >> ImageAddon >> IIP - resizing >> palette.qoi: N --> width=20 height=5 preserveAspectRatio=0
- Location: addons\addon-image\test\ImageAddon.test.ts:404:7

# Error details

```
AssertionError: Expected values to be strictly deep-equal:
+ actual - expected

  [
+   165,
-   160,
    90
  ]

```

# Page snapshot

```yaml
- textbox "Terminal input" [ref=e6]
```

# Test source

```ts
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
  364 |       await ctx.proxy.write(TESTDATA_IIP[2][0]);
  365 |       deepStrictEqual(await getOrigSize(1), TESTDATA_IIP[2][1]);
  366 |     });
  367 |     test('w3c jpeg', async () => {
  368 |       await ctx.proxy.write(TESTDATA_IIP[3][0]);
  369 |       deepStrictEqual(await getOrigSize(1), TESTDATA_IIP[3][1]);
  370 |     });
  371 |     test('w3c png', async () => {
  372 |       await ctx.proxy.write(TESTDATA_IIP[4][0]);
  373 |       deepStrictEqual(await getOrigSize(1), TESTDATA_IIP[4][1]);
  374 |     });
  375 |   });
  376 | 
  377 |   test.describe('IIP - QOI support', () => {
  378 |     test('palette should yield same bytes from PNG and QOI', async () => {
  379 |       await ctx.proxy.write(`\x1b]1337;File=inline=1;size=525:${PALETTE_PNG_BASE64}\x07`);
  380 |       deepStrictEqual(await getOrigSize(1), [640, 80]);
  381 |       await ctx.proxy.write(`\x1b[10H\x1b]1337;File=inline=1;size=${qoiData.length}:${PALETTE_QOI_BASE64}\x07`);
  382 |       deepStrictEqual(await getOrigSize(2), [640, 80]);
  383 |       const pngScrape = await getImageAtBufferCell(0, 0);
  384 |       const qoiScrape = await getImageAtBufferCell(0, 11);
  385 |       deepStrictEqual(qoiScrape, pngScrape);
  386 |     });
  387 |   });
  388 | 
  389 |   test.describe('IIP - resizing', () => {
  390 |     /**
  391 |      * The correct resize behavior is wonky and needs to be tested against iTerm2,
  392 |      * especially for missing params to derive correct default behavior.
  393 |      * We document the current behavior here w'o claiming to be fully in line with iTerm2.
  394 |      * ref: https://iterm2.com/documentation-images.html
  395 |      * imgcat: https://iterm2.com/utilities/imgcat
  396 |      *
  397 |      * NOTE: QOI has a slightly different parse path, thus we test resizing explicitly
  398 |      */
  399 |     const images = [
  400 |       ['palette.png', 525, PALETTE_PNG_BASE64],
  401 |       ['palette.qoi', qoiData.length, PALETTE_QOI_BASE64]
  402 |     ];
  403 |     for (const [name, size, payload] of images) {
  404 |       test(name + ': N --> width=20 height=5 preserveAspectRatio=0', async () => {
  405 |         // cell based resize
  406 |         const header = 'width=20;height=5;preserveAspectRatio=0';
  407 |         await ctx.proxy.write(`\x1b]1337;File=inline=1;size=${size};${header}:${payload}\x07`);
  408 |         const dim = await getDimensions();
> 409 |         deepStrictEqual(await getOrigSize(1), [dim.cellWidth * 20, dim.cellHeight * 5]);
      |         ^ AssertionError: Expected values to be strictly deep-equal:
  410 |       });
  411 |       test(name + ': Npx --> width=320px height=160px preserveAspectRatio=0', async () => {
  412 |         // pixel based resize
  413 |         const header = 'width=320px;height=160px;preserveAspectRatio=0';
  414 |         await ctx.proxy.write(`\x1b]1337;File=inline=1;size=${size};${header}:${payload}\x07`);
  415 |         deepStrictEqual(await getOrigSize(1), [320, 160]);
  416 |       });
  417 |       test(name + ': N% --> width=50% height=30% preserveAspectRatio=0', async () => {
  418 |         // % of viewport resize
  419 |         const header = 'width=50%;height=30%;preserveAspectRatio=0';
  420 |         await ctx.proxy.write(`\x1b]1337;File=inline=1;size=${size};${header}:${payload}\x07`);
  421 |         const dim = await getDimensions();
  422 |         deepStrictEqual(await getOrigSize(1), [Math.floor(dim.width * 0.5), Math.floor(dim.height * 0.3)]);
  423 |       });
  424 |       test(name + ': ommitted dimension assumes preserveAspectRatio=1', async () => {
  425 |         // width provided in percent
  426 |         const header = 'width=50%';
  427 |         await ctx.proxy.write(`\x1b]1337;File=inline=1;size=${size};${header}:${payload}\x07`);
  428 |         const dim = await getDimensions();
  429 |         const width = Math.floor(dim.width * 0.5);
  430 |         deepStrictEqual(await getOrigSize(1), [width, Math.floor(width * 80 / 640)]);
  431 |         // height provided in pixel
  432 |         const header2 = 'height=200px';
  433 |         await ctx.proxy.write(`\x1b]1337;File=inline=1;size=${size};${header2}:${payload}\x07`);
  434 |         deepStrictEqual(await getOrigSize(2), [Math.floor(200 * 640 / 80), 200]);
  435 |       });
  436 |     }
  437 |   });
  438 | 
  439 |   test.describe('text overwrite removes tiles', () => {
  440 |     test('sixel', async () => {
  441 |       await assertTextClearsTiles(SIXEL_SEQ_0);
  442 |     });
  443 |     test('iip', async () => {
  444 |       await assertTextClearsTiles(TESTDATA_IIP[0][0]);
  445 |     });
  446 |   });
  447 | });
  448 | 
  449 | /**
  450 |  * terminal access helpers.
  451 |  */
  452 | async function getDimensions(): Promise<IDimensions> {
  453 |   const dimensions: any = await ctx.page.evaluate(`term.dimensions`);
  454 |   return {
  455 |     cellWidth: Math.round(dimensions.css.cell.width),
  456 |     cellHeight: Math.round(dimensions.css.cell.height),
  457 |     width: Math.round(dimensions.css.canvas.width),
  458 |     height: Math.round(dimensions.css.canvas.height)
  459 |   };
  460 | }
  461 | 
  462 | async function getCursor(): Promise<[number, number]> {
  463 |   return ctx.page.evaluate('[window.term.buffer.active.cursorX, window.term.buffer.active.cursorY]');
  464 | }
  465 | 
  466 | async function getImageStorageLength(): Promise<number> {
  467 |   return ctx.page.evaluate('window.imageAddon._storage._images.size');
  468 | }
  469 | 
  470 | async function getScrollbackPlusRows(): Promise<number> {
  471 |   return ctx.page.evaluate('window.term.options.scrollback + window.term.rows');
  472 | }
  473 | 
  474 | async function getOrigSize(id: number): Promise<[number, number]> {
  475 |   return ctx.page.evaluate<any>(`[
  476 |     window.imageAddon._storage._images.get(${id}).orig.width,
  477 |     window.imageAddon._storage._images.get(${id}).orig.height
  478 |   ]`);
  479 | }
  480 | 
  481 | async function getImageAtBufferCell(x: number, y: number): Promise<string | undefined> {
  482 |   return ctx.page.evaluate<any>(`window.imageAddon.getImageAtBufferCell(${x}, ${y})?.toDataURL('image/png')`);
  483 | }
  484 | 
  485 | async function hasTileAtBufferCell(x: number, y: number): Promise<boolean> {
  486 |   return ctx.page.evaluate(`!!window.imageAddon.extractTileAtBufferCell(${x}, ${y})`);
  487 | }
  488 | 
  489 | async function assertTextClearsTiles(imageSeq: string): Promise<void> {
  490 |   await ctx.proxy.write('\x1b[H' + imageSeq);
  491 |   await pollFor(ctx.page, '!!window.imageAddon.extractTileAtBufferCell(5, 1)', true);
  492 |   ok(await hasTileAtBufferCell(0, 1));
  493 |   await ctx.proxy.write('\x1b[2;6H#######');
  494 |   for (let x = 5; x < 12; x++) {
  495 |     strictEqual(await hasTileAtBufferCell(x, 1), false);
  496 |   }
  497 |   ok(await hasTileAtBufferCell(0, 1));
  498 |   ok(await hasTileAtBufferCell(13, 1));
  499 |   ok(((await ctx.page.evaluate('window.term.buffer.active.getLine(1).translateToString(true)')) as string).includes('#######'));
  500 | }
  501 | 
```