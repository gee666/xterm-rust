# Instructions

- Following Playwright test failed.
- Explain why, be concise, respect Playwright best practices.
- Provide a snippet of code with the fix, if possible.

# Test info

- Name: InputHandler.test.ts >> InputHandler Integration Tests >> CSI >> CSI Ps @ - ICH: Insert Ps (Blank) Character(s) (default = 1)
- Location: test\playwright\InputHandler.test.ts:35:9

# Error details

```
"beforeAll" hook timeout of 10000ms exceeded.
```

```
TypeError: Cannot read properties of undefined (reading 'page')
```

# Test source

```ts
  1   | /**
  2   |  * Copyright (c) 2019 The xterm.js authors. All rights reserved.
  3   |  * @license MIT
  4   |  */
  5   | import { test } from '@playwright/test';
  6   | import { deepStrictEqual, ok } from 'assert';
  7   | import { ITestContext, createTestContext, openTerminal, pollFor } from './TestUtils';
  8   | 
  9   | let ctx: ITestContext;
  10  | test.beforeAll(async ({ browser }) => {
  11  |   ctx = await createTestContext(browser);
  12  |   await openTerminal(ctx);
  13  | });
> 14  | test.afterAll(async () => await ctx.page.close());
      |                                     ^ TypeError: Cannot read properties of undefined (reading 'page')
  15  | 
  16  | 
  17  | test.describe('InputHandler Integration Tests', () => {
  18  |   let recordedData: string[];
  19  |   let recordDataDisposable: { dispose: () => void };
  20  |   test.beforeAll(async () => {
  21  |     recordedData = [];
  22  |     recordDataDisposable = ctx.proxy.onData(d => recordedData.push(d));
  23  |   });
  24  |   test.afterAll(async () => {
  25  |     recordDataDisposable.dispose();
  26  |   });
  27  |   test.beforeEach(async () => {
  28  |     recordedData.length = 0;
  29  |     await ctx.proxy.resize(80, 24);
  30  |   });
  31  | 
  32  |   test.describe('CSI', () => {
  33  |     test.beforeEach(async () => await ctx.proxy.reset());
  34  | 
  35  |     test('CSI Ps @ - ICH: Insert Ps (Blank) Character(s) (default = 1)', async () => {
  36  |       // Default
  37  |       await ctx.proxy.write('foo\x1b[3D\x1b[@\n\r');
  38  |       // Explicit
  39  |       await ctx.proxy.write('bar\x1b[3D\x1b[4@');
  40  |       await pollFor(ctx.page, () => getLinesAsArray(2), [' foo', '    bar']);
  41  |     });
  42  |     test('CSI Ps SP @ - SL: Shift left Ps columns(s) (default = 1), ECMA-48', async () => {
  43  |       // Default
  44  |       await ctx.proxy.write('abcdefg\x1b[ @');
  45  |       await pollFor(ctx.page, () => getLinesAsArray(1), ['bcdefg']);
  46  |       // Explicit
  47  |       await ctx.proxy.reset();
  48  |       await ctx.proxy.write('abcdefg\x1b[3 @');
  49  |       await pollFor(ctx.page, () => getLinesAsArray(1), ['defg']);
  50  |     });
  51  |     test('CSI Ps A - CUU: Cursor Up Ps Times (default = 1)', async () => {
  52  |       // Default
  53  |       await ctx.proxy.write('\n\n\n\n\x1b[Aa');
  54  |       // Explicit
  55  |       await ctx.proxy.write('\x1b[2Ab');
  56  |       await pollFor(ctx.page, () => getLinesAsArray(4), ['', ' b', '', 'a']);
  57  |     });
  58  |     test('CSI Ps B - CUD: Cursor Down Ps Times (default = 1)', async () => {
  59  |       // Default
  60  |       await ctx.proxy.write('\x1b[Ba');
  61  |       // Explicit
  62  |       await ctx.proxy.write('\x1b[2Bb');
  63  |       await pollFor(ctx.page, () => getLinesAsArray(4), ['', 'a', '', ' b']);
  64  |     });
  65  |     test('CSI Ps C - CUF: Cursor Forward Ps Times (default = 1)', async () => {
  66  |       // Default
  67  |       await ctx.proxy.write('\x1b[Ca');
  68  |       // Explicit
  69  |       await ctx.proxy.write('\x1b[2Cb');
  70  |       await pollFor(ctx.page, () => getLinesAsArray(1), [' a  b']);
  71  |     });
  72  |     test('CSI Ps D - CUB: Cursor Backward Ps Times (default = 1)', async () => {
  73  |       // Default
  74  |       await ctx.proxy.write('foo\x1b[Da');
  75  |       // Explicit
  76  |       await ctx.proxy.write('\x1b[2Db');
  77  |       await pollFor(ctx.page, () => getLinesAsArray(1), ['fba']);
  78  |     });
  79  |     test('CSI Ps E - CNL: Cursor Next Line Ps Times (default = 1)', async () => {
  80  |       // Default
  81  |       await ctx.proxy.write('\x1b[Ea');
  82  |       // Explicit
  83  |       await ctx.proxy.write('\x1b[2Eb');
  84  |       await pollFor(ctx.page, () => getLinesAsArray(4), ['', 'a', '', 'b']);
  85  |     });
  86  |     test('CSI Ps F - CPL: Cursor Preceding Line Ps Times (default = 1)', async () => {
  87  |       // Default
  88  |       await ctx.proxy.write('\n\n\n\n\x1b[Fa');
  89  |       // Explicit
  90  |       await ctx.proxy.write('\x1b[2Fb');
  91  |       await pollFor(ctx.page, () => getLinesAsArray(5), ['', 'b', '', 'a', '']);
  92  |     });
  93  |     test('CSI Ps G - CHA: Cursor Character Absolute [column] (default = [row,1])', async () => {
  94  |       // Default
  95  |       await ctx.proxy.write('foo\x1b[Ga');
  96  |       // Explicit
  97  |       await ctx.proxy.write('\x1b[10Gb');
  98  |       await pollFor(ctx.page, () => getLinesAsArray(1), ['aoo      b']);
  99  |     });
  100 |     test('CSI Ps ; Ps H - CUP: Cursor Position [row;column] (default = [1,1])', async () => {
  101 |       // Default
  102 |       await ctx.proxy.write('foo\x1b[Ha');
  103 |       // Explicit
  104 |       await ctx.proxy.write('\x1b[3;3Hb');
  105 |       await pollFor(ctx.page, () => getLinesAsArray(3), ['aoo', '', '  b']);
  106 |     });
  107 |     test('CSI Ps I - CHT: Cursor Forward Tabulation Ps tab stops (default = 1)', async () => {
  108 |       // Default
  109 |       await ctx.proxy.write('\x1b[Ia');
  110 |       // Explicit
  111 |       await ctx.proxy.write('\n\r\x1b[2Ib');
  112 |       await pollFor(ctx.page, () => getLinesAsArray(2), ['        a', '                b']);
  113 |     });
  114 |     test('CSI Ps J - ED: Erase in Display, VT100', async () => {
```