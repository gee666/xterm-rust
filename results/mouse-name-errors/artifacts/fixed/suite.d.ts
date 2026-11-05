/* tslint:disable */
/* eslint-disable */
export function loadFonts(fonts?: Array<any> | null): Promise<any>;
/**
 * Register the source joiner directly, without the addon's open/style policy.
 * Omitted fallbacks mean an empty list, unlike the class constructor defaults.
 */
export function enableLigatures(terminal: any, fallbacks: any): any;
/**
 * Handler for `console.log` invocations.
 *
 * If a test is currently running it takes the `args` array and stringifies
 * it and appends it to the current output of the test. Otherwise it passes
 * the arguments to the original `console.log` function, psased as
 * `original`.
 */
export function __wbgtest_console_log(args: Array<any>): void;
/**
 * Handler for `console.debug` invocations. See above.
 */
export function __wbgtest_console_debug(args: Array<any>): void;
/**
 * Handler for `console.info` invocations. See above.
 */
export function __wbgtest_console_info(args: Array<any>): void;
/**
 * Handler for `console.warn` invocations. See above.
 */
export function __wbgtest_console_warn(args: Array<any>): void;
/**
 * Handler for `console.error` invocations. See above.
 */
export function __wbgtest_console_error(args: Array<any>): void;
export function __wbgtest_cov_dump(): Uint8Array | undefined;
export class AttachAddon {
  free(): void;
  constructor(socket: WebSocket, options: any);
  activate(terminal: Terminal): void;
  dispose(): void;
}
export class Base64 {
  free(): void;
  constructor();
  encodeText(value: any): any;
  decodeText(value: any): any;
}
export class BrowserClipboardProvider {
  free(): void;
  constructor();
  readText(_selection: any): any;
  writeText(_selection: any, text: any): any;
}
export class BrowserHost {
  free(): void;
  constructor();
  /**
   * Reopening does not recreate nodes or move the original terminal root.
   */
  open(parent: HTMLElement): void;
  focus(): void;
  blur(): void;
  dispose(): void;
  readonly element: HTMLElement | undefined;
  readonly screenElement: HTMLElement | undefined;
  readonly textarea: HTMLTextAreaElement | undefined;
}
export class BrowserMouseState {
  private constructor();
  free(): void;
  /**
   * Read the native active mode before touching the event. A field accessor
   * may switch modes recursively, but the current call keeps its policy.
   */
  restrictMouseEvent(event: any): boolean;
  encodeMouseEvent(event: any): any;
  reset(): void;
  /**
   * Built-in string keys share native validation and protocol-change events.
   */
  activeProtocol: string;
  activeEncoding: string;
  readonly areMouseEventsActive: boolean;
  readonly isDefaultEncoding: boolean;
  readonly isPixelEncoding: boolean;
  readonly onProtocolChange: Function;
}
export class BrowserRuntime {
  free(): void;
  constructor();
  readonly strings: object;
}
export class BufferCell {
  private constructor();
  free(): void;
  isUnderline(): number;
  getUnderlineStyle(): any;
  getUnderlineColor(): number;
  getUnderlineColorMode(): number;
  isUnderlineColorRGB(): boolean;
  isUnderlineColorPalette(): boolean;
  isUnderlineColorDefault(): boolean;
  attributesEquals(other: any): boolean;
  getChars(): any;
  getCode(): number;
  getWidth(): number;
  getFgColorMode(): number;
  getBgColorMode(): number;
  getFgColor(): number;
  getBgColor(): number;
  isBold(): number;
  isItalic(): number;
  isDim(): number;
  isBlink(): number;
  isInverse(): number;
  isInvisible(): number;
  isStrikethrough(): number;
  isOverline(): number;
  isFgRGB(): boolean;
  isFgPalette(): boolean;
  isFgDefault(): boolean;
  isBgRGB(): boolean;
  isBgPalette(): boolean;
  isBgDefault(): boolean;
  isAttributeDefault(): boolean;
  get content(): number | undefined;
  set content(value: number | null | undefined);
  get fg(): number | undefined;
  set fg(value: number | null | undefined);
  get bg(): number | undefined;
  set bg(value: number | null | undefined);
  get combinedData(): string | undefined;
  set combinedData(value: string | null | undefined);
  extended: any;
}
export class BufferLine {
  private constructor();
  free(): void;
  getCell(index: number, target: any): any;
  translateToString(trim_right?: boolean | null, start?: number | null, end?: number | null): string;
  /**
   * Copies packed row storage without exporting a mutable WASM memory view.
   */
  readonly _data: Uint32Array;
  readonly _combined: object;
  readonly _extendedAttrs: object;
  readonly _isWrapped: boolean;
  readonly isWrapped: boolean;
  readonly length: number;
}
export class BufferNamespace {
  private constructor();
  free(): void;
  readonly normal: any;
  readonly alternate: any;
  readonly active: any;
  readonly onBufferChange: Function;
}
export class BufferView {
  private constructor();
  free(): void;
  getNullCell(): BufferCell;
  getLine(index: number): any;
  readonly type: string;
  readonly cursorY: number;
  readonly cursorX: number;
  readonly viewportY: number;
  readonly baseY: number;
  readonly length: number;
}
export class ClipboardAddon {
  free(): void;
  constructor(codec: any, provider: any);
  activate(terminal: Terminal): void;
  dispose(): void;
}
/**
 * Read-only inspection of the real core's disposal flag, not a second store.
 */
export class CoreDisposalStore {
  private constructor();
  free(): void;
  readonly _isDisposed: boolean;
  readonly isDisposed: boolean;
}
export class CoreModeServiceView {
  private constructor();
  free(): void;
  readonly decPrivateModes: any;
}
export class CoreServices {
  private constructor();
  free(): void;
  readonly _store: any;
  readonly _renderService: any;
  readonly unicodeService: any;
  readonly mouseStateService: any;
  readonly coreService: any;
  readonly _bufferService: any;
}
export class DecPrivateModesView {
  private constructor();
  free(): void;
  readonly applicationCursorKeys: boolean;
  readonly applicationKeypad: boolean;
  readonly bracketedPasteMode: boolean;
  readonly colorSchemeUpdates: boolean;
  readonly cursorBlink: any;
  readonly cursorStyle: any;
  readonly origin: boolean;
  readonly reverseWraparound: boolean;
  readonly sendFocus: boolean;
  readonly synchronizedOutput: boolean;
  readonly win32InputMode: boolean;
  readonly wraparound: boolean;
}
export class ExtendedCellAttributes {
  private constructor();
  free(): void;
  isEmpty(): boolean;
  clone(): ExtendedCellAttributes;
  _ext: number;
  _urlId: number;
  ext: number;
  urlId: number;
  underlineStyle: number;
  underlineColor: number;
  underlineVariantOffset: number;
  payload: any;
}
export class FitAddon {
  free(): void;
  constructor();
  activate(terminal: Terminal): void;
  proposeDimensions(): any;
  fit(): void;
  dispose(): void;
}
/**
 * Image protocols, control sequences and image storage for the Rust terminal.
 */
