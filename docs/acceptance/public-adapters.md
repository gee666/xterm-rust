# Common public adapters acceptance

Owner: Charulata, task 66. This specification is a completion gate, not evidence that the port passes. Overall equivalence remains NOT PROVEN.

Reference files are `source_repo/src/common/public/{AddonManager,BufferApiView,BufferNamespaceApi,ParserApi}.ts`, `AddonManager.test.ts`, `source_repo/typings/xterm.d.ts` interfaces ITerminalAddon, IBuffer, IBufferNamespace, IBufferLine, IBufferCell, IFunctionIdentifier and IParser. UnicodeApi belongs to task 23. Source behavior controls where declaration comments are stale, including APC support. Keep original assertions intact.

## A1. Addon activation and ownership

- Load records the instance and its original disposal callback, installs the managed disposal route, then calls activation exactly once with the actual public terminal object. Activation sees the same terminal options, dimensions and APIs as its caller.
- Port the two AddonManager.test.ts cases without dropping assertions: activation receives `foo` and is called; three registrations exist before disposal, all three callbacks run, no registrations remain.
- Port the headless/public/Terminal.test.ts and test/playwright/Terminal.test.ts loadAddon groups: activation reads cols=5; direct addon disposal and terminal disposal each invoke cleanup, never before the disposal call.
- Activation is fallible. Registration and wrapped disposal already exist when activation fails; do not silently undo registration. Report the failure and retain the record for later cleanup as upstream does.
- The Rust representation must name the managed handle replacing JavaScript instance method replacement. Do not imply an unmanaged concrete addon's raw dispose method removes a registration.

## A2. Addon disposal, errors and reentrancy

- Manager disposal calls registrations in reverse loading order. Direct disposal marks a registration disposed before the callback, invokes the original callback on the same instance, then removes its record only after successful return.
- Repeated handle disposal, disposal inside its own callback, and manager disposal after successful direct removal do not call that registration twice. Repeated manager disposal after complete cleanup has no callbacks.
- A cleanup error stops the manager traversal and propagates. The failing entry remains recorded but marked disposed. Later retries do not rerun it. Other earlier records remain available. Test this precise partial state rather than clearing everything in a finally block.
- Callbacks must run without a RefCell mutable borrow or mutex held. Test callback-triggered loading, disposal of another addon, and manager reentry. Compare traversal behavior with source, including invalidated indices; report source errors rather than hiding them with a success return.
- Test separate manager identity, duplicate loading of the same instance and nested wrappers. If Rust forbids a JavaScript aliasing operation, record the binding behavior and test it at the browser boundary. Do not silently call this full compatibility.
- Explicit disposal remains observable. Rust Drop must not cause duplicate cleanup, leak a cycle or turn a recoverable cleanup failure into an unwinding destructor panic. Document what callers must do to observe errors.

## B1. Live buffer views and bounds

- BufferApiView exposes exactly type, cursorY, cursorX, viewportY, baseY, length, getLine and getNullCell. Normal/alternate type is immutable. All numeric getters reflect the underlying live buffer with no divergent snapshot state.
- cursorX includes the legal after-last-column value cols. cursorY is viewport-relative, viewportY is ydisp and baseY is ybase. Validate these independently after writes, scrolling, resize and alternate activation.
- `init` replaces the backing buffer and returns the same view identity. Namespace normal/alternate getters refresh it after BufferSet reset; old line references are not a promise of stable data after mutation.
- getLine returns the actual line for 0 and length-1; length, empty buffer, negative, fractional, NaN, infinities and huge JS indices follow the source behavior at the browser binding. The source directly calls CircularList.get with no bounds check, so capacity-wrapped indices and stale backing slots may return lines outside logical length despite the typings comment. Preserve and test that source quirk. Native unsigned indices have a narrower input domain, but must not add a logical-length check.
- getNullCell returns a new independent default CellData each time, with zero content/width/code, default attributes and no shared mutable extended attributes. Test use as a reusable getCell destination.

## B2. Public line and cell contracts

- Do not clone a line and claim it is a live view. Safe Rust borrows may prevent writes while a line is borrowed; document that lifetime representation and verify browser bindings remain usable after writes.
- Map every IBufferLine and IBufferCell method to the existing buffer implementation and test every binding. Include getCell destination reuse and identity, undefined/None outside length, physical length versus terminal cols after resize, and isWrapped.
- translateToString uses inclusive start/exclusive end and trimRight behavior. Cover blank/null cells, ordinary spaces, CJK width-2 and trailing width-0 cells, combining strings, astral scalars, lone UTF-16 units, partial wide-cell ranges, empty/reversed ranges and default arguments. Preserve UTF-16 at browser boundaries.
- Compare all public cell attributes and colors: width/chars/code, fg/bg modes and values, all style bit-returning predicates, RGB/palette/default predicates, default-attribute test, underline style/color/mode and attribute equality. Equality excludes content, URL ids and underline variant offsets. Bit-returning JS methods must not silently become booleans in JS bindings.

