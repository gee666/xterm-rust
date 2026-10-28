# Mouse state acceptance specification

This is the acceptance contract for the platform-neutral port of `source_repo/src/common/services/MouseStateService.ts`. It does not stand in for browser event capture, coordinate conversion, DOM wheel events, terminal mouse-mode parsing, or outbound transport. Those require separate integration criteria. The source's own `triggerMouseEvent` description is stale: no such method exists in the class. Do not invent it or merge restriction and encoding.

## Protocol restrictions

Run every defined button `0,1,2,3,4,8..15`, action `0,1,2,3,32`, and all eight modifier masks against each protocol. Compare the accepted flag and the complete mutated event, not just the encoded result.

- `NONE` advertises mask 0 and rejects every event without mutating any field.
- `X10` advertises DOWN mask 1. It rejects WHEEL and any action other than DOWN, preserving modifiers in rejected cases. An accepted event clears ctrl, alt and shift to explicit false, including previously undefined modifiers. It leaves coordinates, button and action unchanged. Auxiliary buttons with DOWN remain accepted.
- `VT200` advertises DOWN|UP|WHEEL mask 19. It rejects only MOVE, including MOVE with WHEEL, and does not alter fields.
- `DRAG` advertises DOWN|UP|WHEEL|DRAG mask 23. It rejects only MOVE with NONE button and otherwise preserves all fields. Do not reject extra combinations merely because the interface comment calls some combinations invalid.
- `ANY` advertises all five bits, mask 31, accepts every typed combination and leaves fields unchanged.
- Optional ctrl/alt/shift values must preserve the distinction between undefined and false for custom callbacks. Built-in encoders treat either as false.
- `restrictMouseEvent` calls only the active restriction. It does not validate coordinates, encode, emit output, or apply hidden filtering.

Required checks are `tests/mouse_differential.rs` across the whole source corpus and targeted optional-modifier tests in `tests/mouse_state.rs`.

## Encoders and JavaScript numeric behavior

Call `encodeMouseEvent` independently, including on events rejected by restriction.

- The event code ORs CTRL 16, SHIFT 4, ALT 8 when truthy. For WHEEL, OR 64 and the action value. For other buttons, OR the low two button bits, 64 when button bit 4 is set, and 128 when button bit 8 is set.
- MOVE adds 32 for non-wheel events. Non-SGR UP ORs NONE value 3 without clearing other button bits. SGR UP retains the button identity. Wheel UP does not use the release terminator.
- DEFAULT starts with exact UTF-16 units `1b,5b,4d`, followed by event code+32, col+32, row+32 using `String.fromCharCode` semantics.
- DEFAULT returns an empty string only if any biased parameter is greater than 255. Test col and row independently at 223, 223.5 and 224; both axes must enforce the same upper limit.
- DEFAULT has no lower-bound rejection. Negative values wrap through ToUint16. Fractional values truncate after adding the bias. NaN and negative infinity become zero when they are not suppressed by another coordinate; positive infinity suppresses the report. Test these without assuming nonnegative browser inputs.
- Preserve lone surrogate code units, including `-10272 + 32 -> 0xd800` and `-8225 + 32 -> 0xdfff`. A Rust UTF-8 `String` cannot represent all outputs; expose `Vec<u16>` and document the binding conversion.
- SGR is exactly `ESC[<code;col;rowM` or final `m` for UP on a non-wheel button. SGR_PIXELS uses x/y instead. Both have no coordinate limit and must retain JS formatting for NaN, infinities, fractions, large exponents, and negative zero. There is no coordinate +1 conversion in this service.
- Every defined button/action and all modifier masks must match exact UTF-16 output in each encoder, including combinations not produced by a normal browser.
- Custom encoders run exactly once, receive the original mutable event, may alter any representable field, return empty output or lone surrogates, and replace existing names without an activation change. The caller observes mutations after encoding.

The checked-in corpus must include all 7,800 defined protocol/encoding/button/action/modifier combinations, numeric boundary rows, and 4,096 xorshift32 inputs at seed `0x6d0a5e`. After adding surrogate boundaries it has 12,226 rows. Never edit expected units manually.

