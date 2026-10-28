# Headless declarations and runtime bindings

Scope is exactly `source_repo/typings/xterm-headless.d.ts`, 1,485 lines in the supplied tree. The existing AST inventory contains 241 declaration/member records. This map assigns focused acceptance ownership, not an implementation pass. `docs/headless-declaration-rules.json` supplies per-owner matching rules to inventory task 22. No other declaration file or browser Types module is covered by inference.

| Declaration anchor | Focused criteria | Runtime/native binding and remaining checks |
| --- | --- | --- |
| LogLevel 14, ILogger 394 | 23 | `core::log_service::{LogLevel,LogSink,LogRecord}` and WASM logger. Compare all five methods, levels, argument/error forwarding and callback failures. ILogger's variadic any values need host evidence. |
| ITerminalOptions 19 | 14,134,135,232 | `OptionsService`, `headless::PublicOptions`, browser OptionsBinding. All 32 declared members must retain defaults, optionality, mutability and reference comparisons. Rendering-only option comments do not invent headless rendering. Real renderer/input side effects have their separate owners. |
| ITerminalInitOnlyOptions 263 | 232 | cols/rows constructor-only public assignment guard, resize path and showCursorImmediately. Native constructors have explicit typed dimensions; source Number coercion is a separate binding contract. |
| ITheme 285 | 193,232 | Option storage and browser ThemeService. Declaration's legacy `selection` key differs from current source theme options. Record actual runtime behavior before translating that field; do not silently rename it to selectionBackground. |
| IWindowsPty 335 | 63,178,179 | backend/buildNumber option object, Windows wrap and resize/reflow behavior. Preserve absent and boundary build values; native Windows execution remains required. |
| IVtExtensions 349 | 132,235 | kittyKeyboard, kittySgrBoldFaintControl, win32InputMode, colorSchemeQuery. Configuration enablement is distinct from resulting modes and protocol replies. Engine/mode/browser integration criteria supply executable behavior. |
| IDisposable 425 | 161,162,164,234 | Explicit disposable handles, terminal lifecycle and callback-error ordering. Native Drop is not a substitute for observable disposal failures. |
| IEvent 433 | 221,222,223,234 | Event/Emitter, subscription handle and public synchronous event forwarding. JS second-argument/default generic representation and callback exception identity require binding evidence. |
| IMarker 442, IDisposableWithEvent 458 | 120,121,122,236 | shared Marker id/line/isDisposed/onDispose; explicit disposal and actual buffer mutation. Test disposed line=-1 and active-buffer registration against source, not stale alternate-buffer comments. |
| ILocalizableStrings 473 | 232 | `promptLabel`, `tooMuchOutput`. This is a named headless source/declaration discrepancy, not implemented headless localization: declaration Terminal.strings exists but source headless Terminal has no strings accessor. Browser localization is separate. |
| IWindowOptions 513 | 218,232 | All 22 permission flags are stored as options and guard public CSI-t hooks. Most operations intentionally lack default host implementations upstream. Report source handler behavior and security gates; do not manufacture successful OS window operations. |
| Terminal 646 | 231-236 | See member table below. Native headless Terminal and browser bindings share the Rust engine; native synchronous and explicitly driven pending writes are representation changes, not proof of JS event-loop equivalence. |
| ITerminalAddon 930 | 213,214,236 | `TerminalAddon`, AddonManager and managed handles replace JS method wrapping. Actual activation terminal identity, error state, LIFO/reentrant cleanup and terminal disposal remain required. |
| IViewportRange 940, IViewportRangePosition 955 | 119 | Data-only start/end and x/y shapes. No source headless Terminal method consumes them. Existing native range coordinates use i32; a complete exported f64-compatible shape/representation decision is still a named gap, not an exercised API. |
| IBufferRange 975, IBufferCellPosition 990 | 119 | BufferRange/BufferPosition and browser selection ranges. Native signed coordinates are narrower than arbitrary JS Number data. Preserve endpoint conventions in the consuming API; do not infer inclusive semantics from this bare interface. |
| IBuffer 1005 | 215 | BufferApiView/WebBufferView type, cursorY/X, viewportY/baseY, length, getLine/getNullCell. JS circular Number indexing may alias stale slots despite the declaration's out-of-range comment. Native unsigned and signed methods have separate domains. |
| IBufferNamespace 1063 | 217 | BufferNamespaceApi/WebBufferNamespace active/normal/alternate identity and onBufferChange, including reset/disposal. No detached snapshot may replace live views. |
| IBufferLine 1090 | 216,410,411 | BufferLine/WebBufferLine wrapped/length/getCell/translateToString. Numeric fractions/NaN/undefined, reusable targets and UTF-16 range handling have explicit task 136 tests. Hostile non-terminating source ranges have a named Rust safety difference. |
| IBufferCell 1130 | 216,410 | All 30 declared methods map to CellData/AttributeData and WebBufferCell. Numeric getters return bit masks where declared number, not booleans. Default getFgColor/getBgColor is -1 in source, contradicting the comments' zero. Source runtime wins; retain this decision explicitly. |
| IFunctionIdentifier 1279 | 218 | Prefix/intermediates/final and source validation/ordering. The APC unsupported comment is stale: source and declaration now expose APC registration. |
| IParser 1300 | 218,219,220 | All five register methods and common ParserApi/WebParserApi. Source accepts Promise<boolean> despite headless signatures saying boolean only. Deprecated add aliases are source runtime extras, not declarations to silently omit. |
| IUnicodeVersionProvider 1388 | 142,143,153,155 | version/wcwidth/charProperties mapped to Unicode provider traits and host callbacks. Preserve the full codepoint, UTF-16, packed property and callback-error contracts. |
| IUnicodeHandling 1404 | 144,154,155,232 | register/versions/activeVersion forwarding and Unicode-only proposed gate. Native version vectors are snapshots; JS provider/registry objects retain their declared observable behavior. |
| IModes 1424 | 235 | All twelve readonly projections map to ModesSnapshot and exported modes object. Exact defaults, escape transitions, mouse protocol projection, cursor inverse and snapshot independence are required. |

