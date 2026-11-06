# Instructions

- Following Playwright test failed.
- Explain why, be concise, respect Playwright best practices.
- Provide a snippet of code with the fix, if possible.

# Test info

- Name: FitAddon.test.ts >> FitAddon >> no terminal
- Location: addons\addon-fit\test\FitAddon.test.ts:42:7

# Error details

```
"beforeAll" hook timeout of 10000ms exceeded.
```

```
Error: page.setViewportSize: Target page, context or browser has been closed
```

# Test source

```ts
  1   | /**
  2   |  * Copyright (c) 2019 The xterm.js authors. All rights reserved.
  3   |  * @license MIT
  4   |  */
  5   | 
  6   | import test from '@playwright/test';
  7   | import { deepEqual, ok, strictEqual } from 'assert';
  8   | import { ITestContext, createTestContext, openTerminal, timeout } from '../../../test/playwright/TestUtils';
  9   | 
  10  | let ctx: ITestContext;
  11  | test.beforeAll(async ({ browser }) => {
  12  |   ctx = await createTestContext(browser);
> 13  |   ctx.page.setViewportSize({ width: 1024, height: 768 });
      |            ^ Error: page.setViewportSize: Target page, context or browser has been closed
  14  |   await openTerminal(ctx);
  15  | });
  16  | test.afterAll(async () => await ctx.page.close());
  17  | 
  18  | test.describe('FitAddon', () => {
  19  |   test.beforeEach(async function(): Promise<any> {
  20  |     await ctx.page.evaluate(`document.querySelector('#terminal-container').style.display=''`);
  21  |     await ctx.page.evaluate(`
  22  |       window.term.reset()
  23  |       window.fit?.dispose();
  24  |       window.fit = new FitAddon();
  25  |       window.term.loadAddon(window.fit);
  26  |     `);
  27  |   });
  28  | 
  29  |   // test.beforeEach(async function(): Promise<any> {
  30  |   //   await ctx.page.evaluate(`document.querySelector('#terminal-container').style.display=''`);
  31  |   //   await openTerminal(page);
  32  |   // });
  33  | 
  34  |   // after(async () => {
  35  |   //   await browser.close();
  36  |   // });
  37  | 
  38  |   // afterEach(async function(): Promise<any> {
  39  |   //   await ctx.proxy.dispose();
  40  |   // });
  41  | 
  42  |   test('no terminal', async function(): Promise<any> {
  43  |     await ctx.page.evaluate(`window.fit2 = new FitAddon();`);
  44  |     strictEqual(await ctx.page.evaluate(`window.fit2.proposeDimensions()`), undefined);
  45  |     await ctx.page.evaluate(`window.fit2.dispose();`);
  46  |   });
  47  | 
  48  |   test.describe('proposeDimensions', () => {
  49  |     test('default', async function(): Promise<any> {
  50  |       await setDimensions();
  51  |       const dimensions: {cols: number, rows: number} = await ctx.page.evaluate(`window.fit.proposeDimensions()`);
  52  |       ok(dimensions.cols > 85);
  53  |       ok(dimensions.cols < 88);
  54  |       ok(dimensions.rows > 24);
  55  |       ok(dimensions.rows < 29);
  56  |     });
  57  | 
  58  |     test('width', async function(): Promise<any> {
  59  |       await setDimensions(1008);
  60  |       const dimensions: {cols: number, rows: number} = await ctx.page.evaluate(`window.fit.proposeDimensions()`);
  61  |       ok(dimensions.cols > 108);
  62  |       ok(dimensions.cols < 111);
  63  |       ok(dimensions.rows > 24);
  64  |       ok(dimensions.rows < 29);
  65  |     });
  66  | 
  67  |     test('small', async function(): Promise<any> {
  68  |       await setDimensions(1, 1);
  69  |       deepEqual(await ctx.page.evaluate(`window.fit.proposeDimensions()`), {
  70  |         cols: 2,
  71  |         rows: 1
  72  |       });
  73  |     });
  74  | 
  75  |     test('hidden', async function(): Promise<any> {
  76  |       await ctx.proxy.dispose();
  77  |       await ctx.page.evaluate(`document.querySelector('#terminal-container').style.display='none'`);
  78  |       await ctx.page.evaluate(`window.term = new Terminal()`);
  79  |       await ctx.page.evaluate(`window.term.open(document.querySelector('#terminal-container'))`);
  80  |       await setDimensions();
  81  |       const dimensions: { cols: number, rows: number } | undefined = await ctx.page.evaluate(`window.fit.proposeDimensions()`);
  82  |       // The value of dims will be undefined if the char measure strategy falls back to the DOM
  83  |       // method, so only assert if it's not undefined.
  84  |       if (dimensions) {
  85  |         ok(dimensions.cols > 85);
  86  |         ok(dimensions.cols < 88);
  87  |         ok(dimensions.rows > 24);
  88  |         ok(dimensions.rows < 29);
  89  |       }
  90  |     });
  91  |   });
  92  | 
  93  |   test.describe('fit', () => {
  94  |     test('default', async function(): Promise<any> {
  95  |       await setDimensions();
  96  |       await ctx.page.evaluate(`window.fit.fit()`);
  97  |       const cols: number = await ctx.proxy.cols;
  98  |       const rows: number = await ctx.proxy.rows;
  99  |       ok(cols > 85);
  100 |       ok(cols < 88);
  101 |       ok(rows > 24);
  102 |       ok(rows < 29);
  103 |     });
  104 | 
  105 |     test('width', async function(): Promise<any> {
  106 |       await setDimensions(1008);
  107 |       await ctx.page.evaluate(`window.fit.fit()`);
  108 |       const cols: number = await ctx.proxy.cols;
  109 |       const rows: number = await ctx.proxy.rows;
  110 |       ok(cols > 108);
  111 |       ok(cols < 111);
  112 |       ok(rows > 24);
  113 |       ok(rows < 29);
```