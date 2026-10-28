# Mouse state service

The platform-independent port is `xterm_core::mouse_state::MouseStateService<W>`. Its acceptance contract is [acceptance/mouse-state.md](acceptance/mouse-state.md). `mouse_protocol.rs` owns protocol data and encoding algorithms; `mouse_state.rs` owns registration, activation, callbacks and lifetime. Both compile as one public module.

## API mapping

| JavaScript | Rust |
| --- | --- |
| constructor | `MouseStateService::<W>::new()` |
| activeProtocol getter/setter | `active_protocol`, `set_active_protocol` |
| activeEncoding getter/setter | `active_encoding`, `set_active_encoding` |
| addProtocol/addEncoding | `add_protocol`, `add_encoding` |
| areMouseEventsActive | `are_mouse_events_active` |
| isDefaultEncoding/isPixelEncoding | `is_default_encoding`, `is_pixel_encoding` |
| onProtocolChange | `on_protocol_change().subscribe(...)` |
| restrictMouseEvent | `restrict_mouse_event(&mut MouseEvent)` |
| encodeMouseEvent | `encode_mouse_event(&mut MouseEvent) -> Vec<u16>` |
| setCustomWheelEventHandler | `set_custom_wheel_event_handler` |
| allowCustomWheelEvent | `allow_custom_wheel_event(&W)` |
| reset/dispose | `reset`, `Disposable::dispose` |

Unknown selection errors use `MouseStateError` with the original messages. Dispatch errors from the shared event emitter propagate. Coordinates are `f64` because the source accepts fractions, NaN, infinities and negative values. Typed button and action enums retain source numeric values. `MouseButton::Raw(i32)` also represents signed DOM button values such as -1; equality uses numeric value so Raw(4) has the same wheel semantics as the named variant. Actions reject out-of-enum values using `TryFrom<u8>`. Modifier fields are `Option<bool>` so custom protocols can observe JavaScript `undefined`. `MouseEventTypes` holds advertised event bits. Custom callbacks use `Rc` and run outside service borrows; callers should capture weak service references when reentering. `add_shared_protocol` retains an `Rc<SharedMouseProtocol>` whose events and restriction can be changed through setters and observed by multiple services. The owned `add_protocol` adapter transfers one protocol object into the service. Encoders receive mutable events because the source custom encoder can mutate its argument.

Encoder output uses UTF-16 units, not Rust `String`. DEFAULT can produce lone surrogates from negative coordinates. Its behavior follows `String.fromCharCode`, not UTF-8 byte encoding or replacement-character conversion. SGR number formatting uses the workspace's existing `ryu-js` dependency. Restriction and encoding remain separate calls, even for rejected events.

The wheel payload type `W` belongs to the adapter. It can be a browser WheelEvent without importing DOM types into the core. Explicit disposal stops notifications but leaves registrations, setters, restrictions, encoders and wheel handlers usable, matching source cleanup. Rust drop also disposes the emitter.

## Reproducible scoped checks

With Node.js 24.15 and Rust 1.85.1 available on PATH:

```sh
node scripts/check-mouse-state.mjs
```

The checker first verifies the tracked corpus against unchanged source and original assertions. It then checks formatting, runs three Rust suites in debug and release, and runs warning-denying Clippy. It writes every command, result, tool version, source/implementation hash and raw log under `results/mouse-state/`. It refuses commands below 512 MiB available disk space and uses the shared build directory/profiles. It never changes expected fixtures during verification.

To intentionally regenerate source expectations:

```sh
node scripts/mouse-state-oracle.mjs --generate
```

The eight original test bodies run unchanged through TypeScript transformation and Node assertions. A Chai-compatible `throws` adapter preserves substring message checks. The corpus has 12,226 rows, including all 7,800 defined protocol/encoding/button/action/modifier combinations, 330 boundary rows and 4,096 seeded rows at `0x6d0a5e`. Rows compare restriction results, modifier mutations, coordinate bits and exact encoding units. The surrogate boundaries were added after independent reviewer Tuah found that an earlier negative-coordinate value did not actually produce a surrogate.

`tests/mouse_state.rs` maps all eight source cases and adds six targeted contracts. `tests/mouse_callbacks.rs` adds five mask, reentrancy and partial-error-state cases. `tests/mouse_differential.rs`, independently written by Tuah, compares every corpus row. A second independently generated corpus adds 4,200 raw signed-button traces from unchanged source. The two differential tests consume 16,426 rows. Five custom mutation/identity/error/drop regressions and four source-derived custom assertions were added after independent review found those contracts missing. This totals 26 Rust tests per profile, with no skipped or ignored mouse cases. See the machine-readable report for the latest actual pass/fail state rather than inferring success from these commands.

## Limits of this evidence

Dynamic inherited/prototype registry names remain a separate implementation gap in [mouse-registry-gaps.md](mouse-registry-gaps.md), tracked by task171. The typed service and corpus do not prove parity for every string name or inherited encoder's non-string returns. The latest custom mutable/shared-object fixes require their new26-test receipt before acceptance; older21-test receipts are preserved historical evidence.

Observed execution is native Windows only. No Linux/macOS runtime result is claimed. These scoped checks do not establish browser event capture, coordinate conversion, DEC mode parser integration, transport delivery or end-to-end mouse interaction. Those require the browser/core integration tasks and full workspace checks. The report intentionally remains failed when another shared module prevents a build or Clippy run. Full terminal equivalence is NOT PROVEN.
