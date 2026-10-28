# Instructions

- Following Playwright test failed.
- Explain why, be concise, respect Playwright best practices.
- Provide a snippet of code with the fix, if possible.

# Test info

- Name: WebglRenderer.test.js >> WebGL Renderer Integration Tests >> selection blending >> background
- Location: test\playwright\SharedRendererTests.ts:1009:9

# Error details

```
AssertionError: pollFor max duration exceeded.
Last comparison: [64,13,13,255] (actual) !== [230,128,128,255] (expected)
Stack: Error: 
    at pollFor (P:\workshop\Windows_Worshop\oira666\xterm-rust\source_repo\test\playwright\TestUtils.ts:531:21)
    at P:\workshop\Windows_Worshop\oira666\xterm-rust\source_repo\test\playwright\SharedRendererTests.ts:1021:20
    at P:\workshop\Windows_Worshop\oira666\xterm-rust\source_repo\node_modules\playwright\lib\worker\workerProcessEntry.js:3045:9
    at P:\workshop\Windows_Worshop\oira666\xterm-rust\source_repo\node_modules\playwright\lib\worker\workerProcessEntry.js:2537:11
    at TimeoutManager.withRunnable (P:\workshop\Windows_Worshop\oira666\xterm-rust\source_repo\node_modules\playwright\lib\worker\workerProcessEntry.js:1816:14)
    at TestInfoImpl._runWithTimeout (P:\workshop\Windows_Worshop\oira666\xterm-rust\source_repo\node_modules\playwright\lib\worker\workerProcessEntry.js:2535:7)
    at P:\workshop\Windows_Worshop\oira666\xterm-rust\source_repo\node_modules\playwright\lib\worker\workerProcessEntry.js:3043:7
    at WorkerMain._runTest (P:\workshop\Windows_Worshop\oira666\xterm-rust\source_repo\node_modules\playwright\lib\worker\workerProcessEntry.js:3016:5)
    at WorkerMain.runTestGroup (P:\workshop\Windows_Worshop\oira666\xterm-rust\source_repo\node_modules\playwright\lib\worker\workerProcessEntry.js:2911:9)
    at process.<anonymous> (P:\workshop\Windows_Worshop\oira666\xterm-rust\source_repo\node_modules\playwright\lib\common\index.js:1955:25)
+ actual - expected

  [
+   64,
+   13,
+   13,
-   230,
-   128,
-   128,
    255
  ]

```

# Page snapshot

```yaml
- generic [ref=e6]:
  - textbox "Terminal input" [active] [ref=e7]
  - text: WWWWWWWWWWWWWWWWWWWWWWWWWWWWWWWW
```

# Test source

