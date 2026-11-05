# Instructions

- Following Playwright test failed.
- Explain why, be concise, respect Playwright best practices.
- Provide a snippet of code with the fix, if possible.

# Test info

- Name: WebglCustomGlyphs.test.ts >> WebGL custom glyphs >> pattern glyphs render after the terminal is adopted into another document
- Location: addons\addon-webgl\test\WebglCustomGlyphs.test.ts:32:7

# Error details

```
Error: page.evaluate: TypeError: undefined is not a constructor (evaluating 'new window.WebglAddon({ customGlyphs: true })')
```

# Test source

```ts
  1  | /**
  2  |  * Copyright (c) 2026 The xterm.js authors. All rights reserved.
  3  |  * @license MIT
  4  |  */
  5  | 
  6  | import test, { expect } from '@playwright/test';
  7  | import { platform } from 'os';
  8  | import { ITestContext, createTestContext, openTerminal } from '../../../test/playwright/TestUtils';
  9  | 
  10 | interface ITestRendererWithAtlasCanvas {
  11 |   readonly _charAtlas?: {
  12 |     readonly _tmpCanvas?: HTMLCanvasElement;
  13 |   };
  14 | }
  15 | 
  16 | async function writeAndWaitForRender(ctx: ITestContext, data: string): Promise<void> {
  17 |   const renderPromise = new Promise<void>(resolve => {
  18 |     const disposable = ctx.proxy.onRender(() => {
  19 |       disposable.dispose();
  20 |       resolve();
  21 |     });
  22 |   });
  23 |   await ctx.proxy.write(data);
  24 |   await renderPromise;
  25 | }
  26 | 
  27 | test.describe('WebGL custom glyphs', () => {
  28 |   if (platform() === 'linux') {
  29 |     test.skip(({ browserName }) => browserName === 'firefox' || browserName === 'webkit');
  30 |   }
  31 | 
  32 |   test('pattern glyphs render after the terminal is adopted into another document', async ({ browser }) => {
  33 |     const ctx = await createTestContext(browser);
  34 |     const errors: string[] = [];
  35 |     const onError = (error: Error): void => { errors.push(error.message); };
  36 |     ctx.page.on('pageerror', onError);
  37 |     try {
  38 |       await openTerminal(ctx);
> 39 |       await ctx.page.evaluate(`
     |                      ^ Error: page.evaluate: TypeError: undefined is not a constructor (evaluating 'new window.WebglAddon({ customGlyphs: true })')
  40 |         window.addon = new window.WebglAddon({ customGlyphs: true });
  41 |         window.term.loadAddon(window.addon);
  42 |       `);
  43 |       await ctx.page.evaluate(() => {
  44 |         const frame = document.createElement('iframe');
  45 |         document.body.appendChild(frame);
  46 |         const auxiliaryDocument = frame.contentDocument;
  47 |         if (!auxiliaryDocument?.body || !window.term.element) {
  48 |           throw new Error('Auxiliary document and terminal element must be available');
  49 |         }
  50 |         const renderer = window.term._core?._renderService?._renderer?.value as ITestRendererWithAtlasCanvas | undefined;
  51 |         const atlasCanvas = renderer?._charAtlas?._tmpCanvas;
  52 |         if (!atlasCanvas) {
  53 |           throw new Error('Texture atlas canvas must be available');
  54 |         }
  55 |         // Model the atlas state after an uncached glyph has attached the measurement canvas.
  56 |         window.term.element.appendChild(atlasCanvas);
  57 |         auxiliaryDocument.body.appendChild(window.term.element);
  58 |         if (atlasCanvas.ownerDocument !== auxiliaryDocument) {
  59 |           throw new Error('Texture atlas canvas must be adopted into the auxiliary document');
  60 |         }
  61 |         auxiliaryDocument.createElement = () => {
  62 |           throw new Error('Not allowed to create elements in the auxiliary document');
  63 |         };
  64 |       });
  65 | 
  66 |       await writeAndWaitForRender(ctx, '\u2591');
  67 |       await writeAndWaitForRender(ctx, 'X');
  68 | 
  69 |       const line = await ctx.page.evaluate(() => window.term.buffer.active.getLine(0)?.translateToString(true));
  70 |       expect(errors, `renderer must not create pattern canvases in the auxiliary document: ${errors[0] ?? ''}`).toEqual([]);
  71 |       expect(line).toBe('\u2591X');
  72 |     } finally {
  73 |       ctx.page.off('pageerror', onError);
  74 |       await ctx.page.close();
  75 |     }
  76 |   });
  77 | });
  78 | 
```