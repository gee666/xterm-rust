# Instructions

- Following Playwright test failed.
- Explain why, be concise, respect Playwright best practices.
- Provide a snippet of code with the fix, if possible.

# Test info

- Name: ImageAddon.test.ts >> ImageAddon >> test for private accessors
- Location: addons\addon-image\test\ImageAddon.test.ts:101:7

# Error details

```
"beforeAll" hook timeout of 10000ms exceeded.
```

```
Error: page.evaluateHandle: Target page, context or browser has been closed
```

# Page snapshot

```yaml
- generic [ref=e5]:
  - generic:
    - textbox "Terminal input"
```

# Test source

```ts
  204 | 
  205 |   // #region Events
  206 |   private _onBell = new EventEmitter<void>();
  207 |   public get onBell(): IEvent<void> { return this._onBell.event; }
  208 |   private _onBinary = new EventEmitter<string>();
  209 |   public get onBinary(): IEvent<string> { return this._onBinary.event; }
  210 |   private _onCursorMove = new EventEmitter<void>();
  211 |   public get onCursorMove(): IEvent<void> { return this._onCursorMove.event; }
  212 |   private _onData = new EventEmitter<string>();
  213 |   public get onData(): IEvent<string> { return this._onData.event; }
  214 |   private _onKey = new EventEmitter<{ key: string, domEvent: KeyboardEvent }>();
  215 |   public get onKey(): IEvent<{ key: string, domEvent: KeyboardEvent }> { return this._onKey.event; }
  216 |   private _onLineFeed = new EventEmitter<void>();
  217 |   public get onLineFeed(): IEvent<void> { return this._onLineFeed.event; }
  218 |   private _onRender = new EventEmitter<{ start: number, end: number }>();
  219 |   public get onRender(): IEvent<{ start: number, end: number }> { return this._onRender.event; }
  220 |   private _onResize = new EventEmitter<{ cols: number, rows: number }>();
  221 |   public get onResize(): IEvent<{ cols: number, rows: number }> { return this._onResize.event; }
  222 |   private _onScroll = new EventEmitter<number>();
  223 |   public get onScroll(): IEvent<number> { return this._onScroll.event; }
  224 |   private _onDimensionsChange = new EventEmitter<IRenderDimensions>();
  225 |   public get onDimensionsChange(): IEvent<IRenderDimensions> { return this._onDimensionsChange.event; }
  226 |   private _onSelectionChange = new EventEmitter<void>();
  227 |   public get onSelectionChange(): IEvent<void> { return this._onSelectionChange.event; }
  228 |   private _onTitleChange = new EventEmitter<string>();
  229 |   public get onTitleChange(): IEvent<string> { return this._onTitleChange.event; }
  230 |   private _onWriteParsed = new EventEmitter<void>();
  231 |   public get onWriteParsed(): IEvent<void> { return this._onWriteParsed.event; }
  232 |   // #endregion
  233 | 
  234 |   // #region Simple properties
  235 |   public get cols(): Promise<number> { return this.evaluate(([term]) => term.cols); }
  236 |   public get rows(): Promise<number> { return this.evaluate(([term]) => term.rows); }
  237 |   public get modes(): Promise<IModes> { return this.evaluate(([term]) => term.modes); }
  238 |   public get dimensions(): Promise<IRenderDimensions | undefined> { return this.evaluate(([term]) => term.dimensions); }
  239 |   // #endregion
  240 | 
  241 |   // #region Complex properties
  242 |   public get buffer(): TerminalBufferNamespaceProxy { return new TerminalBufferNamespaceProxy(this._page, this); }
  243 |   /**
  244 |    * Exposes somewhat unsafe access to internals for testing things difficult to do with the regular
  245 |    * API
  246 |    */
  247 |   public get core(): TerminalCoreProxy { return new TerminalCoreProxy(this._page, this); }
  248 |   // #endregion
  249 | 
  250 |   // #region Proxied methods
  251 |   public async dispose(): Promise<void> { return this.evaluate(([term]) => term.dispose()); }
  252 |   public async reset(): Promise<void> { return this.evaluate(([term]) => term.reset()); }
  253 |   public async clear(): Promise<void> { return this.evaluate(([term]) => term.clear()); }
  254 |   public async focus(): Promise<void> { return this.evaluate(([term]) => term.focus()); }
  255 |   public async blur(): Promise<void> { return this.evaluate(([term]) => term.blur()); }
  256 |   public async hasSelection(): Promise<boolean> { return this.evaluate(([term]) => term.hasSelection()); }
  257 |   public async getSelection(): Promise<string> { return this.evaluate(([term]) => term.getSelection()); }
  258 |   public async getSelectionPosition(): Promise<IBufferRange | undefined> { return this.evaluate(([term]) => term.getSelectionPosition()); }
  259 |   public async selectAll(): Promise<void> { return this.evaluate(([term]) => term.selectAll()); }
  260 |   public async selectLines(start: number, end: number): Promise<void> { return this._page.evaluate(([term, start, end]) => term.selectLines(start, end), [await this.getHandle(), start, end] as const); }
  261 |   public async clearSelection(): Promise<void> { return this.evaluate(([term]) => term.clearSelection()); }
  262 |   public async select(column: number, row: number, length: number): Promise<void> { return this._page.evaluate(([term, column, row, length]) => term.select(column, row, length), [await this.getHandle(), column, row, length] as const); }
  263 |   public async paste(data: string): Promise<void> { return this._page.evaluate(([term, data]) => term.paste(data), [await this.getHandle(), data] as const); }
  264 |   public async refresh(start: number, end: number): Promise<void> { return this._page.evaluate(([term, start, end]) => term.refresh(start, end), [await this.getHandle(), start, end] as const); }
  265 |   public async getOption<T extends keyof ITerminalOptions>(key: T): Promise<ITerminalOptions[T]> { return this._page.evaluate(([term, key]) => term.options[key as T], [await this.getHandle(), key] as const); }
  266 |   public async setOption<T extends keyof ITerminalOptions>(key: T, value: ITerminalOptions[T]): Promise<any> { return this._page.evaluate(([term, key, value]) => term.options[key as T] = (value as ITerminalOptions[T]), [await this.getHandle(), key, value] as const); }
  267 |   public async setOptions(value: Partial<ITerminalOptions>): Promise<any> {
  268 |     return this._page.evaluate(([term, value]) => {
  269 |       term.options = value;
  270 |     }, [await this.getHandle(), value] as const);
  271 |   }
  272 |   public async scrollToTop(): Promise<void> { return this.evaluate(([term]) => term.scrollToTop()); }
  273 |   public async scrollToBottom(): Promise<void> { return this.evaluate(([term]) => term.scrollToBottom()); }
  274 |   public async scrollPages(pageCount: number): Promise<void> { return this._page.evaluate(([term, pageCount]) => term.scrollPages(pageCount), [await this.getHandle(), pageCount] as const); }
  275 |   public async scrollToLine(line: number): Promise<void> { return this._page.evaluate(([term, line]) => term.scrollToLine(line), [await this.getHandle(), line] as const); }
  276 |   public async scrollLines(amount: number): Promise<void> { return this._page.evaluate(([term, amount]) => term.scrollLines(amount), [await this.getHandle(), amount] as const); }
  277 |   public async write(data: string | Uint8Array): Promise<void> {
  278 |     return this._page.evaluate(([term, data]) => {
  279 |       return new Promise(r => term.write(typeof data === 'string' ? data : new Uint8Array(data), r));
  280 |     }, [await this.getHandle(), typeof data === 'string' ? data : Array.from(data)] as const);
  281 |   }
  282 |   public async writeln(data: string | Uint8Array): Promise<void> {
  283 |     return this._page.evaluate(([term, data]) => {
  284 |       return new Promise(r => term.writeln(typeof data === 'string' ? data : new Uint8Array(data), r));
  285 |     }, [await this.getHandle(), typeof data === 'string' ? data : Array.from(data)] as const);
  286 |   }
  287 |   public async input(data: string, wasUserInput: boolean = true): Promise<void> { return this.evaluate(([term]) => term.input(data, wasUserInput)); }
  288 |   public async resize(cols: number, rows: number): Promise<void> { return this._page.evaluate(([term, cols, rows]) => term.resize(cols, rows), [await this.getHandle(), cols, rows] as const); }
  289 |   public async registerMarker(y?: number | undefined): Promise<IMarker> { return this._page.evaluate(([term, y]) => term.registerMarker(y), [await this.getHandle(), y] as const); }
  290 |   public async registerDecoration(decorationOptions: IDecorationOptions): Promise<IDecoration | undefined> { return this._page.evaluate(([term, decorationOptions]) => term.registerDecoration(decorationOptions), [await this.getHandle(), decorationOptions] as const); }
  291 |   public async clearTextureAtlas(): Promise<void> { return this.evaluate(([term]) => term.clearTextureAtlas()); }
  292 |   // #endregion
  293 | 
  294 |   public async evaluate<T>(pageFunction: PageFunction<Terminal[], T>): Promise<T> {
  295 |     return this._page.evaluate(pageFunction, [await this.getHandle()]);
  296 |   }
  297 | 
  298 |   public async evaluateHandle<T>(pageFunction: PageFunction<Terminal[], T>): Promise<JSHandle<T>> {
  299 |     return this._page.evaluateHandle(pageFunction, [await this.getHandle()]);
  300 |   }
  301 | 
  302 |   public async getHandle(): Promise<JSHandle<Terminal>> {
  303 |     // This is async because it must be evaluated each time it is called since term may have changed
> 304 |     return this._page.evaluateHandle('window.term');
      |                       ^ Error: page.evaluateHandle: Target page, context or browser has been closed
  305 |   }
  306 | }
  307 | 
  308 | class TerminalBufferNamespaceProxy implements PlaywrightApiProxy<IBufferNamespace, never, never, 'active' | 'normal' | 'alternate'> {
  309 |   private _onBufferChange = new EventEmitter<IBuffer>();
  310 |   public readonly onBufferChange = this._onBufferChange.event;
  311 | 
  312 |   constructor(
  313 |     private readonly _page: Page,
  314 |     private readonly _proxy: TerminalProxy
  315 |   ) {
  316 | 
  317 |   }
  318 | 
  319 |   public get active(): TerminalBufferProxy { return new TerminalBufferProxy(this._page, this._proxy, this._proxy.evaluateHandle(([term]) => term.buffer.active)); }
  320 |   public get normal(): TerminalBufferProxy { return new TerminalBufferProxy(this._page, this._proxy, this._proxy.evaluateHandle(([term]) => term.buffer.normal)); }
  321 |   public get alternate(): TerminalBufferProxy { return new TerminalBufferProxy(this._page, this._proxy, this._proxy.evaluateHandle(([term]) => term.buffer.alternate)); }
  322 | }
  323 | 
  324 | // TODO: Adopt PlaywrightApiProxy
  325 | class TerminalBufferProxy /* implements EnsureAsyncProperties<IBuffer>*/ {
  326 |   constructor(
  327 |     private readonly _page: Page,
  328 |     private readonly _proxy: TerminalProxy,
  329 |     private readonly _handle: Promise<JSHandle<IBuffer>>
  330 |   ) {
  331 |   }
  332 | 
  333 |   public get type(): Promise<'normal' | 'alternate'> { return this.evaluate(([buffer]) => buffer.type); }
  334 |   public get cursorY(): Promise<number> { return this.evaluate(([buffer]) => buffer.cursorY); }
  335 |   public get cursorX(): Promise<number> { return this.evaluate(([buffer]) => buffer.cursorX); }
  336 |   public get viewportY(): Promise<number> { return this.evaluate(([buffer]) => buffer.viewportY); }
  337 |   public get baseY(): Promise<number> { return this.evaluate(([buffer]) => buffer.baseY); }
  338 |   public get length(): Promise<number> { return this.evaluate(([buffer]) => buffer.length); }
  339 |   public async getLine(y: number): Promise<TerminalBufferLine | undefined> {
  340 |     const lineHandle = await this._page.evaluateHandle(([buffer, y]) => buffer.getLine(y), [await this._handle, y] as const);
  341 |     const value = await lineHandle.jsonValue();
  342 |     if (value) {
  343 |       return new TerminalBufferLine(this._page, lineHandle as JSHandle<IBufferLine>);
  344 |     }
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
```