```ts
  431 | 
  432 |   public get isDisposed(): Promise<boolean> { return this.evaluate(([core]) => (core as any)._isDisposed); }
  433 |   public get renderDimensions(): Promise<IRenderDimensionsInternal> { return this.evaluate(([core]) => ((core as any)._renderService as IRenderService).dimensions); }
  434 | 
  435 |   public async triggerBinaryEvent(data: string): Promise<void> {
  436 |     return this._page.evaluate(([core, data]) => core.coreService.triggerBinaryEvent(data), [await this._getCoreHandle(), data] as const);
  437 |   }
  438 | 
  439 | 
  440 |   private async _getCoreHandle(): Promise<JSHandle<ICoreTerminal>> {
  441 |     return this._proxy.evaluateHandle(([term]) => (term as any)._core as ICoreTerminal);
  442 |   }
  443 | 
  444 |   public async evaluate<T>(pageFunction: PageFunction<ICoreTerminal[], T>): Promise<T> {
  445 |     return this._page.evaluate(pageFunction, [await this._getCoreHandle()]);
  446 |   }
  447 | }
  448 | 
  449 | export async function openTerminal(
  450 |   ctx: ITestContext,
  451 |   options: ITerminalOptions | ITerminalInitOnlyOptions = {},
  452 |   testOptions: { useShadowDom?: boolean, loadUnicodeGraphemesAddon?: boolean } = {}
  453 | ): Promise<void> {
  454 |   testOptions.useShadowDom ??= false;
  455 |   testOptions.loadUnicodeGraphemesAddon ??= true;
  456 | 
  457 |   await ctx.page.evaluate(`
  458 |   if ('term' in window) {
  459 |     try {
  460 |       window.term.dispose();
  461 |     } catch {}
  462 |   }
  463 |   `);
  464 |   // HACK: Tests may have side effects that could cause the terminal not to be removed. This
  465 |   //       assertion catches this case early.
  466 |   strictEqual(await ctx.page.evaluate(`document.querySelector('#terminal-container').children.length`), 0, 'there must be no terminals on the page');
  467 | 
  468 |   let script = `
  469 |     window.term = new window.Terminal(${JSON.stringify({ allowProposedApi: true, ...options })});
  470 |     let element = document.querySelector('#terminal-container');
  471 | 
  472 |     // Remove shadow root if it exists
  473 |     const newElement = element.cloneNode(false);
  474 |     element.replaceWith(newElement);
  475 |     element = newElement
  476 | `;
  477 | 
  478 | 
  479 |   if (testOptions.useShadowDom) {
  480 |     script += `
  481 |     const shadowRoot = element.attachShadow({ mode: "open" });
  482 | 
  483 |     // Copy parent styles to shadow DOM
  484 |     const styles = Array.from(document.querySelectorAll('link[rel="stylesheet"]'));
  485 |     styles.forEach((styleEl) => {
  486 |       const clone = document.createElement('link');
  487 |       clone.rel = 'stylesheet';
  488 |       clone.href = styleEl.href;
  489 |       shadowRoot.appendChild(clone);
  490 |     });
  491 | 
  492 |     // Create new element inside the shadow DOM
  493 |     element = document.createElement('div');
  494 |     element.style.width = '100%';
  495 |     element.style.height = '100%';
  496 |     shadowRoot.appendChild(element);
  497 |     `;
  498 |   }
  499 | 
  500 |   script += `
  501 |     window.term.open(element);
  502 |   `;
  503 | 
  504 |   await ctx.page.evaluate(script);
  505 | 
  506 |   // HACK: This is a soft layer breaker that's temporarily included until unicode graphemes have
  507 |   // more complete integration tests. See https://github.com/xtermjs/xterm.js/pull/4519#discussion_r1285234453
  508 |   if (testOptions.loadUnicodeGraphemesAddon) {
  509 |     await ctx.page.evaluate(`
  510 |       window.unicode = new UnicodeGraphemesAddon();
  511 |       window.term.loadAddon(window.unicode);
  512 |       window.term.unicode.activeVersion = '15-graphemes';
  513 |     `);
  514 |   }
  515 |   await ctx.page.waitForSelector('.xterm-rows');
  516 |   ctx.termHandle = await ctx.page.evaluateHandle('window.term');
  517 |   await ctx.proxy.initTerm();
  518 | }
  519 | 
  520 | 
  521 | export type MaybeAsync<T> = Promise<T> | T;
  522 | 
  523 | interface IPollForOptions<T> {
  524 |   equalityFn?: (a: T, b: T) => boolean;
  525 |   maxDuration?: number;
  526 |   stack?: string;
  527 | }
  528 | 
  529 | export async function pollFor<T>(page: playwright.Page, evalOrFn: string | (() => MaybeAsync<T>), val: T, preFn?: () => Promise<void>, options?: IPollForOptions<T>): Promise<void> {
  530 |   options ??= {};
> 531 |   options.stack ??= new Error().stack;
      |                     ^ AssertionError: pollFor max duration exceeded.
  532 |   if (preFn) {
  533 |     await preFn();
  534 |   }
  535 |   const result = typeof evalOrFn === 'string' ? await page.evaluate(evalOrFn) : await evalOrFn();
  536 | 
  537 |   if (process.env.DEBUG) {
  538 |     console.log('pollFor\n  actual: ', JSON.stringify(result), '  expected: ', JSON.stringify(val));
  539 |   }
  540 | 
  541 |   let equalityCheck: boolean;
  542 |   if (options.equalityFn) {
  543 |     equalityCheck = options.equalityFn(result as T, val);
  544 |   } else {
  545 |     equalityCheck = true;
  546 |     try {
  547 |       deepStrictEqual(result, val);
  548 |     } catch {
  549 |       equalityCheck = false;
  550 |     }
  551 |   }
  552 | 
  553 |   if (!equalityCheck) {
  554 |     options.maxDuration ??= 2000;
  555 |     if (options.maxDuration <= 0) {
  556 |       deepStrictEqual(result, val, ([
  557 |         `pollFor max duration exceeded.`,
  558 |         (`Last comparison: ` +
  559 |           `${typeof result === 'object' ? JSON.stringify(result) : result} (actual) !== ` +
  560 |           `${typeof val === 'object' ? JSON.stringify(val) : val} (expected)`),
  561 |         `Stack: ${options.stack}`
  562 |       ].join('\n')));
  563 |     }
  564 |     return new Promise<void>(r => {
  565 |       setTimeout(() => r(pollFor(page, evalOrFn, val, preFn, {
  566 |         ...options,
  567 |         maxDuration: options!.maxDuration! - 10,
  568 |         stack: options!.stack
  569 |       })), 10);
  570 |     });
  571 |   }
  572 | }
  573 | 
  574 | export async function pollForApproximate<T>(page: playwright.Page, marginOfError: number, evalOrFn: string | (() => MaybeAsync<T>), val: T, preFn?: () => Promise<void>, maxDuration?: number, stack?: string): Promise<void> {
  575 |   await pollFor(page, evalOrFn, val, preFn, {
  576 |     maxDuration,
  577 |     stack,
  578 |     equalityFn: (a, b) => {
  579 |       if (a === b) {
  580 |         return true;
  581 |       }
  582 |       if (Array.isArray(a) && Array.isArray(b) && a.length === b.length) {
  583 |         let success = true;
  584 |         for (let i = 0 ; i < a.length; i++) {
  585 |           if (Math.abs(a[i] - b[i]) > marginOfError) {
  586 |             success = false;
  587 |             break;
  588 |           }
  589 |         }
  590 |         if (success) {
  591 |           return true;
  592 |         }
  593 |       }
  594 |       return false;
  595 |     }
  596 |   });
  597 | }
  598 | 
  599 | export async function writeSync(page: playwright.Page, data: string): Promise<void> {
  600 |   await page.evaluate(`
  601 |     window.ready = false;
  602 |     window.term.write('${data}', () => window.ready = true);
  603 |   `);
  604 |   await pollFor(page, 'window.ready', true);
  605 | }
  606 | 
  607 | export async function timeout(ms: number): Promise<void> {
  608 |   return new Promise<void>(r => setTimeout(r, ms));
  609 | }
  610 | 
  611 | export function getBrowserType(): playwright.BrowserType<playwright.WebKitBrowser> | playwright.BrowserType<playwright.ChromiumBrowser> | playwright.BrowserType<playwright.FirefoxBrowser> {
  612 |   // Default to chromium
  613 |   let browserType: playwright.BrowserType<playwright.WebKitBrowser> | playwright.BrowserType<playwright.ChromiumBrowser> | playwright.BrowserType<playwright.FirefoxBrowser> = playwright['chromium'];
  614 | 
  615 |   const index = process.argv.indexOf('--browser');
  616 |   if (index !== -1 && process.argv.length > index + 1 && typeof process.argv[index + 1] === 'string') {
  617 |     const string = process.argv[index + 1];
  618 |     if (string === 'firefox' || string === 'webkit') {
  619 |       browserType = playwright[string];
  620 |     }
  621 |   }
  622 | 
  623 |   return browserType;
  624 | }
  625 | 
  626 | export function launchBrowser(opts?: playwright.LaunchOptions): Promise<playwright.Browser> {
  627 |   const browserType = getBrowserType();
  628 |   const options: playwright.LaunchOptions = {
  629 |     ...opts,
  630 |     headless: process.argv.includes('--headless')
  631 |   };
```