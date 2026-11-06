# Instructions

- Following Playwright test failed.
- Explain why, be concise, respect Playwright best practices.
- Provide a snippet of code with the fix, if possible.

# Test info

- Name: Terminal.test.ts >> API Integration Tests >> Default options
- Location: test\playwright\Terminal.test.ts:19:7

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
  6   | import { deepStrictEqual, notStrictEqual, strictEqual } from 'assert';
  7   | import { ITestContext, createTestContext, openTerminal, pollFor, timeout } from './TestUtils';
  8   | import { IRenderDimensions } from '../../src/browser/renderer/shared/Types';
  9   | 
  10  | let ctx: ITestContext;
  11  | test.beforeAll(async ({ browser }) => {
  12  |   ctx = await createTestContext(browser);
  13  |   await openTerminal(ctx);
  14  | });
> 15  | test.afterAll(async () => await ctx.page.close());
      |                                     ^ TypeError: Cannot read properties of undefined (reading 'page')
  16  | 
  17  | 
  18  | test.describe('API Integration Tests', () => {
  19  |   test('Default options', async () => {
  20  |     strictEqual(await ctx.proxy.cols, 80);
  21  |     strictEqual(await ctx.proxy.rows, 24);
  22  |   });
  23  | 
  24  |   test('Proposed API check', async () => {
  25  |     await openTerminal(ctx, { allowProposedApi: false }, { loadUnicodeGraphemesAddon: false });
  26  |     await ctx.page.evaluate(`
  27  |       try {
  28  |         window.term.unicode;
  29  |       } catch (e) {
  30  |         window.throwMessage = e.message;
  31  |       }
  32  |     `);
  33  |     await pollFor(ctx.page, 'window.throwMessage', 'You must set the allowProposedApi option to true to use proposed API');
  34  |   });
  35  | 
  36  |   test('write', async () => {
  37  |     await openTerminal(ctx);
  38  |     await ctx.proxy.write('foo');
  39  |     await ctx.proxy.write('bar');
  40  |     await ctx.proxy.write('文');
  41  |     strictEqual(await (await ctx.proxy.buffer.active.getLine(0))!.translateToString(true), 'foobar文');
  42  |   });
  43  | 
  44  |   test('write with callback', async () => {
  45  |     await openTerminal(ctx);
  46  |     await ctx.page.evaluate(`
  47  |       window.term.write('foo', () => { window.__x = 'a'; });
  48  |       window.term.write('bar', () => { window.__x += 'b'; });
  49  |       window.term.write('文', () => { window.__x += 'c'; });
  50  |     `);
  51  |     await pollFor(ctx.page, `window.term.buffer.active.getLine(0).translateToString(true)`, 'foobar文');
  52  |     await pollFor(ctx.page, `window.__x`, 'abc');
  53  |   });
  54  | 
  55  |   test('write - bytes (UTF8)', async () => {
  56  |     await openTerminal(ctx);
  57  |     await ctx.page.evaluate(`
  58  |       window.term.write(new Uint8Array([102, 111, 111])); // foo
  59  |       window.term.write(new Uint8Array([98, 97, 114])); // bar
  60  |       window.term.write(new Uint8Array([230, 150, 135])); // 文
  61  |     `);
  62  |     await pollFor(ctx.page, `window.term.buffer.active.getLine(0).translateToString(true)`, 'foobar文');
  63  |   });
  64  | 
  65  |   test('write - bytes (UTF8) with callback', async () => {
  66  |     await openTerminal(ctx);
  67  |     await ctx.page.evaluate(`
  68  |       window.term.write(new Uint8Array([102, 111, 111]), () => { window.__x = 'A'; }); // foo
  69  |       window.term.write(new Uint8Array([98, 97, 114]), () => { window.__x += 'B'; }); // bar
  70  |       window.term.write(new Uint8Array([230, 150, 135]), () => { window.__x += 'C'; }); // 文
  71  |     `);
  72  |     await pollFor(ctx.page, `window.term.buffer.active.getLine(0).translateToString(true)`, 'foobar文');
  73  |     await pollFor(ctx.page, `window.__x`, 'ABC');
  74  |   });
  75  | 
  76  |   test('writeln', async () => {
  77  |     await openTerminal(ctx);
  78  |     await ctx.proxy.writeln('foo');
  79  |     await ctx.proxy.writeln('bar');
  80  |     await ctx.proxy.writeln('文');
  81  |     strictEqual(await (await ctx.proxy.buffer.active.getLine(0))!.translateToString(true), 'foo');
  82  |     strictEqual(await (await ctx.proxy.buffer.active.getLine(1))!.translateToString(true), 'bar');
  83  |     strictEqual(await (await ctx.proxy.buffer.active.getLine(2))!.translateToString(true), '文');
  84  |   });
  85  | 
  86  |   test('writeln with callback', async () => {
  87  |     await openTerminal(ctx);
  88  |     await ctx.page.evaluate(`
  89  |       window.term.writeln('foo', () => { window.__x = '1'; });
  90  |       window.term.writeln('bar', () => { window.__x += '2'; });
  91  |       window.term.writeln('文', () => { window.__x += '3'; });
  92  |     `);
  93  |     await pollFor(ctx.page, `window.term.buffer.active.getLine(0).translateToString(true)`, 'foo');
  94  |     await pollFor(ctx.page, `window.term.buffer.active.getLine(1).translateToString(true)`, 'bar');
  95  |     await pollFor(ctx.page, `window.term.buffer.active.getLine(2).translateToString(true)`, '文');
  96  |     await pollFor(ctx.page, `window.__x`, '123');
  97  |   });
  98  | 
  99  |   test('writeln - bytes (UTF8)', async () => {
  100 |     await openTerminal(ctx);
  101 |     await ctx.page.evaluate(`
  102 |       window.term.writeln(new Uint8Array([102, 111, 111]));
  103 |       window.term.writeln(new Uint8Array([98, 97, 114]));
  104 |       window.term.writeln(new Uint8Array([230, 150, 135]));
  105 |     `);
  106 |     await pollFor(ctx.page, `window.term.buffer.active.getLine(0).translateToString(true)`, 'foo');
  107 |     await pollFor(ctx.page, `window.term.buffer.active.getLine(1).translateToString(true)`, 'bar');
  108 |     await pollFor(ctx.page, `window.term.buffer.active.getLine(2).translateToString(true)`, '文');
  109 |   });
  110 | 
  111 |   test('paste', async () => {
  112 |     await openTerminal(ctx);
  113 |     const calls: string[] = [];
  114 |     ctx.proxy.onData(e => calls.push(e));
  115 |     await ctx.proxy.paste('foo');
```