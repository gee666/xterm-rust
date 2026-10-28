# Common declarations and test fixtures

`source_repo/src/common/Types.ts` contains 36 exported declarations. These are shared contracts, not 36 standalone runtime modules. The table identifies implementation tasks and test responsibility. A task link is planned coverage, not proof that every field or assertion passed. Detailed evidence belongs to those tasks and the generated compatibility inventory.

| Upstream declarations | Runtime owner and evidence responsibility |
| --- | --- |
| ParamsArray, IParams | #8 Params construction, storage, subparameters and borrowed views; criteria110-113 |
| IDisposable | #5 lifecycle identity, ordering, callback failures and RAII; criteria161-165 |
| ITerminalOptions, CursorStyle, CursorInactiveStyle | #25 option defaults/coercion/events, #30/#31 public validation and binding representation; criteria14,134,135,231,232 |
| XtermListener | #14 event listener callbacks and ownership, #30/#31 callback binding signatures; criteria221-223,234 |
| IKeyboardEvent, KeyboardResultType, IKeyboardResult | #19 legacy keyboard, #51 Kitty and #33 browser input; criteria13,27,85-89,209-212 |
| IScrollEvent | #27 buffer scroll and #34 viewport/selection, #30 public forwarding; criteria181,234,236 |
| ICharset | #20 charset mapping and #28 actual printing; criteria183-188 |
| IColor, IColorRGB | #4 color conversions and #32 rendering theme; criteria189-193 |
| IOscLinkData | #29 OSC link service and #35 link provider; criteria45-47 |
| IModes, IDecPrivateModes | #29 CoreService and #28 mode handlers, #30 public snapshots; criteria22,132,235 |
| IKittyKeyboardState | #51 input encoding and #28 keyboard-state/alternate-screen stack handling; criteria85-89,132,235 |
| IRowRange | #32 renderer refresh ranges and #27 buffer ranges; criteria119,198 |
| CoreMouseButton, CoreMouseAction, ICoreMouseEvent, CoreMouseEventType, ICoreMouseProtocol, CoreMouseEncoding | #55-57 mouse contracts and #33 live browser forwarding; criteria16,26,75-78 |
| IWindowOptions | #25 option representation, #28 parsing and #31 browser window protocol; criteria132,135,207 |
| ColorRequestType, ColorIndex, AllColorIndex, SpecialColorIndex, IColorReportRequest, IColorSetRequest, IColorRestoreRequest, IColorEvent | #28 OSC color parsing and events, #32 browser colors, #31 protocol replies; criteria132,193,207 |
| IInputHandler | #28 owns every method/escape dispatch, #26 parser dispatch and #66 public parser adapters; criteria131-133,218-220 |
| IParseStack | #28 write continuation and #26 parser suspension, failures and reset; criteria32-34,133 |

The internal `Enumerate` and `IntRange` aliases are compile-time constraints, represented by validated integer boundaries rather than dead runtime types. Rust does not need a mirrored `types.rs` solely to repeat these declarations. JavaScript UTF-16 strings must retain their owning contract's lossless representation. Browser-only objects must not enter the portable core.

## TestUtils.test.ts

This file exports 11 helpers/mocks, not a shipped terminal API. Reference assertions remain intact. Test adapters may supply the required fixture contract but must identify substitutions in reports. Do not claim the real service was tested merely because a mock passed.

| Export | Test owner |
| --- | --- |
| createCellData, extendedAttributes | #11 cell/attributes and #24/#52 line fixtures |
| MockBufferService | #27 buffer and #28 InputHandler; #58 Windows oracle explicitly uses a structural BufferService fixture backed by actual BufferLine |
| MockMouseStateService | #33 browser input and #55-57 mouse tests |
| MockCharsetService | #20 charset and #28 InputHandler |
| MockCoreService | #29 service and #28 InputHandler |
| MockLogService | #29 logging and each isolated test harness; #58 queue uses explicit recording Scheduler |
| MockOptionsService | #25 options, #27 buffers and #28 InputHandler |
| MockOscLinkService | #29 link service and #35 browser links |
| MockUnicodeService | #23 Unicode service, #28 printing and #41 addons |
| MockDecorationService | #29 decorations and #32 renderer |

No mock is compiled as production support merely to increase ported-module counts. Each owning suite must retain original assertions, document fixture substitutions, and test real subsystem integration separately. #22's declaration inventory is the completeness gate; this table must not automatically mark all of Types.ts behavior verified by #58.
