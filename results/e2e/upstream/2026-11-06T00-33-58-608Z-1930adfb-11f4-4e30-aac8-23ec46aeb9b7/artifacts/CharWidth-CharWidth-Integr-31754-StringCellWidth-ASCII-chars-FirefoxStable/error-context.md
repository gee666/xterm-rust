# Instructions

- Following Playwright test failed.
- Explain why, be concise, respect Playwright best practices.
- Provide a snippet of code with the fix, if possible.

# Test info

- Name: CharWidth.test.ts >> CharWidth Integration Tests >> getStringCellWidth >> ASCII chars
- Location: test\playwright\CharWidth.test.ts:20:9

# Error details

```
"beforeAll" hook timeout of 10000ms exceeded.
```

```
TypeError: Cannot read properties of undefined (reading 'page')
```

# Page snapshot

```yaml
- generic [ref=e5]:
  - generic:
    - textbox "Terminal input"
```

# Test source

```ts
  1  | /**
  2  |  * Copyright (c) 2017 The xterm.js authors. All rights reserved.
  3  |  * @license MIT
  4  |  */
  5  | 
  6  | import { test } from '@playwright/test';
  7  | import { createTestContext, ITestContext, openTerminal, pollFor } from './TestUtils';
  8  | 
  9  | let ctx: ITestContext;
  10 | test.beforeAll(async ({ browser }) => {
  11 |   ctx = await createTestContext(browser);
  12 |   await openTerminal(ctx);
  13 | });
> 14 | test.afterAll(async () => await ctx.page.close());
     |                                     ^ TypeError: Cannot read properties of undefined (reading 'page')
  15 | 
  16 | test.beforeEach(async () => await ctx.proxy.reset());
  17 | 
  18 | test.describe('CharWidth Integration Tests', () => {
  19 |   test.describe('getStringCellWidth', () => {
  20 |     test('ASCII chars', async () => {
  21 |       await ctx.proxy.write('This is just ASCII text.#');
  22 |       await pollFor(ctx.page, () => sumWidths(0, 1, '#'), 25);
  23 |     });
  24 | 
  25 |     test('combining chars', async () => {
  26 |       await ctx.proxy.write('e\u0301e\u0301e\u0301e\u0301e\u0301e\u0301e\u0301e\u0301e\u0301#');
  27 |       await pollFor(ctx.page, () => sumWidths(0, 1, '#'), 10);
  28 |     });
  29 | 
  30 |     test('surrogate chars', async () => {
  31 |       await ctx.proxy.write('𝄞𝄞𝄞𝄞𝄞𝄞𝄞𝄞𝄞𝄞𝄞𝄞𝄞𝄞𝄞𝄞𝄞𝄞𝄞𝄞𝄞𝄞𝄞𝄞𝄞𝄞𝄞#');
  32 |       await pollFor(ctx.page, () => sumWidths(0, 1, '#'), 28);
  33 |     });
  34 | 
  35 |     test('surrogate combining chars', async () => {
  36 |       await ctx.proxy.write('𓂀\u0301𓂀\u0301𓂀\u0301𓂀\u0301𓂀\u0301𓂀\u0301𓂀\u0301𓂀\u0301𓂀\u0301𓂀\u0301𓂀\u0301#');
  37 |       await pollFor(ctx.page, () => sumWidths(0, 1, '#'), 12);
  38 |     });
  39 | 
  40 |     test('fullwidth chars', async () => {
  41 |       await ctx.proxy.write('１２３４５６７８９０#');
  42 |       await pollFor(ctx.page, () => sumWidths(0, 1, '#'), 21);
  43 |     });
  44 | 
  45 |     test('fullwidth chars offset 1', async () => {
  46 |       await ctx.proxy.write('a１２３４５６７８９０#');
  47 |       await pollFor(ctx.page, () => sumWidths(0, 1, '#'), 22);
  48 |     });
  49 | 
  50 |     // TODO: multiline tests once #1685 is resolved
  51 |   });
  52 | });
  53 | 
  54 | async function sumWidths(start: number, end: number, sentinel: string): Promise<number> {
  55 |   await ctx.page.evaluate(`
  56 |     (function() {
  57 |       window.result = 0;
  58 |       const buffer = window.term.buffer.active;
  59 |       for (let i = ${start}; i < ${end}; i++) {
  60 |         const line = buffer.getLine(i);
  61 |         let j = 0;
  62 |         while (true) {
  63 |           const cell = line.getCell(j++);
  64 |           if (!cell) {
  65 |             break;
  66 |           }
  67 |           window.result += cell.getWidth();
  68 |           if (cell.getChars() === '${sentinel}') {
  69 |             return;
  70 |           }
  71 |         }
  72 |       }
  73 |     })();
  74 |   `);
  75 |   return await ctx.page.evaluate(`window.result`);
  76 | }
  77 | 
```