export class ImageAddon {
  free(): void;
  constructor(options: any);
  activate(terminal: Terminal): void;
  dispose(): void;
  reset(): boolean;
  getImageAtBufferCell(x: number, y: number): any;
  extractTileAtBufferCell(x: number, y: number): any;
  /**
   * Live read-only storage inspection. Writable source-private fields are not
   * part of this binding; image decoding and storage remain authoritative Rust.
   */
  readonly _storage: any;
  /**
   * Read-only native handler inspection, not a writable private Map.
   */
  readonly _handlers: any;
  /**
   * Live read-only options. Placeholder, storage, SIXEL, IIP and Kitty size-limit
   * values retain JavaScript identity; other fields remain typed projections.
   */
  readonly _opts: any;
  readonly onImageAdded: Function;
  get storageLimit(): number;
  set storageLimit(value: any);
  readonly storageUsage: number;
  /**
   * Return the original JavaScript value, not its rendering truthiness.
   */
  showPlaceholder: any;
}
export class ImageHandlerInspection {
  private constructor();
  free(): void;
  reset(): void;
  kitty(): any;
  readonly identity: number;
  readonly kind: string;
}
export class ImageHandlersIterator {
  private constructor();
  free(): void;
  next(): any;
}
export class ImageHandlersReader {
  private constructor();
  free(): void;
  has(key: any): boolean;
  get(key: any): any;
  entries(): ImageHandlersIterator;
  readonly size: number;
}
export class ImageOptionsReader {
  private constructor();
  free(): void;
  get(key: string): any;
}
export class ImageSpecInspection {
  private constructor();
  free(): void;
  readonly orig: any;
  readonly actual: any;
  readonly origCellSize: any;
  readonly actualCellSize: any;
  readonly tileCount: number;
  readonly bufferType: string;
  readonly layer: string;
  readonly zIndex: number;
}
export class ImageStoreIterator {
  private constructor();
  free(): void;
  next(): any;
}
export class ImageStoreReader {
  private constructor();
  free(): void;
  has(key: any): boolean;
  get(key: any): any;
  entries(): ImageStoreIterator;
  readonly size: number;
}
export class InternalBufferServiceView {
  private constructor();
  free(): void;
  readonly cols: number;
  readonly rows: number;
  readonly buffer: any;
}
export class InternalBufferView {
  private constructor();
  free(): void;
  readonly scrollTop: number;
  readonly scrollBottom: number;
  readonly x: number;
  readonly y: number;
  readonly ybase: number;
  readonly ydisp: number;
}
/**
 * Internal diagnostic view. Raw writable service replacement remains unsupported.
 */
export class InternalRenderServiceView {
  private constructor();
  free(): void;
  readonly dimensions: any;
}
export class KittyImageInspection {
  private constructor();
  free(): void;
  readonly identity: number;
  readonly id: number;
  readonly width: number;
  readonly height: number;
  readonly format: number;
  readonly compression: string;
  readonly data: any;
}
export class KittyImagesReader {
  private constructor();
  free(): void;
  has(key: any): boolean;
  get(key: any): any;
  entries(): KittyKeys;
  readonly size: number;
}
export class KittyKeys {
  private constructor();
  free(): void;
  next(): any;
}
/**
 * Membership-only diagnostics. Pending command/decoder objects are deliberately
 * not presented as detached snapshots of the source's mutable records.
 */
export class KittyPendingReader {
  private constructor();
  free(): void;
  has(key: any): boolean;
  entries(): KittyKeys;
  readonly size: number;
}
export class KittyPlacementsReader {
  private constructor();
  free(): void;
  has(key: any): boolean;
  get(key: any): any;
  entries(): KittyKeys;
  readonly size: number;
}
export class LigaturesAddon {
  free(): void;
  constructor(options: any);
  activate(terminal: any): void;
  dispose(): void;
}
export class Marker {
  private constructor();
  free(): void;
  /**
   * Retain the original raw resource without inspecting it until disposal.
   * Duplicate and late registrations are intentional source behavior.
   */
  register(resource: any): any;
  dispose(): void;
  readonly id: number;
  readonly line: any;
  readonly isDisposed: boolean;
  readonly onDispose: Function;
}
export class ParserApi {
  private constructor();
  free(): void;
  registerCsiHandler(id: any, callback: Function): any;
  addCsiHandler(id: any, callback: Function): any;
  registerDcsHandler(id: any, callback: Function): any;
  addDcsHandler(id: any, callback: Function): any;
  registerEscHandler(id: any, callback: Function): any;
  addEscHandler(id: any, callback: Function): any;
  registerOscHandler(id: number, callback: Function): any;
  addOscHandler(id: number, callback: Function): any;
  registerApcHandler(id: any, callback: Function): any;
}
export class ProgramStatusAddon {
  free(): void;
  constructor();
  activate(terminal: Terminal): void;
  clear(id: any): void;
  processExited(): void;
  promptStarted(): void;
  dispose(): void;
  readonly records: any;
  readonly onChange: any;
}
export class ProgressAddon {
  free(): void;
  constructor();
  activate(terminal: Terminal): void;
  dispose(): void;
  progress: any;
  readonly onChange: any;
}
export class SearchAddon {
  free(): void;
  constructor(initial_options: any);
  activate(terminal: Terminal): void;
  findNext(term: any, options: any, internal_options?: any | null): boolean;
  findPrevious(term: any, options: any, internal_options?: any | null): boolean;
  clearDecorations(retain_cached_term?: any | null): void;
  clearActiveDecoration(): void;
  dispose(): void;
  readonly onBeforeSearch: Function;
  readonly onAfterSearch: Function;
  readonly onDidChangeResults: Function;
}
export class SearchSubscription {
  private constructor();
  free(): void;
  dispose(): void;
}
/**
 * The compatible JS facade retains its host in a JS field. Rust stores only a
 * weak link, so an unreachable manager/addon/Terminal cycle is collectible.
 */
export class SerializeAddon {
  free(): void;
  constructor();
  activate(terminal: Terminal): void;
  serialize(options?: any | null): string;
  serializeAsHTML(options?: any | null): string;
  dispose(): void;
}
/**
 * Browser API over the Rust engine. No upstream JavaScript terminal is loaded.
 */