## B3. Buffer namespace identity and events

- Namespace constructs one normal and one alternate view and subscribes once to BufferSet activation. Getter calls reuse identities; active is identical to the matching normal/alternate view.
- Every core activation event emits exactly the now-active public view synchronously, including BufferSet reset to new objects. Same-buffer activation suppression belongs to BufferSet; do not introduce extra suppression in the namespace.
- At event time, cursor, active buffer and public getter results agree. Test normal to alternate to normal, repeated activation, reset while alternate, resize, retained getter handles and listener removal during dispatch.
- A corrupted active identity must produce the explicit `Active buffer is neither normal nor alternate` failure, not choose normal. A Rust exhaustive enum can make this invalid state unrepresentable; document the mapping.
- Disposal removes the core subscription and disposes the outgoing event. Later core activation emits no namespace event. Repeated disposal is safe; dropped subscriptions must not retain a terminal cycle.

## P1. Parser methods and parameter conversion

- Implement registerCsiHandler, registerDcsHandler, registerEscHandler, registerOscHandler and registerApcHandler. Preserve deprecated addCsiHandler/addDcsHandler/addEscHandler/addOscHandler aliases as exact delegation with no second registration. There is no addApcHandler in the source.
- CSI/DCS callbacks receive a fresh `Params.toArray()` snapshot per invocation. Cover empty/default zero, several parameters, colon subparameters including -1 defaults and nested arrays, int32 clamping and capacity truncation. Retaining or mutating a prior public array cannot alter parser state or later calls.
- DCS callback arguments are payload then converted params. ESC has no arguments. OSC/APC receive only payload; APC excludes identifier. Preserve string data as UTF-16 rather than lossy replacement.
- Identifier validation and errors must match the actual core: absent/empty/valid/invalid prefix, zero/one/two/three intermediates, each accepted final boundary and adjacent rejected byte, multibyte and surrogate strings. CSI/DCS final range is 0x40..0x7e, ESC/APC 0x30..0x7e. Check APC prefix handling against the source rather than stale comments. Test distinct prefix/intermediate/final registrations do not collide.
- OSC numeric identifiers must cover zero, large, negative, fractional, NaN/infinity at the JS boundary. Document the native identifier domain and do not use unchecked narrowing to pretend it accepts all JS numbers.

## P2. Handler order, removal and asynchronous flow

- Registration returns an independently disposable handle removing only that callback. Newest handler runs first; false continues backward, true prevents older handlers and defaults. Unrelated ids remain registered. Dispose twice and dispatch afterward; no duplicate removal or invocation.
- Port every assertion in test/playwright/Parser.test.ts, including CSI parameter snapshots, CSI/DCS/ESC/OSC/APC true/false ordering, async false continuation, APC identifier removal, short payload and independent identifiers. Record exact discovered and executed case counts.
- Pending handlers stop subsequent input processing without replaying the pending sequence or consuming later input. Resolve false to continue older handlers, true to skip them. Callback parameter/data snapshots survive intervening resize and buffer changes.
- Cover cancellation/reset/disposal during pending work, rejection or Rust error, listener self-removal/other-removal/registration during dispatch, unknown ids and parser fallback behavior. Any core unsupported case stays a named gap, not a passing adapter no-op.

## P3. Streaming strings and terminal integration

- Real terminal writes must exercise public registrations through UTF-8 and UTF-16 entry points, not direct callback invocation alone. Split every representative sequence at every byte/unit boundary, including ESC/ST, astral pairs, nested parameter separators and empty chunks.
- DCS/OSC/APC invoke completed payload callbacks once, only on successful termination. Cover BEL where applicable, ST as ESC-backslash and C1, CAN/SUB abort, malformed headers, reset/disposal mid-payload and UTF-16 payload limits at limit-1/limit/limit+1. Verify following display text and parser state recover exactly.
- Public terminal proposed-API gates, lazy adapter getters, addon loading and terminal disposal must use these implementations in both headless and browser/WASM paths. A detached helper or JS-only passing test does not satisfy this section.

## Proof and ownership rules

Task 66 owns the four adapters and focused tests; tasks 27/28/30/31/23 and browser binding owners supply live core integration, parser semantics and Unicode support. Tests must use those real components. Separate implementation and integration tasks if a dependency is incomplete, and do not claim completion from a fake core.

For each A/B/P section, record source lines, Rust API names, faithful assertion locations, discovered/mapped/executed/passed/failed/skipped/unrun counts, raw result paths and retained differential counterexamples. Run native focused tests, workspace tests, doctests, fmt and warning-free Clippy. Run the original browser groups and the same groups against Rust-backed Chromium, Firefox and WebKit, plus native Windows/Linux/macOS as required globally. Await independent code and evidence review. Missing platform execution is unrun, not a pass or a cross-compile result.
