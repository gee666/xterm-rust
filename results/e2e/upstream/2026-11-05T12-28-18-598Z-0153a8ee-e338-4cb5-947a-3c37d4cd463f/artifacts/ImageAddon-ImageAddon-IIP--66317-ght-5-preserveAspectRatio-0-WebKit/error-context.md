# Instructions

- Following Playwright test failed.
- Explain why, be concise, respect Playwright best practices.
- Provide a snippet of code with the fix, if possible.

# Test info

- Name: ImageAddon.test.ts >> ImageAddon >> IIP - resizing >> palette.png: N --> width=20 height=5 preserveAspectRatio=0
- Location: addons\addon-image\test\ImageAddon.test.ts:404:11

# Error details

```
Error: page.evaluate: ReferenceError: Can't find variable: ImageAddon
```

# Page snapshot

```yaml
- generic [ref=e6]:
  - textbox "Terminal input"
  - text: WWWWWWWWWWWWWWWWWWWWWWWWWWWWWWWW
```

# Test source

```ts
  1   | /**
  2   |  * Copyright (c) 2020 The xterm.js authors. All rights reserved.
  3   |  * @license MIT
  4   |  */
  5   | 
  6   | import test from '@playwright/test';
  7   | import { readFileSync } from 'fs';
  8   | import { FINALIZER, introducer, sixelEncode } from 'sixel';
  9   | import { ITestContext, createTestContext, openTerminal, pollFor, timeout } from '../../../test/playwright/TestUtils';
  10  | import { deepStrictEqual, ok, strictEqual } from 'assert';
  11  | import QoiEncoder from 'xterm-wasm-parts/lib/qoi/QoiEncoder.wasm';
  12  | 
  13  | /**
  14  |  * Plugin ctor options.
  15  |  */
  16  | export interface IImageAddonOptions {
  17  |   enableSizeReports: boolean;
  18  |   pixelLimit: number;
  19  |   storageLimit: number;
  20  |   showPlaceholder: boolean;
  21  |   sixelSupport: boolean;
  22  |   sixelScrolling: boolean;
  23  |   sixelPaletteLimit: number;
  24  |   sixelSizeLimit: number;
  25  |   iipSupport: boolean;
  26  |   iipSizeLimit: number;
  27  |   kittySupport: boolean;
  28  |   kittySizeLimit: number;
  29  | }
  30  | 
  31  | // eslint-disable-next-line
  32  | declare const ImageAddon: {
  33  |   new(options?: Partial<IImageAddonOptions>): any;
  34  | };
  35  | 
  36  | interface ITestData {
  37  |   width: number;
  38  |   height: number;
  39  |   bytes: Uint8Array;
  40  |   palette: number[];
  41  |   sixel: string;
  42  | }
  43  | 
  44  | interface IDimensions {
  45  |   cellWidth: number;
  46  |   cellHeight: number;
  47  |   width: number;
  48  |   height: number;
  49  | }
  50  | 
  51  | // image: 640 x 80, 512 color
  52  | const TESTDATA: ITestData = (() => {
  53  |   const data8 = readFileSync('./addons/addon-image/fixture/palette.blob');
  54  |   const data32 = new Uint32Array(data8.buffer);
  55  |   const palette = new Set<number>();
  56  |   for (let i = 0; i < data32.length; ++i) palette.add(data32[i]);
  57  |   const sixel = sixelEncode(data8, 640, 80, [...palette]);
  58  |   return {
  59  |     width: 640,
  60  |     height: 80,
  61  |     bytes: data8,
  62  |     palette: [...palette],
  63  |     sixel
  64  |   };
  65  | })();
  66  | const SIXEL_SEQ_0 = introducer(0) + TESTDATA.sixel + FINALIZER;
  67  | // const SIXEL_SEQ_1 = introducer(1) + TESTDATA.sixel + FINALIZER;
  68  | // const SIXEL_SEQ_2 = introducer(2) + TESTDATA.sixel + FINALIZER;
  69  | 
  70  | // NOTE: the data is loaded as string for easier transport through playwright
  71  | const TESTDATA_IIP: [string, [number, number]][] = [
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
> 93  |     await ctx.page.evaluate(`
      |                    ^ Error: page.evaluate: ReferenceError: Can't find variable: ImageAddon
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
```