## Terminal members

| Declared members | Native/binding mapping | Criteria and gaps |
| --- | --- | --- |
| constructor, rows, cols, options | Terminal::new/with_options, rows/cols, PublicOptions | 231/232. Constructor Number boundaries, readonly setters, bulk partial errors and live view behavior. |
| buffer, parser, unicode | cached buffer, with_parser/ParserApi, gated live UnicodeService | 232 plus 215-220/155. Native borrowed parser facade replaces cached JS object; document it without claiming identical pointer identity. |
| markers, registerMarker | markers snapshot of shared Marker handles, register_marker/add_marker | 236. JS optional offset defaults to zero. Source supports alternate markers despite declaration comments. |
| modes | modes/ModesSnapshot | 235. Twelve fields and real protocol transitions. |
| static strings | absent in supplied headless source | 232 named discrepancy. No invented headless getter or false implementation mapping. Runtime/source-versus-declaration decision must be recorded in compatibility evidence. |
| onBell/onBinary/onCursorMove/onData/onLineFeed/onRender/onWriteParsed/onResize/onScroll/onTitleChange | corresponding on_* getters and actual Rust-backed binding callbacks | 234. Headless onRender requests rows but does not render DOM; writeParsed is batched by the scheduling contract, not necessarily once per write. |
| input/write/writeln/reset | input, UTF-16/byte write APIs, callback and queue adapters, direct reset | 233. Byte streams are UTF-8; strings preserve UTF-16. Writeln actually appends CRLF, not just the LF described by its comment. Direct reset differs from in-band RIS/DECSTR in parser/pending-input state. |
| resize | resize with Number guard at JS boundary | 231/234. Native usize dimensions do not establish the JS numeric ABI. |
| scrollLines/scrollPages/scrollToTop/scrollToBottom/scrollToLine/clear | corresponding snake_case methods | 236. Pages use source page size, signed counts and viewport clamps; clear retains the current prompt line and disposes active-buffer markers. |
| dispose/loadAddon | explicit terminal disposal and actual AddonManager | 234/236 plus 213/214. Managed addon handles are a documented native replacement for instance method rewriting. |

The 241 AST rows still need implementation/test/evidence links or explicit gaps in the compatibility index. This table and its rules are planning metadata; neither turns a missing implementation into a pass. Overall equivalence remains NOT PROVEN.
