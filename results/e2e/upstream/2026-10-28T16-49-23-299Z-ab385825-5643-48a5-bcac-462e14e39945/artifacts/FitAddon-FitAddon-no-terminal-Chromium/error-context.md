# Instructions

- Following Playwright test failed.
- Explain why, be concise, respect Playwright best practices.
- Provide a snippet of code with the fix, if possible.

# Test info

- Name: FitAddon.test.ts >> FitAddon >> no terminal
- Location: addons\addon-fit\test\FitAddon.test.ts:42:7

# Error details

```
Error: page.evaluate: TypeError: Cannot set properties of undefined (setting 'activeVersion')
    at eval (eval at evaluate (:302:30), <anonymous>:3:41)
    at eval (<anonymous>)
    at UtilityScript.evaluate (<anonymous>:302:30)
    at UtilityScript.<anonymous> (<anonymous>:1:44)
```

```
Error: page.exposeFunction: Target page, context or browser has been closed
```

# Page snapshot

```yaml
- generic [ref=e5]:
  - generic:
    - textbox "Terminal input"
```

# Test source

```ts
  49  |   private _disposed: boolean = false;
  50  | 
  51  |   public get event(): IEvent<T, U> {
  52  |     this._event ??= (listener: (arg1: T, arg2: U) => any) => {
  53  |       this._listeners.add(listener);
  54  |       const disposable = {
  55  |         dispose: () => {
  56  |           if (!this._disposed) {
  57  |             this._listeners.delete(listener);
  58  |           }
  59  |         }
  60  |       };
  61  |       return disposable;
  62  |     };
  63  |     return this._event;
  64  |   }
  65  | 
  66  |   public fire(arg1: T, arg2: U): void {
  67  |     const queue: IListener<T, U>[] = [];
  68  |     for (const l of this._listeners.values()) {
  69  |       queue.push(l);
  70  |     }
  71  |     for (let i = 0; i < queue.length; i++) {
  72  |       queue[i].call(undefined, arg1, arg2);
  73  |     }
  74  |   }
  75  | 
  76  |   public dispose(): void {
  77  |     this.clearListeners();
  78  |     this._disposed = true;
  79  |   }
  80  | 
  81  |   public clearListeners(): void {
  82  |     if (this._listeners) {
  83  |       this._listeners.clear();
  84  |     }
  85  |   }
  86  | }
  87  | 
  88  | type EnsureAsync<T> = T extends PromiseLike<any> ? T : Promise<T>;
  89  | type EnsureAsyncProperties<T> = {
  90  |   [Key in keyof T]: EnsureAsync<T[Key]>
  91  | };
  92  | type EnsureAsyncMethods<T> = {
  93  |   [K in keyof T]: T[K] extends (...args: infer Args) => infer Return
  94  |     ? (...args: Args) => Promise<Return>
  95  |     : T[K];
  96  | };
  97  | 
  98  | /**
  99  |  * A type that proxy objects implement that enables overriding a base interface with a set of types
  100 |  * to override as async (ie. properties that are not synced to the node side), and a set of types to
  101 |  * omit from the interface (ie. properties that map to completely different implementations).
  102 |  */
  103 | type PlaywrightApiProxy<TBaseInterface, TAsyncPropOverrides extends keyof TBaseInterface, TAsyncMethodOverrides extends keyof TBaseInterface, TCustomOverrides extends keyof TBaseInterface> = (
  104 |   // Interfaces that the proxy implements as async
  105 |   EnsureAsyncProperties<Pick<TBaseInterface, TAsyncPropOverrides>> &
  106 |   EnsureAsyncMethods<Pick<TBaseInterface, TAsyncMethodOverrides>> &
  107 |   // Interfaces that the proxy implements as is (exclude async/custom)
  108 |   Omit<TBaseInterface, TAsyncPropOverrides | TAsyncMethodOverrides | TCustomOverrides>
  109 | );
  110 | 
  111 | interface ITerminalProxyCustomMethods {
  112 |   evaluate<T>(pageFunction: PageFunction<Terminal[], T>): Promise<T>;
  113 |   write(data: string | Uint8Array): Promise<void>;
  114 | }
  115 | 
  116 | type TerminalProxyAsyncPropOverrides = 'cols' | 'rows' | 'modes';
  117 | type TerminalProxyAsyncMethodOverrides = 'hasSelection' | 'getSelection' | 'getSelectionPosition' | 'registerMarker' | 'registerDecoration';
  118 | type TerminalProxyCustomOverrides = 'buffer' | 'dimensions' | (
  119 |   // The below are not implemented yet
  120 |   'element' |
  121 |   'screenElement' |
  122 |   'textarea' |
  123 |   'markers' |
  124 |   'unicode' |
  125 |   'parser' |
  126 |   'options' |
  127 |   'open' |
  128 |   'attachCustomKeyEventHandler' |
  129 |   'attachCustomWheelEventHandler' |
  130 |   'registerLinkProvider' |
  131 |   'registerCharacterJoiner' |
  132 |   'deregisterCharacterJoiner' |
  133 |   'loadAddon'
  134 | );
  135 | export class TerminalProxy implements ITerminalProxyCustomMethods, PlaywrightApiProxy<Terminal, TerminalProxyAsyncPropOverrides, TerminalProxyAsyncMethodOverrides, TerminalProxyCustomOverrides> {
  136 |   constructor(private readonly _page: Page) {
  137 |   }
  138 | 
  139 |   /**
  140 |    * Initialize the proxy for a new playwright page.
  141 |    */
  142 |   public async initPage(): Promise<void> {
  143 |     await this._page.exposeFunction('onBell', () => this._onBell.fire());
  144 |     await this._page.exposeFunction('onBinary', (e: string) => this._onBinary.fire(e));
  145 |     await this._page.exposeFunction('onCursorMove', () => this._onCursorMove.fire());
  146 |     await this._page.exposeFunction('onData', (e: string) => this._onData.fire(e));
  147 |     await this._page.exposeFunction('onKey', (e: { key: string, domEvent: KeyboardEvent }) => this._onKey.fire(e));
  148 |     await this._page.exposeFunction('onLineFeed', () => this._onLineFeed.fire());
> 149 |     await this._page.exposeFunction('onRender', (e: { start: number, end: number }) => this._onRender.fire(e));
      |                      ^ Error: page.exposeFunction: Target page, context or browser has been closed
  150 |     await this._page.exposeFunction('onResize', (e: { cols: number, rows: number }) => this._onResize.fire(e));
  151 |     await this._page.exposeFunction('onScroll', (e: number) => this._onScroll.fire(e));
  152 |     await this._page.exposeFunction('onSelectionChange', () => this._onSelectionChange.fire());
  153 |     await this._page.exposeFunction('onTitleChange', (e: string) => this._onTitleChange.fire(e));
  154 |     await this._page.exposeFunction('onWriteParsed', () => this._onWriteParsed.fire());
  155 |     await this._page.exposeFunction('onDimensionsChange', (e: IRenderDimensions) => this._onDimensionsChange.fire(e));
  156 |   }
  157 | 
  158 |   /**
  159 |    * Initialize the proxy for a new terminal object.
  160 |    */
  161 |   public async initTerm(): Promise<void> {
  162 |     this._onBell.dispose();
  163 |     this._onBinary.dispose();
  164 |     this._onCursorMove.dispose();
  165 |     this._onData.dispose();
  166 |     this._onKey.dispose();
  167 |     this._onLineFeed.dispose();
  168 |     this._onRender.dispose();
  169 |     this._onResize.dispose();
  170 |     this._onScroll.dispose();
  171 |     this._onSelectionChange.dispose();
  172 |     this._onTitleChange.dispose();
  173 |     this._onWriteParsed.dispose();
  174 |     this._onDimensionsChange.dispose();
  175 | 
  176 |     this._onBell = new EventEmitter();
  177 |     this._onBinary = new EventEmitter();
  178 |     this._onCursorMove = new EventEmitter();
  179 |     this._onData = new EventEmitter();
  180 |     this._onKey = new EventEmitter();
  181 |     this._onLineFeed = new EventEmitter();
  182 |     this._onRender = new EventEmitter();
  183 |     this._onResize = new EventEmitter();
  184 |     this._onScroll = new EventEmitter();
  185 |     this._onSelectionChange = new EventEmitter();
  186 |     this._onTitleChange = new EventEmitter();
  187 |     this._onWriteParsed = new EventEmitter();
  188 |     this._onDimensionsChange = new EventEmitter();
  189 | 
  190 |     await this.evaluate(([term]) => term.onBell((window as any).onBell));
  191 |     await this.evaluate(([term]) => term.onBinary((window as any).onBinary));
  192 |     await this.evaluate(([term]) => term.onCursorMove((window as any).onCursorMove));
  193 |     await this.evaluate(([term]) => term.onData((window as any).onData));
  194 |     await this.evaluate(([term]) => term.onKey((window as any).onKey));
  195 |     await this.evaluate(([term]) => term.onLineFeed((window as any).onLineFeed));
  196 |     await this.evaluate(([term]) => term.onRender((window as any).onRender));
  197 |     await this.evaluate(([term]) => term.onResize((window as any).onResize));
  198 |     await this.evaluate(([term]) => term.onScroll((window as any).onScroll));
  199 |     await this.evaluate(([term]) => term.onSelectionChange((window as any).onSelectionChange));
  200 |     await this.evaluate(([term]) => term.onTitleChange((window as any).onTitleChange));
  201 |     await this.evaluate(([term]) => term.onWriteParsed((window as any).onWriteParsed));
  202 |     await this.evaluate(([term]) => term.onDimensionsChange((window as any).onDimensionsChange));
  203 |   }
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
```