# Instructions

- Following Playwright test failed.
- Explain why, be concise, respect Playwright best practices.
- Provide a snippet of code with the fix, if possible.

# Test info

- Name: Renderer.test.ts >> DOM Renderer Integration Tests >> colors >> foreground 0-15
- Location: test\playwright\SharedRendererTests.ts:30:9

# Error details

```
"beforeAll" hook timeout of 10000ms exceeded.
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
  2  |  * Copyright (c) 2019 The xterm.js authors. All rights reserved.
  3  |  * @license MIT
  4  |  */
  5  | 
  6  | import { test } from '@playwright/test';
  7  | import { ITestContext, createTestContext, openTerminal } from './TestUtils';
  8  | import { ISharedRendererTestContext, injectSharedRendererTestsStandalone, injectSharedRendererTests } from './SharedRendererTests';
  9  | 
  10 | let ctx: ITestContext;
  11 | const ctxWrapper: ISharedRendererTestContext = {
  12 |   value: undefined,
  13 |   skipDomExceptions: true
  14 | } as any;
> 15 | test.beforeAll(async ({ browser }) => {
     |      ^ "beforeAll" hook timeout of 10000ms exceeded.
  16 |   ctx = await createTestContext(browser);
  17 |   ctxWrapper.value = ctx;
  18 |   await openTerminal(ctx);
  19 | });
  20 | test.afterAll(async () => await ctx.page.close());
  21 | 
  22 | test.describe('DOM Renderer Integration Tests', () => {
  23 |   // HACK: Skip on WebKit, not clear why it's failing
  24 |   test.skip(({ browserName }) => browserName === 'webkit', 'Skipped on WebKit');
  25 | 
  26 |   injectSharedRendererTests(ctxWrapper);
  27 |   injectSharedRendererTestsStandalone(ctxWrapper, () => {});
  28 | });
  29 | 
```