export class Terminal {
  free(): void;
  open(parent: any): void;
  refresh(start: number, end: number): void;
  attachCustomKeyEventHandler(handler: Function): void;
  attachCustomWheelEventHandler(handler: Function): void;
  paste(data: string): void;
  registerCharacterJoiner(handler: Function): number;
  deregisterCharacterJoiner(id: number): void;
  registerLinkProvider(provider: any): any;
  registerMarker(offset: any): any;
  constructor(initial_options: any);
  _setRuntime(runtime: BrowserRuntime): void;
  hasSelection(): boolean;
  getSelection(): string;
  getSelectionPosition(): any;
  select(column: number, row: number, length: number): void;
  clearSelection(): void;
  selectAll(): void;
  selectLines(start: number, end: number): void;
  write(value: any, completion?: Function | null): void;
  writeln(value: any, completion?: Function | null): void;
  input(value: string, was_user_input?: boolean | null): void;
  resize(columns: number, rows: number): void;
  reset(): void;
  clear(): void;
  scrollLines(amount: number): void;
  scrollPages(amount: number): void;
  scrollToTop(): void;
  scrollToBottom(): void;
  scrollToLine(line: number): void;
  focus(): void;
  blur(): void;
  _takeAddonOwner(): Array<any>;
  _loadAddon(public_terminal: any, addon: any): void;
  dispose(): void;
  _disposeCore(): void;
  readonly dimensions: any;
  /**
   * A live read-only Array view, preserving list and marker handle identity.
   * Source's writable Array operations remain unsupported.
   */
  readonly markers: Array<any>;
  readonly modes: any;
  readonly cols: number;
  readonly rows: number;
  get options(): object;
  set options(value: any);
  readonly buffer: any;
  readonly parser: any;
  readonly unicode: UnicodeApi;
  readonly element: HTMLElement | undefined;
  readonly screenElement: HTMLElement | undefined;
  readonly textarea: HTMLTextAreaElement | undefined;
  readonly onBell: Function;
  readonly onBinary: Function;
  readonly onCursorMove: Function;
  readonly onData: Function;
  readonly onLineFeed: Function;
  readonly onResize: Function;
  readonly onScroll: Function;
  readonly onTitleChange: Function;
  readonly onWriteParsed: Function;
  readonly onKey: Function;
  readonly onRender: Function;
  readonly onSelectionChange: Function;
  readonly onDimensionsChange: Function;
  readonly _core: any;
}
export class Unicode11Addon {
  free(): void;
  constructor();
  activate(terminal: Terminal): void;
  /**
   * Source disposal leaves the registered provider available for selection.
   */
  dispose(): void;
}
/**
 * Each public terminal getter returns a fresh view over the same Rust service.
 */
export class UnicodeApi {
  private constructor();
  free(): void;
  register(provider: any): void;
  readonly versions: Array<any>;
  activeVersion: string;
}
export class UnicodeGraphemesAddon {
  free(): void;
  constructor();
  activate(terminal: Terminal): void;
  dispose(): void;
}
/**
 * A retained view of the terminal's actual service, not a separate registry.
 * Only the width entry point is exported here; the public Unicode API owns
 * registration and version selection.
 */
export class UnicodeService {
  private constructor();
  free(): void;
  getStringCellWidth(text: string): number;
}
/**
 * Runtime test harness support instantiated in JS.
 *
 * The node.js entry script instantiates a `Context` here which is used to
 * drive test execution.
 */
export class WasmBindgenTestContext {
  free(): void;
  /**
   * Creates a new context ready to run tests.
   *
   * A `Context` is the main structure through which test execution is
   * coordinated, and this will collect output and results for all executed
   * tests.
   */
  constructor();
  /**
   * Handle `--include-ignored` flag.
   */
  include_ignored(include_ignored: boolean): void;
  /**
   * Handle filter argument.
   */
  filtered_count(filtered: number): void;
  /**
   * Executes a list of tests, returning a promise representing their
   * eventual completion.
   *
   * This is the main entry point for executing tests. All the tests passed
   * in are the JS `Function` object that was plucked off the
   * `WebAssembly.Instance` exports list.
   *
   * The promise returned resolves to either `true` if all tests passed or
   * `false` if at least one test failed.
   */
  run(tests: any[]): Promise<any>;
}
export class WebFontsAddon {
  free(): void;
  constructor(initial_relayout?: boolean | null);
  activate(terminal: any): void;
  dispose(): void;
  loadFonts(fonts?: Array<any> | null): Promise<any>;
  relayout(): Promise<any>;
  initialRelayout: boolean;
}
export class WebLinksAddon {
  free(): void;
  constructor(handler: any, options: any);
  activate(terminal: Terminal): void;
  dispose(): void;
  /**
   * The unchanged upstream suite updates this live constructor options bag.
   */
  readonly _options: any;
}

export type InitInput = RequestInfo | URL | Response | BufferSource | WebAssembly.Module;

