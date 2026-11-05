# Instructions

- Following Playwright test failed.
- Explain why, be concise, respect Playwright best practices.
- Provide a snippet of code with the fix, if possible.

# Test info

- Name: InputHandler.test.ts >> InputHandler Integration Tests >> CSI >> CSI ? Pm l - DECRST: DEC Private Mode Reset >> Ps = 2 0 0 1 - Disable readline mouse button-1, xterm.
- Location: test\playwright\InputHandler.test.ts:1024:11

# Error details

```
Error: page.evaluate: TypeError: undefined is not an object (evaluating 'core.mouseStateService.activeProtocol')
```

# Page snapshot

```yaml
- generic [ref=e6]:
  - textbox "Terminal input"
  - text: WWWWWWWWWWWWWWWWWWWWWWWWWWWWWWWW
```

# Test source

```ts
  345 |     return undefined;
  346 |   }
  347 | 
  348 |   public async evaluate<T>(pageFunction: PageFunction<IBuffer[], T>): Promise<T> {
  349 |     return this._page.evaluate(pageFunction, [await this._handle]);
  350 |   }
  351 | }
  352 | 
  353 | // TODO: Adopt PlaywrightApiProxy
  354 | class TerminalBufferLine {
  355 |   constructor(
  356 |     private readonly _page: Page,
  357 |     private readonly _handle: JSHandle<IBufferLine>
  358 |   ) {
  359 |   }
  360 | 
  361 |   public get length(): Promise<number> { return this.evaluate(([bufferLine]) => bufferLine.length); }
  362 |   public get isWrapped(): Promise<boolean> { return this.evaluate(([bufferLine]) => bufferLine.isWrapped); }
  363 | 
  364 |   public translateToString(trimRight?: boolean, startColumn?: number, endColumn?: number): Promise<string> {
  365 |     return this._page.evaluate(([bufferLine, trimRight, startColumn, endColumn]) => {
  366 |       return bufferLine.translateToString(trimRight, startColumn, endColumn);
  367 |     }, [this._handle, trimRight, startColumn, endColumn] as const);
  368 |   }
  369 | 
  370 |   public async getCell(x: number): Promise<TerminalBufferCell | undefined> {
  371 |     const cellHandle = await this._page.evaluateHandle(([bufferLine, x]) => bufferLine.getCell(x), [this._handle, x] as const);
  372 |     const value = await cellHandle.jsonValue();
  373 |     if (value) {
  374 |       return new TerminalBufferCell(this._page, cellHandle as JSHandle<IBufferCell>);
  375 |     }
  376 |     return undefined;
  377 |   }
  378 | 
  379 |   public async evaluate<T>(pageFunction: PageFunction<IBufferLine[], T>): Promise<T> {
  380 |     return this._page.evaluate(pageFunction, [this._handle]);
  381 |   }
  382 | }
  383 | 
  384 | // TODO: Adopt PlaywrightApiProxy
  385 | class TerminalBufferCell {
  386 |   constructor(
  387 |     private readonly _page: Page,
  388 |     private readonly _handle: JSHandle<IBufferCell>
  389 |   ) {
  390 |   }
  391 | 
  392 |   public getWidth(): Promise<number> { return this.evaluate(([cell]) => cell.getWidth()); }
  393 |   public getChars(): Promise<string> { return this.evaluate(([cell]) => cell.getChars()); }
  394 |   public getCode(): Promise<number> { return this.evaluate(([cell]) => cell.getCode()); }
  395 | 
  396 |   public getFgColorMode(): Promise<number> { return this.evaluate(([cell]) => cell.getFgColorMode()); }
  397 |   public getBgColorMode(): Promise<number> { return this.evaluate(([cell]) => cell.getBgColorMode()); }
  398 |   public getFgColor(): Promise<number> { return this.evaluate(([cell]) => cell.getFgColor()); }
  399 |   public getBgColor(): Promise<number> { return this.evaluate(([cell]) => cell.getBgColor()); }
  400 | 
  401 |   public isBold(): Promise<number> { return this.evaluate(([cell]) => cell.isBold()); }
  402 |   public isItalic(): Promise<number> { return this.evaluate(([cell]) => cell.isItalic()); }
  403 |   public isDim(): Promise<number> { return this.evaluate(([cell]) => cell.isDim()); }
  404 |   public isUnderline(): Promise<number> { return this.evaluate(([cell]) => cell.isUnderline()); }
  405 |   public isBlink(): Promise<number> { return this.evaluate(([cell]) => cell.isBlink()); }
  406 |   public isInverse(): Promise<number> { return this.evaluate(([cell]) => cell.isInverse()); }
  407 |   public isInvisible(): Promise<number> { return this.evaluate(([cell]) => cell.isInvisible()); }
  408 |   public isStrikethrough(): Promise<number> { return this.evaluate(([cell]) => cell.isStrikethrough()); }
  409 |   public isOverline(): Promise<number> { return this.evaluate(([cell]) => cell.isOverline()); }
  410 | 
  411 |   public isFgRGB(): Promise<boolean> { return this.evaluate(([cell]) => cell.isFgRGB()); }
  412 |   public isBgRGB(): Promise<boolean> { return this.evaluate(([cell]) => cell.isBgRGB()); }
  413 |   public isFgPalette(): Promise<boolean> { return this.evaluate(([cell]) => cell.isFgPalette()); }
  414 |   public isBgPalette(): Promise<boolean> { return this.evaluate(([cell]) => cell.isBgPalette()); }
  415 |   public isFgDefault(): Promise<boolean> { return this.evaluate(([cell]) => cell.isFgDefault()); }
  416 |   public isBgDefault(): Promise<boolean> { return this.evaluate(([cell]) => cell.isBgDefault()); }
  417 | 
  418 |   public isAttributeDefault(): Promise<boolean> { return this.evaluate(([cell]) => cell.isAttributeDefault()); }
  419 | 
  420 |   public async evaluate<T>(pageFunction: PageFunction<IBufferCell[], T>): Promise<T> {
  421 |     return this._page.evaluate(pageFunction, [this._handle]);
  422 |   }
  423 | }
  424 | 
  425 | class TerminalCoreProxy {
  426 |   constructor(
  427 |     private readonly _page: Page,
  428 |     private readonly _proxy: TerminalProxy
  429 |   ) {
  430 |   }
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
> 445 |     return this._page.evaluate(pageFunction, [await this._getCoreHandle()]);
      |                       ^ Error: page.evaluate: TypeError: undefined is not an object (evaluating 'core.mouseStateService.activeProtocol')
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
  531 |   options.stack ??= new Error().stack;
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
```