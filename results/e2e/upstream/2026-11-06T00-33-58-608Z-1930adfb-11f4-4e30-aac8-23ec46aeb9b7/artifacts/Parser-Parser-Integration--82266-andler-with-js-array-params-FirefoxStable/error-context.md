# Instructions

- Following Playwright test failed.
- Explain why, be concise, respect Playwright best practices.
- Provide a snippet of code with the fix, if possible.

# Test info

- Name: Parser.test.ts >> Parser Integration Tests >> registerCsiHandler >> should call custom CSI handler with js array params
- Location: test\playwright\Parser.test.ts:42:9

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
  6   | import { deepStrictEqual, ok, strictEqual } from 'assert';
  7   | import type { IDisposable } from '@xterm/xterm';
  8   | import { createTestContext, ITestContext, openTerminal, pollFor } from './TestUtils';
  9   | 
  10  | let ctx: ITestContext;
  11  | test.beforeAll(async ({ browser }) => {
  12  |   ctx = await createTestContext(browser);
  13  |   await openTerminal(ctx);
  14  | });
> 15  | test.afterAll(async () => await ctx.page.close());
      |                                     ^ TypeError: Cannot read properties of undefined (reading 'page')
  16  | 
  17  | declare global {
  18  |   interface Window { // eslint-disable-line @typescript-eslint/naming-convention
  19  |     customCsiHandlerParams?: (number | number[])[][];
  20  |     customCsiHandlerCallStack?: string[];
  21  |     customDcsHandlerCallStack?: [string, (number | number[])[], string][];
  22  |     customEscHandlerCallStack?: string[];
  23  |     customOscHandlerCallStack?: string[][];
  24  |     customApcHandlerCallStack?: string[][];
  25  |     disposable?: IDisposable;
  26  |     disposables?: IDisposable[];
  27  |   }
  28  | }
  29  | 
  30  | test.describe('Parser Integration Tests', () => {
  31  |   test.beforeEach(async () => await ctx.proxy.reset());
  32  |   test.afterEach(async () => {
  33  |     await ctx.page.evaluate(() => {
  34  |       window.disposable?.dispose();
  35  |       window.disposable = undefined;
  36  |       window.disposables?.forEach(e => e.dispose());
  37  |       window.disposables = undefined;
  38  |     });
  39  |   });
  40  | 
  41  |   test.describe('registerCsiHandler', () => {
  42  |     test('should call custom CSI handler with js array params', async () => {
  43  |       await ctx.proxy.evaluate(([term]) => {
  44  |         window.customCsiHandlerParams = [];
  45  |         window.disposable = term.parser.registerCsiHandler({final: 'm'}, (params) => {
  46  |           window.customCsiHandlerParams!.push(params);
  47  |           return false;
  48  |         });
  49  |       });
  50  |       await ctx.proxy.write('\x1b[38;5;123mparams\x1b[38:2::50:100:150msubparams');
  51  |       deepStrictEqual(await ctx.page.evaluate(() => window.customCsiHandlerParams), [
  52  |         [38, 5, 123],
  53  |         [38, [2, -1, 50, 100, 150]]
  54  |       ]);
  55  |     });
  56  |     test('async', async () => {
  57  |       await ctx.proxy.evaluate(([term]) => {
  58  |         window.customCsiHandlerCallStack = [];
  59  |         window.customCsiHandlerParams = [];
  60  |         window.disposables = [
  61  |           term.parser.registerCsiHandler({intermediates:'+', final: 'Z'}, params => {
  62  |             window.customCsiHandlerCallStack!.push('A');
  63  |             window.customCsiHandlerParams!.push(params);
  64  |             return false;
  65  |           }),
  66  |           term.parser.registerCsiHandler({intermediates:'+', final: 'Z'}, params => {
  67  |             return new Promise(res => setTimeout(res, 50)).then(() => {
  68  |               window.customCsiHandlerCallStack!.push('B');
  69  |               window.customCsiHandlerParams!.push(params);
  70  |               return false;
  71  |             });
  72  |           }),
  73  |           term.parser.registerCsiHandler({intermediates:'+', final: 'Z'}, params => {
  74  |             window.customCsiHandlerCallStack!.push('C');
  75  |             window.customCsiHandlerParams!.push(params);
  76  |             return false;
  77  |           })
  78  |         ];
  79  |       });
  80  |       await ctx.proxy.write('\x1b[1;2+Z');
  81  |       deepStrictEqual(await ctx.page.evaluate(() => window.customCsiHandlerCallStack), [
  82  |         'C',
  83  |         'B',
  84  |         'A'
  85  |       ]);
  86  |       deepStrictEqual(await ctx.page.evaluate(() => window.customCsiHandlerParams), [
  87  |         [1, 2],
  88  |         [1, 2],
  89  |         [1, 2]
  90  |       ]);
  91  |     });
  92  |   });
  93  |   test.describe('registerDcsHandler', () => {
  94  |     test('should respects return value', async () => {
  95  |       await ctx.proxy.evaluate(([term]) => {
  96  |         window.customDcsHandlerCallStack = [];
  97  |         window.disposables = [
  98  |           term.parser.registerDcsHandler({intermediates:'+', final: 'p'}, (data, params) => {
  99  |             window.customDcsHandlerCallStack!.push(['A', params, data]);
  100 |             return false;
  101 |           }),
  102 |           term.parser.registerDcsHandler({intermediates:'+', final: 'p'}, (data, params) => {
  103 |             window.customDcsHandlerCallStack!.push(['B', params, data]);
  104 |             return true;
  105 |           }),
  106 |           term.parser.registerDcsHandler({intermediates:'+', final: 'p'}, (data, params) => {
  107 |             window.customDcsHandlerCallStack!.push(['C', params, data]);
  108 |             return false;
  109 |           })
  110 |         ];
  111 |       });
  112 |       await ctx.proxy.write('\x1bP1;2+psome data\x1b\\');
  113 |       deepStrictEqual(await ctx.page.evaluate(() => window.customDcsHandlerCallStack), [
  114 |         ['C', [1, 2], 'some data'],
  115 |         ['B', [1, 2], 'some data']
```