export interface InitOutput {
  readonly memory: WebAssembly.Memory;
  readonly __wbgt__wasm_terminal_mouse_names::unknown_string_names_preserve_error_text_state_and_events: (a: number) => void;
  readonly __wbgt__wasm_terminal_mouse_names::all_2048_lone_surrogates_survive_both_mode_errors: (a: number) => void;
  readonly __wbgt__wasm_terminal_mouse_names::seeded_names_and_long_names_preserve_every_code_unit: (a: number) => void;
  readonly __wbgt__wasm_terminal_mouse_names::listener_reentry_keeps_error_identity_and_committed_modes: (a: number) => void;
  readonly __wbgt__wasm_terminal_mouse_names::parsed_modes_and_reset_survive_rejected_names: (a: number) => void;
  readonly main: (a: number, b: number) => number;
  readonly __wbg_programstatusaddon_free: (a: number, b: number) => void;
  readonly programstatusaddon_new: () => number;
  readonly programstatusaddon_activate: (a: number, b: number) => [number, number];
  readonly programstatusaddon_records: (a: number) => [number, number, number];
  readonly programstatusaddon_onChange: (a: number) => any;
  readonly programstatusaddon_clear: (a: number, b: any) => [number, number];
  readonly programstatusaddon_processExited: (a: number) => [number, number];
  readonly programstatusaddon_promptStarted: (a: number) => [number, number];
  readonly programstatusaddon_dispose: (a: number) => void;
  readonly __wbg_progressaddon_free: (a: number, b: number) => void;
  readonly progressaddon_new: () => number;
  readonly progressaddon_activate: (a: number, b: number) => [number, number];
  readonly progressaddon_progress: (a: number) => [number, number, number];
  readonly progressaddon_set_progress: (a: number, b: any) => [number, number];
  readonly progressaddon_onChange: (a: number) => any;
  readonly progressaddon_dispose: (a: number) => void;
  readonly terminal_open: (a: number, b: any) => [number, number];
  readonly terminal_refresh: (a: number, b: number, c: number) => [number, number];
  readonly terminal_dimensions: (a: number) => [number, number, number];
  readonly terminal_attachCustomKeyEventHandler: (a: number, b: any) => void;
  readonly terminal_attachCustomWheelEventHandler: (a: number, b: any) => void;
  readonly terminal_paste: (a: number, b: any) => [number, number];
  readonly __wbg_internalbufferserviceview_free: (a: number, b: number) => void;
  readonly internalbufferserviceview_cols: (a: number) => number;
  readonly internalbufferserviceview_rows: (a: number) => number;
  readonly internalbufferserviceview_buffer: (a: number) => any;
  readonly __wbg_internalbufferview_free: (a: number, b: number) => void;
  readonly internalbufferview_scrollTop: (a: number) => [number, number, number];
  readonly internalbufferview_scrollBottom: (a: number) => [number, number, number];
  readonly internalbufferview_x: (a: number) => [number, number, number];
  readonly internalbufferview_y: (a: number) => [number, number, number];
  readonly internalbufferview_ybase: (a: number) => [number, number, number];
  readonly internalbufferview_ydisp: (a: number) => [number, number, number];
  readonly __wbg_coremodeserviceview_free: (a: number, b: number) => void;
  readonly coremodeserviceview_decPrivateModes: (a: number) => any;
  readonly __wbg_decprivatemodesview_free: (a: number, b: number) => void;
  readonly decprivatemodesview_applicationCursorKeys: (a: number) => number;
  readonly decprivatemodesview_applicationKeypad: (a: number) => number;
  readonly decprivatemodesview_bracketedPasteMode: (a: number) => number;
  readonly decprivatemodesview_colorSchemeUpdates: (a: number) => number;
  readonly decprivatemodesview_cursorBlink: (a: number) => any;
  readonly decprivatemodesview_cursorStyle: (a: number) => any;
  readonly decprivatemodesview_origin: (a: number) => number;
  readonly decprivatemodesview_reverseWraparound: (a: number) => number;
  readonly decprivatemodesview_sendFocus: (a: number) => number;
  readonly decprivatemodesview_synchronizedOutput: (a: number) => number;
  readonly decprivatemodesview_win32InputMode: (a: number) => number;
  readonly decprivatemodesview_wraparound: (a: number) => number;
  readonly terminal_registerCharacterJoiner: (a: number, b: any) => [number, number, number];
  readonly terminal_deregisterCharacterJoiner: (a: number, b: number) => [number, number];
  readonly terminal_registerLinkProvider: (a: number, b: any) => any;
  readonly __wbg_marker_free: (a: number, b: number) => void;
  readonly marker_id: (a: number) => number;
  readonly marker_line: (a: number) => any;
  readonly marker_isDisposed: (a: number) => number;
  readonly marker_onDispose: (a: number) => any;
  readonly marker_register: (a: number, b: any) => any;
  readonly marker_dispose: (a: number) => [number, number];
  readonly terminal_registerMarker: (a: number, b: any) => [number, number, number];
  readonly terminal_markers: (a: number) => [number, number, number];
  readonly terminal_modes: (a: number) => [number, number, number];
  readonly __wbg_terminal_free: (a: number, b: number) => void;
  readonly terminal_new: (a: any) => [number, number, number];
  readonly terminal__setRuntime: (a: number, b: number) => [number, number];
  readonly terminal_cols: (a: number) => [number, number, number];
  readonly terminal_rows: (a: number) => [number, number, number];
  readonly terminal_options: (a: number) => any;
  readonly terminal_set_options: (a: number, b: any) => [number, number];
  readonly terminal_buffer: (a: number) => any;
  readonly terminal_parser: (a: number) => any;
  readonly terminal_unicode: (a: number) => [number, number, number];
  readonly terminal_element: (a: number) => any;
  readonly terminal_screenElement: (a: number) => any;
  readonly terminal_textarea: (a: number) => any;
  readonly terminal_onBell: (a: number) => any;
  readonly terminal_onBinary: (a: number) => any;
  readonly terminal_onCursorMove: (a: number) => any;
  readonly terminal_onData: (a: number) => any;
  readonly terminal_onLineFeed: (a: number) => any;
  readonly terminal_onResize: (a: number) => any;
  readonly terminal_onScroll: (a: number) => any;
  readonly terminal_onTitleChange: (a: number) => any;
  readonly terminal_onWriteParsed: (a: number) => any;
  readonly terminal_onKey: (a: number) => any;
  readonly terminal_onRender: (a: number) => any;
  readonly terminal_onSelectionChange: (a: number) => any;
  readonly terminal_onDimensionsChange: (a: number) => any;
  readonly __wbg_browserhost_free: (a: number, b: number) => void;
  readonly browserhost_new: () => number;
  readonly browserhost_open: (a: number, b: any) => [number, number];
  readonly browserhost_element: (a: number) => any;
  readonly browserhost_screenElement: (a: number) => any;
  readonly browserhost_textarea: (a: number) => any;
  readonly browserhost_focus: (a: number) => [number, number];
  readonly browserhost_blur: (a: number) => [number, number];
  readonly browserhost_dispose: (a: number) => void;
  readonly __wbg_imagestorereader_free: (a: number, b: number) => void;
  readonly imagestorereader_size: (a: number) => number;
  readonly imagestorereader_has: (a: number, b: any) => number;
  readonly imagestorereader_get: (a: number, b: any) => any;
  readonly imagestorereader_entries: (a: number) => number;
  readonly __wbg_imagestoreiterator_free: (a: number, b: number) => void;
  readonly imagestoreiterator_next: (a: number) => any;
  readonly __wbg_imagespecinspection_free: (a: number, b: number) => void;
  readonly imagespecinspection_orig: (a: number) => [number, number, number];
  readonly imagespecinspection_actual: (a: number) => [number, number, number];
  readonly imagespecinspection_origCellSize: (a: number) => [number, number, number];
  readonly imagespecinspection_actualCellSize: (a: number) => [number, number, number];
  readonly imagespecinspection_tileCount: (a: number) => number;
  readonly imagespecinspection_bufferType: (a: number) => [number, number];
  readonly imagespecinspection_layer: (a: number) => [number, number];
  readonly imagespecinspection_zIndex: (a: number) => number;
  readonly __wbg_internalrenderserviceview_free: (a: number, b: number) => void;
  readonly internalrenderserviceview_dimensions: (a: number) => [number, number, number];
  readonly __wbg_browsermousestate_free: (a: number, b: number) => void;
  readonly browsermousestate_activeProtocol: (a: number) => [number, number];
  readonly browsermousestate_set_activeProtocol: (a: number, b: any) => [number, number];
  readonly browsermousestate_activeEncoding: (a: number) => [number, number];
  readonly browsermousestate_set_activeEncoding: (a: number, b: any) => [number, number];
  readonly browsermousestate_areMouseEventsActive: (a: number) => number;
  readonly browsermousestate_isDefaultEncoding: (a: number) => number;
  readonly browsermousestate_isPixelEncoding: (a: number) => number;
  readonly browsermousestate_onProtocolChange: (a: number) => any;
  readonly browsermousestate_restrictMouseEvent: (a: number, b: any) => [number, number, number];
  readonly browsermousestate_encodeMouseEvent: (a: number, b: any) => [number, number, number];
  readonly browsermousestate_reset: (a: number) => [number, number];
  readonly terminal_hasSelection: (a: number) => [number, number, number];
  readonly terminal_getSelection: (a: number) => [number, number, number];
  readonly terminal_getSelectionPosition: (a: number) => [number, number, number];
  readonly terminal_select: (a: number, b: number, c: number, d: number) => [number, number];
  readonly terminal_clearSelection: (a: number) => [number, number];
  readonly terminal_selectAll: (a: number) => [number, number];
  readonly terminal_selectLines: (a: number, b: number, c: number) => [number, number];
  readonly __wbg_searchsubscription_free: (a: number, b: number) => void;
  readonly searchsubscription_dispose: (a: number) => void;
  readonly __wbg_searchaddon_free: (a: number, b: number) => void;
  readonly searchaddon_new: (a: any) => [number, number, number];
  readonly searchaddon_activate: (a: number, b: number) => [number, number];
  readonly searchaddon_findNext: (a: number, b: any, c: any, d: number) => [number, number, number];
  readonly searchaddon_findPrevious: (a: number, b: any, c: any, d: number) => [number, number, number];
  readonly searchaddon_clearDecorations: (a: number, b: number) => [number, number];
  readonly searchaddon_clearActiveDecoration: (a: number) => [number, number];
  readonly searchaddon_dispose: (a: number) => [number, number];
  readonly searchaddon_onBeforeSearch: (a: number) => any;
  readonly searchaddon_onAfterSearch: (a: number) => any;
  readonly searchaddon_onDidChangeResults: (a: number) => any;
  readonly __wbg_unicode11addon_free: (a: number, b: number) => void;
  readonly unicode11addon_new: () => number;
  readonly unicode11addon_activate: (a: number, b: number) => [number, number];
  readonly unicode11addon_dispose: (a: number) => void;
  readonly __wbg_unicodegraphemesaddon_free: (a: number, b: number) => void;
  readonly unicodegraphemesaddon_new: () => number;
  readonly unicodegraphemesaddon_activate: (a: number, b: number) => [number, number];
  readonly unicodegraphemesaddon_dispose: (a: number) => [number, number];
  readonly __wbg_attachaddon_free: (a: number, b: number) => void;
  readonly attachaddon_new: (a: any, b: any) => [number, number, number];
  readonly attachaddon_activate: (a: number, b: number) => [number, number];
  readonly attachaddon_dispose: (a: number) => void;
  readonly __wbg_base64_free: (a: number, b: number) => void;
  readonly base64_new: () => number;
  readonly base64_encodeText: (a: number, b: any) => [number, number, number];
  readonly base64_decodeText: (a: number, b: any) => [number, number, number];
  readonly __wbg_browserclipboardprovider_free: (a: number, b: number) => void;
  readonly browserclipboardprovider_new: () => number;
  readonly browserclipboardprovider_readText: (a: number, b: any) => [number, number, number];
  readonly browserclipboardprovider_writeText: (a: number, b: any, c: any) => [number, number, number];
  readonly __wbg_clipboardaddon_free: (a: number, b: number) => void;
  readonly clipboardaddon_new: (a: any, b: any) => number;
  readonly clipboardaddon_activate: (a: number, b: number) => [number, number];
  readonly clipboardaddon_dispose: (a: number) => void;
  readonly __wbg_weblinksaddon_free: (a: number, b: number) => void;
  readonly weblinksaddon_new: (a: any, b: any) => number;
  readonly weblinksaddon__options: (a: number) => any;
  readonly weblinksaddon_activate: (a: number, b: number) => [number, number];
  readonly weblinksaddon_dispose: (a: number) => void;
  readonly __wbg_imageaddon_free: (a: number, b: number) => void;
  readonly imageaddon_new: (a: any) => [number, number, number];
  readonly imageaddon_activate: (a: number, b: number) => [number, number];
  readonly imageaddon_dispose: (a: number) => [number, number];
  readonly imageaddon_reset: (a: number) => [number, number, number];
  readonly imageaddon__storage: (a: number) => any;
  readonly imageaddon__handlers: (a: number) => any;
  readonly imageaddon__opts: (a: number) => any;
  readonly imageaddon_onImageAdded: (a: number) => any;
  readonly imageaddon_storageLimit: (a: number) => number;
  readonly imageaddon_set_storageLimit: (a: number, b: any) => [number, number];
  readonly imageaddon_storageUsage: (a: number) => [number, number, number];
  readonly imageaddon_showPlaceholder: (a: number) => any;
  readonly imageaddon_set_showPlaceholder: (a: number, b: any) => [number, number];
  readonly imageaddon_getImageAtBufferCell: (a: number, b: number, c: number) => [number, number, number];
  readonly imageaddon_extractTileAtBufferCell: (a: number, b: number, c: number) => [number, number, number];
  readonly __wbg_imagehandlersreader_free: (a: number, b: number) => void;
  readonly imagehandlersreader_size: (a: number) => number;
  readonly imagehandlersreader_has: (a: number, b: any) => number;
  readonly imagehandlersreader_get: (a: number, b: any) => any;
  readonly imagehandlersreader_entries: (a: number) => number;
  readonly __wbg_imagehandlersiterator_free: (a: number, b: number) => void;
  readonly imagehandlersiterator_next: (a: number) => any;
  readonly __wbg_imagehandlerinspection_free: (a: number, b: number) => void;
  readonly imagehandlerinspection_identity: (a: number) => number;
  readonly imagehandlerinspection_kind: (a: number) => [number, number];
  readonly imagehandlerinspection_reset: (a: number) => [number, number];
  readonly imagehandlerinspection_kitty: (a: number) => [number, number, number];
  readonly __wbg_kittyimagesreader_free: (a: number, b: number) => void;
  readonly kittyimagesreader_size: (a: number) => [number, number, number];
  readonly kittyimagesreader_has: (a: number, b: any) => [number, number, number];
  readonly kittyimagesreader_get: (a: number, b: any) => [number, number, number];
  readonly kittyimagesreader_entries: (a: number) => number;
  readonly __wbg_kittyplacementsreader_free: (a: number, b: number) => void;
  readonly kittyplacementsreader_size: (a: number) => [number, number, number];
  readonly kittyplacementsreader_has: (a: number, b: any) => [number, number, number];
  readonly kittyplacementsreader_get: (a: number, b: any) => [number, number, number];
  readonly kittyplacementsreader_entries: (a: number) => number;
  readonly __wbg_kittypendingreader_free: (a: number, b: number) => void;
  readonly kittypendingreader_size: (a: number) => [number, number, number];
  readonly kittypendingreader_has: (a: number, b: any) => [number, number, number];
  readonly kittypendingreader_entries: (a: number) => number;
  readonly __wbg_kittykeys_free: (a: number, b: number) => void;
  readonly kittykeys_next: (a: number) => [number, number, number];
  readonly __wbg_kittyimageinspection_free: (a: number, b: number) => void;
  readonly kittyimageinspection_identity: (a: number) => number;
  readonly kittyimageinspection_id: (a: number) => number;
  readonly kittyimageinspection_width: (a: number) => number;
  readonly kittyimageinspection_height: (a: number) => number;
  readonly kittyimageinspection_format: (a: number) => number;
  readonly kittyimageinspection_compression: (a: number) => any;
  readonly kittyimageinspection_data: (a: number) => [number, number, number];
  readonly __wbg_imageoptionsreader_free: (a: number, b: number) => void;
  readonly imageoptionsreader_get: (a: number, b: number, c: number) => any;
  readonly __wbg_serializeaddon_free: (a: number, b: number) => void;
  readonly serializeaddon_new: () => number;
  readonly serializeaddon_activate: (a: number, b: number) => void;
  readonly serializeaddon_serialize: (a: number, b: number) => [number, number, number];
  readonly serializeaddon_serializeAsHTML: (a: number, b: number) => [number, number, number];
  readonly serializeaddon_dispose: (a: number) => void;
  readonly __wbg_coreservices_free: (a: number, b: number) => void;
  readonly __wbg_coredisposalstore_free: (a: number, b: number) => void;
  readonly coredisposalstore__isDisposed: (a: number) => number;
  readonly coredisposalstore_isDisposed: (a: number) => number;
  readonly coreservices__store: (a: number) => any;
  readonly coreservices__renderService: (a: number) => any;
  readonly coreservices_unicodeService: (a: number) => any;
  readonly coreservices_mouseStateService: (a: number) => any;
  readonly coreservices_coreService: (a: number) => any;
  readonly coreservices__bufferService: (a: number) => any;
  readonly terminal__core: (a: number) => any;
  readonly terminal_write: (a: number, b: any, c: number) => [number, number];
  readonly terminal_writeln: (a: number, b: any, c: number) => [number, number];
  readonly terminal_input: (a: number, b: any, c: number) => [number, number];
  readonly terminal_resize: (a: number, b: number, c: number) => [number, number];
  readonly terminal_reset: (a: number) => [number, number];
  readonly terminal_clear: (a: number) => [number, number];
  readonly terminal_scrollLines: (a: number, b: number) => [number, number];
  readonly terminal_scrollPages: (a: number, b: number) => [number, number];
  readonly terminal_scrollToTop: (a: number) => [number, number];
  readonly terminal_scrollToBottom: (a: number) => [number, number];
  readonly terminal_scrollToLine: (a: number, b: number) => [number, number];
  readonly terminal_focus: (a: number) => [number, number];
  readonly terminal_blur: (a: number) => [number, number];
  readonly terminal__takeAddonOwner: (a: number) => [number, number, number];
  readonly terminal__loadAddon: (a: number, b: any, c: any) => [number, number];
  readonly terminal_dispose: (a: number) => [number, number];
  readonly terminal__disposeCore: (a: number) => [number, number];
  readonly __wbg_browserruntime_free: (a: number, b: number) => void;
  readonly browserruntime_new: () => number;
  readonly browserruntime_strings: (a: number) => any;
  readonly __wbg_fitaddon_free: (a: number, b: number) => void;
  readonly fitaddon_new: () => number;
  readonly fitaddon_activate: (a: number, b: number) => void;
  readonly fitaddon_proposeDimensions: (a: number) => [number, number, number];
  readonly fitaddon_fit: (a: number) => [number, number];
  readonly fitaddon_dispose: (a: number) => void;
  readonly loadFonts: (a: number) => any;
  readonly __wbg_webfontsaddon_free: (a: number, b: number) => void;
  readonly webfontsaddon_new: (a: number) => number;
  readonly webfontsaddon_initialRelayout: (a: number) => number;
  readonly webfontsaddon_set_initialRelayout: (a: number, b: number) => void;
  readonly webfontsaddon_activate: (a: number, b: any) => [number, number];
  readonly webfontsaddon_dispose: (a: number) => void;
  readonly webfontsaddon_loadFonts: (a: number, b: number) => any;
  readonly webfontsaddon_relayout: (a: number) => any;
  readonly enableLigatures: (a: any, b: any) => [number, number, number];
  readonly __wbg_ligaturesaddon_free: (a: number, b: number) => void;
  readonly ligaturesaddon_new: (a: any) => [number, number, number];
  readonly ligaturesaddon_activate: (a: number, b: any) => [number, number];
  readonly ligaturesaddon_dispose: (a: number) => [number, number];
  readonly buffercell_isUnderline: (a: number) => [number, number, number];
  readonly buffercell_getUnderlineStyle: (a: number) => [number, number, number];
  readonly buffercell_getUnderlineColor: (a: number) => [number, number, number];
  readonly buffercell_getUnderlineColorMode: (a: number) => [number, number, number];
  readonly buffercell_isUnderlineColorRGB: (a: number) => [number, number, number];
  readonly buffercell_isUnderlineColorPalette: (a: number) => [number, number, number];
  readonly buffercell_isUnderlineColorDefault: (a: number) => [number, number, number];
  readonly buffercell_attributesEquals: (a: number, b: any) => [number, number, number];
  readonly __wbg_buffernamespace_free: (a: number, b: number) => void;
  readonly buffernamespace_normal: (a: number) => any;
  readonly buffernamespace_alternate: (a: number) => any;
  readonly buffernamespace_active: (a: number) => [number, number, number];
  readonly buffernamespace_onBufferChange: (a: number) => any;
  readonly __wbg_bufferview_free: (a: number, b: number) => void;
  readonly bufferview_type: (a: number) => [number, number];
  readonly bufferview_cursorY: (a: number) => [number, number, number];
  readonly bufferview_cursorX: (a: number) => [number, number, number];
  readonly bufferview_viewportY: (a: number) => [number, number, number];
  readonly bufferview_baseY: (a: number) => [number, number, number];
  readonly bufferview_length: (a: number) => [number, number, number];
  readonly bufferview_getNullCell: (a: number) => [number, number, number];
  readonly bufferview_getLine: (a: number, b: number) => [number, number, number];
  readonly __wbg_bufferline_free: (a: number, b: number) => void;
  readonly bufferline__data: (a: number) => [number, number, number];
  readonly bufferline__combined: (a: number) => [number, number, number];
  readonly bufferline__extendedAttrs: (a: number) => [number, number, number];
  readonly bufferline__isWrapped: (a: number) => [number, number, number];
  readonly bufferline_isWrapped: (a: number) => [number, number, number];
  readonly bufferline_length: (a: number) => [number, number, number];
  readonly bufferline_getCell: (a: number, b: number, c: any) => [number, number, number];
  readonly bufferline_translateToString: (a: number, b: number, c: number, d: number, e: number, f: number) => [number, number, number];
  readonly __wbg_buffercell_free: (a: number, b: number) => void;
  readonly buffercell_getChars: (a: number) => any;
  readonly buffercell_getCode: (a: number) => [number, number, number];
  readonly buffercell_getWidth: (a: number) => number;
  readonly buffercell_getFgColorMode: (a: number) => number;
  readonly buffercell_getBgColorMode: (a: number) => number;
  readonly buffercell_getFgColor: (a: number) => number;
  readonly buffercell_getBgColor: (a: number) => number;
  readonly buffercell_isBold: (a: number) => number;
  readonly buffercell_isItalic: (a: number) => number;
  readonly buffercell_isDim: (a: number) => number;
  readonly buffercell_isBlink: (a: number) => number;
  readonly buffercell_isInverse: (a: number) => number;
  readonly buffercell_isInvisible: (a: number) => number;
  readonly buffercell_isStrikethrough: (a: number) => number;
  readonly buffercell_isOverline: (a: number) => number;
  readonly buffercell_isFgRGB: (a: number) => number;
  readonly buffercell_isFgPalette: (a: number) => number;
  readonly buffercell_isFgDefault: (a: number) => number;
  readonly buffercell_isBgRGB: (a: number) => number;
  readonly buffercell_isBgPalette: (a: number) => number;
  readonly buffercell_isBgDefault: (a: number) => number;
  readonly buffercell_isAttributeDefault: (a: number) => number;
  readonly buffercell_content: (a: number) => number;
  readonly buffercell_set_content: (a: number, b: number) => void;
  readonly buffercell_fg: (a: number) => number;
  readonly buffercell_set_fg: (a: number, b: number) => void;
  readonly buffercell_bg: (a: number) => number;
  readonly buffercell_set_bg: (a: number, b: number) => void;
  readonly buffercell_combinedData: (a: number) => any;
  readonly buffercell_set_combinedData: (a: number, b: number) => void;
  readonly buffercell_extended: (a: number) => any;
  readonly buffercell_set_extended: (a: number, b: any) => void;
  readonly __wbg_extendedcellattributes_free: (a: number, b: number) => void;
  readonly extendedcellattributes__ext: (a: number) => number;
  readonly extendedcellattributes_set__ext: (a: number, b: number) => void;
  readonly extendedcellattributes__urlId: (a: number) => number;
  readonly extendedcellattributes_set__urlId: (a: number, b: number) => [number, number];
  readonly extendedcellattributes_ext: (a: number) => number;
  readonly extendedcellattributes_set_ext: (a: number, b: number) => void;
  readonly extendedcellattributes_urlId: (a: number) => number;
  readonly extendedcellattributes_set_urlId: (a: number, b: number) => [number, number];
  readonly extendedcellattributes_underlineStyle: (a: number) => number;
  readonly extendedcellattributes_set_underlineStyle: (a: number, b: number) => void;
  readonly extendedcellattributes_underlineColor: (a: number) => number;
  readonly extendedcellattributes_set_underlineColor: (a: number, b: number) => void;
  readonly extendedcellattributes_underlineVariantOffset: (a: number) => number;
  readonly extendedcellattributes_set_underlineVariantOffset: (a: number, b: number) => void;
  readonly extendedcellattributes_payload: (a: number) => [number, number, number];
  readonly extendedcellattributes_set_payload: (a: number, b: any) => void;
  readonly extendedcellattributes_isEmpty: (a: number) => number;
  readonly extendedcellattributes_clone: (a: number) => [number, number, number];
  readonly __wbg_parserapi_free: (a: number, b: number) => void;
  readonly parserapi_registerCsiHandler: (a: number, b: any, c: any) => [number, number, number];
  readonly parserapi_addCsiHandler: (a: number, b: any, c: any) => [number, number, number];
  readonly parserapi_registerDcsHandler: (a: number, b: any, c: any) => [number, number, number];
  readonly parserapi_addDcsHandler: (a: number, b: any, c: any) => [number, number, number];
  readonly parserapi_registerEscHandler: (a: number, b: any, c: any) => [number, number, number];
  readonly parserapi_addEscHandler: (a: number, b: any, c: any) => [number, number, number];
  readonly parserapi_registerOscHandler: (a: number, b: number, c: any) => [number, number, number];
  readonly parserapi_addOscHandler: (a: number, b: number, c: any) => [number, number, number];
  readonly parserapi_registerApcHandler: (a: number, b: any, c: any) => [number, number, number];
  readonly __wbg_unicodeapi_free: (a: number, b: number) => void;
  readonly unicodeapi_register: (a: number, b: any) => [number, number];
  readonly unicodeapi_versions: (a: number) => any;
  readonly unicodeapi_activeVersion: (a: number) => any;
  readonly unicodeapi_set_activeVersion: (a: number, b: any) => [number, number];
  readonly __wbg_unicodeservice_free: (a: number, b: number) => void;
  readonly unicodeservice_getStringCellWidth: (a: number, b: any) => [number, number, number];
  readonly __wbg_wasmbindgentestcontext_free: (a: number, b: number) => void;
  readonly wasmbindgentestcontext_new: () => number;
  readonly wasmbindgentestcontext_include_ignored: (a: number, b: number) => void;
  readonly wasmbindgentestcontext_filtered_count: (a: number, b: number) => void;
  readonly wasmbindgentestcontext_run: (a: number, b: number, c: number) => any;
  readonly __wbgtest_console_log: (a: any) => void;
  readonly __wbgtest_console_debug: (a: any) => void;
  readonly __wbgtest_console_info: (a: any) => void;
  readonly __wbgtest_console_warn: (a: any) => void;
  readonly __wbgtest_console_error: (a: any) => void;
  readonly __wbgtest_cov_dump: () => [number, number];
  readonly __externref_table_alloc: () => number;
  readonly __wbindgen_export_1: WebAssembly.Table;
  readonly __wbindgen_malloc: (a: number, b: number) => number;
  readonly __wbindgen_realloc: (a: number, b: number, c: number, d: number) => number;
  readonly __wbindgen_exn_store: (a: number) => void;
  readonly __wbindgen_export_5: WebAssembly.Table;
  readonly __externref_table_dealloc: (a: number) => void;
  readonly __wbindgen_free: (a: number, b: number, c: number) => void;
  readonly closure968_externref_shim: (a: number, b: number) => any;
  readonly closure995_externref_shim_multivalue_shim: (a: number, b: number, c: any) => [number, number];
  readonly closure991_externref_shim: (a: number, b: number, c: any) => void;
  readonly closure1005_externref_shim_multivalue_shim: (a: number, b: number, c: any) => [number, number, number];
  readonly _dyn_core__ops__function__Fn_____Output___R_as_wasm_bindgen__closure__WasmClosure___describe__invoke__hcd0dcfec4fbab121_multivalue_shim: (a: number, b: number) => [number, number];
  readonly _dyn_core__ops__function__FnMut__A____Output___R_as_wasm_bindgen__closure__WasmClosure___describe__invoke__h6f4174b98aaf8323: (a: number, b: number, c: number) => void;
  readonly _dyn_core__ops__function__FnMut__A____Output___R_as_wasm_bindgen__closure__WasmClosure___describe__invoke__hb91f40168af055a9: (a: number, b: number, c: number) => void;
  readonly _dyn_core__ops__function__Fn_____Output___R_as_wasm_bindgen__closure__WasmClosure___describe__invoke__hadcafc84a18506cc_multivalue_shim: (a: number, b: number) => [number, number, number];
  readonly closure1001_externref_shim_multivalue_shim: (a: number, b: number, c: any, d: any) => [number, number];
  readonly closure999_externref_shim_multivalue_shim: (a: number, b: number, c: any, d: any) => [number, number];
  readonly _dyn_core__ops__function__FnMut_____Output___R_as_wasm_bindgen__closure__WasmClosure___describe__invoke__hdb88ec8af928c1f1: (a: number, b: number) => void;
  readonly closure993_externref_shim: (a: number, b: number, c: any) => void;
  readonly closure1013_externref_shim: (a: number, b: number, c: any, d: any) => number;
  readonly _dyn_core__ops__function__Fn__A____Output___R_as_wasm_bindgen__closure__WasmClosure___describe__invoke__hcaae87c30ffd7e1f_multivalue_shim: (a: number, b: number, c: number) => [number, number];
  readonly closure989_externref_shim_multivalue_shim: (a: number, b: number, c: any) => [number, number];
  readonly closure1009_externref_shim: (a: number, b: number, c: number) => any;
  readonly _dyn_core__ops__function__FnMut__A____Output___R_as_wasm_bindgen__closure__WasmClosure___describe__invoke__h4bd2fa9f02bba4d0_multivalue_shim: (a: number, b: number, c: number) => [number, number];
  readonly closure1851_externref_shim_multivalue_shim: (a: number, b: number, c: any) => [number, number, number];
  readonly _dyn_core__ops__function__FnMut_____Output___R_as_wasm_bindgen__closure__WasmClosure___describe__invoke__h6ba1f6b7b7654eee_multivalue_shim: (a: number, b: number) => [number, number];
  readonly closure1853_externref_shim_multivalue_shim: (a: number, b: number, c: any) => [number, number];
  readonly _dyn_core__ops__function__Fn_____Output___R_as_wasm_bindgen__closure__WasmClosure___describe__invoke__h6bb8594aa947c132_multivalue_shim: (a: number, b: number) => [number, number, number];
  readonly _dyn_core__ops__function__Fn__A____Output___R_as_wasm_bindgen__closure__WasmClosure___describe__invoke__hcf60c50c026fa841_multivalue_shim: (a: number, b: number, c: number, d: number) => [number, number];
  readonly _dyn_core__ops__function__Fn_____Output___R_as_wasm_bindgen__closure__WasmClosure___describe__invoke__haabb29f2b0594031: (a: number, b: number) => void;
  readonly closure2115_externref_shim_multivalue_shim: (a: number, b: number, c: any) => [number, number];
  readonly closure2117_externref_shim_multivalue_shim: (a: number, b: number, c: any) => [number, number, number];
  readonly closure2111_externref_shim_multivalue_shim: (a: number, b: number, c: any) => [number, number, number];
  readonly closure2121_externref_shim_multivalue_shim: (a: number, b: number, c: number, d: number, e: any) => [number, number, number];
  readonly closure2119_externref_shim_multivalue_shim: (a: number, b: number, c: any, d: any) => [number, number];
  readonly closure2123_externref_shim_multivalue_shim: (a: number, b: number, c: any, d: any, e: any) => [number, number, number];
  readonly closure2915_externref_shim: (a: number, b: number, c: any) => void;
  readonly wasm_bindgen__convert__closures__invoke0_mut__h6ccbabcefd85a5b8: (a: number, b: number) => void;
  readonly closure2946_externref_shim: (a: number, b: number, c: any, d: number, e: any) => void;
  readonly closure2938_externref_shim: (a: number, b: number, c: any, d: any) => void;
  readonly __wbindgen_start: () => void;
}

export type SyncInitInput = BufferSource | WebAssembly.Module;
/**
* Instantiates the given `module`, which can either be bytes or
* a precompiled `WebAssembly.Module`.
*
* @param {{ module: SyncInitInput }} module - Passing `SyncInitInput` directly is deprecated.
*
* @returns {InitOutput}
*/
export function initSync(module: { module: SyncInitInput } | SyncInitInput): InitOutput;

/**
* If `module_or_path` is {RequestInfo} or {URL}, makes a request and
* for everything else, calls `WebAssembly.instantiate` directly.
*
* @param {{ module_or_path: InitInput | Promise<InitInput> }} module_or_path - Passing `InitInput` directly is deprecated.
*
* @returns {Promise<InitOutput>}
*/
export default function __wbg_init (module_or_path?: { module_or_path: InitInput | Promise<InitInput> } | InitInput | Promise<InitInput>): Promise<InitOutput>;