## Registry and activation

- Construction installs protocols in source order NONE, X10, VT200, DRAG, ANY and encodings DEFAULT, SGR, SGR_PIXELS. Active values are NONE and DEFAULT. Mouse-events-active, default-encoding, pixel-encoding getters use the active registered values/names.
- Setting a known protocol stores the active name before firing its advertised mask. Reassigning the same protocol still emits. Setting an encoding emits no protocol-change event.
- Unknown protocol/encoding names return errors with exact text `unknown protocol "name"` or `unknown encoding "name"`. A rejected setter must leave both previous selections and event history unchanged.
- `addProtocol` retains a mutable protocol object. External edits to its events or restriction affect every service retaining that same object. Rust's `add_shared_protocol` exposes that identity through `Rc<SharedMouseProtocol>` and setters; callbacks snapshot the current function before invoking it. `addProtocol` and `addEncoding` may overwrite defaults or active registrations without emitting activation events. Existing selected names then use the replacement behavior. Custom names can be selected normally.
- Reentrant callbacks must not run while Rust holds a RefCell borrow or a lock. A protocol-change listener can inspect state and select another protocol. A custom restriction or encoder can replace its own registration; the current call finishes using its captured callback and the next call observes the replacement.
- `reset` selects NONE first, emits its current advertised mask, then selects DEFAULT. It preserves custom registrations and wheel hooks, including overwritten NONE/DEFAULT entries.
- A dispatch error propagates with its original concrete cause available through the Rust error source chain, not a fabricated listener-array error. Replaced callback captures drop outside service borrows so Rust cleanup can reenter safely.
- A dispatch error propagates. Do not report a successful reset if a protocol listener interrupted it before the encoding reset step.

## Wheel hook and lifecycle

- With no hook, `allowCustomWheelEvent` returns true. A hook receives the caller's original event object by reference and its false result vetoes; true allows. Replacing or clearing the hook takes effect on the next call. Invoke at most once per call.
- Keep the wheel-event payload generic so browser bindings can pass the actual WheelEvent without making the terminal core browser-dependent.
- Disposal is idempotent and stops protocol-change delivery. Subscribing afterward must not deliver. Source disposal does not erase registered protocols, encodings or the wheel hook, or prohibit subsequent state setters, restrictions and encodings. Test continued state operations after disposal.
- Disposal during nested protocol-change dispatch uses the shared Event implementation's error semantics. Expose errors rather than discarding them.
- Avoid Rc cycles. Reentrancy tests capture weak service references; custom callbacks retain only explicitly owned caller captures. Dispose on Rust drop is an ownership-boundary adaptation, not an extra claimed source method.

## Upstream assertion mapping and review gate

Keep all eight original source cases unchanged in the baseline:

| Source case | Required Rust assertion coverage |
| --- | --- |
| init | NONE and DEFAULT active defaults |
| default protocols | All five names in source order |
| default encodings | DEFAULT, SGR, SGR_PIXELS in source order |
| setters/reset | Both changes, both reset defaults, both exact unknown-name errors |
| addEncoding | A custom encoder can activate |
| addProtocol | A custom protocol can activate |
| onProtocolChange | NONE emits 0, ANY emits 31 in order |
| restrict/encode | ANY accepts and DEFAULT output is `1b,5b,4d,20,21,21` |

Acceptance requires the original 8/8 baseline, faithful Rust mapping, all 12,226 differential rows, the targeted registry/wheel/lifecycle/reentrancy cases above, and independent source-and-test review. Record executed/passed/failed/skipped/unrun counts and source hashes, retain a reproducible command and seed, and save any mismatch as a counterexample rather than weakening a fixture.

Run `node scripts/mouse-state-oracle.mjs`, `cargo test -p xterm-core --test mouse_state --test mouse_differential`, targeted rustfmt and Clippy, plus workspace gates at integration. Results belong under `results/mouse-state/`. Native Windows execution is evidence only for Windows; no Linux/macOS claim follows from it. Terminal equivalence remains NOT PROVEN while other required subsystems or platforms are missing.
