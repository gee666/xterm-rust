# Instructions

- Following Playwright test failed.
- Explain why, be concise, respect Playwright best practices.
- Provide a snippet of code with the fix, if possible.

# Test info

- Name: Unicode11Addon.test.js >> Unicode11Addon >> wcwidth V11 emoji test
- Location: addons\addon-unicode11\test\Unicode11Addon.test.ts:28:3

# Error details

```
Error: page.goto: Protocol error (Playwright.navigate): Cannot navigate to invalid URL
Call log:
  - navigating to "/test", waiting until "load"

```

```
TypeError: Cannot read properties of undefined (reading 'page')
```

# Test source

```ts
  1  | /**
  2  |  * Copyright (c) 2019 The xterm.js authors. All rights reserved.
  3  |  * @license MIT
  4  |  */
  5  | 
  6  | import test from '@playwright/test';
  7  | import { deepStrictEqual } from 'assert';
  8  | import { ITestContext, createTestContext, openTerminal } from '../../../test/playwright/TestUtils';
  9  | 
  10 | let ctx: ITestContext;
  11 | test.beforeAll(async ({ browser }) => {
  12 |   ctx = await createTestContext(browser);
  13 |   await openTerminal(ctx);
  14 | });
> 15 | test.afterAll(async () => await ctx.page.close());
     |                                     ^ TypeError: Cannot read properties of undefined (reading 'page')
  16 | 
  17 | test.describe('Unicode11Addon', () => {
  18 | 
  19 |   test.beforeEach(async () => {
  20 |     await ctx.page.evaluate(`
  21 |       window.term.reset()
  22 |       window.unicode11?.dispose();
  23 |       window.unicode11 = new Unicode11Addon();
  24 |       window.term.loadAddon(window.unicode11);
  25 |     `);
  26 |   });
  27 | 
  28 |   test('wcwidth V11 emoji test', async () => {
  29 |     // should have loaded '11'
  30 |     deepStrictEqual((await ctx.page.evaluate(`window.term.unicode.versions`) as string[]).includes('11'), true);
  31 |     // switch should not throw
  32 |     await ctx.page.evaluate(`window.term.unicode.activeVersion = '11';`);
  33 |     deepStrictEqual(await ctx.page.evaluate(`window.term.unicode.activeVersion`), '11');
  34 |     // v6: 10, V11: 20
  35 |     deepStrictEqual(await ctx.page.evaluate(`window.term._core.unicodeService.getStringCellWidth('🤣🤣🤣🤣🤣🤣🤣🤣🤣🤣')`), 20);
  36 |   });
  37 | });
  38 | 
```