# Unicode service and public view

This ports `source_repo/src/common/services/UnicodeService.ts` and `source_repo/src/common/public/UnicodeApi.ts`. It is not a complete terminal or browser port. The exact acceptance branches are in [acceptance/unicode-core.md](acceptance/unicode-core.md), sections UR, UW, UE and UA. Full equivalence remains NOT PROVEN.

## Callable mapping

| Upstream | Rust |
| --- | --- |
| `UnicodeService` | `unicode_service::UnicodeService` |
| `register(provider)` | `register(Rc<dyn UnicodeVersionProvider>) -> Result` |
| `versions` | `versions() -> Vec<String>` |
| `activeVersion` getter/setter | `active_version()` / `set_active_version(&str)` |
| `onChange` | `on_change() -> Event<String>` |
| `wcwidth`, `charProperties` | `wcwidth`, `char_properties`, returning typed errors |
| `getStringCellWidth(s)` | `get_string_cell_width(&str)` or lossless `get_string_cell_width_utf16(&[u16])` |
| `UnicodeApi` | Borrowed `UnicodeApi<'_>` view over the same service |
| `dispose()` | `Disposable::dispose`, also called on native owner drop |

The service starts with no provider, matching upstream. The terminal must register UnicodeV6. Empty strings have width zero even without an active provider; nonempty width/property operations report `NoActiveProvider` until registration.

Provider identity is shared through `Rc`. Registering the same name replaces its registry entry but not the active provider object until the name is selected again. Versions follow ECMAScript Object.keys ordering, with canonical numeric array-index names first and other names in insertion order. Empty names remain valid, including the source's empty-active-name sentinel behavior.

Selection commits state before notifying listeners. No RefCell borrow remains across listener or provider callbacks, so a callback may reenter the service or change providers. Capture a weak service reference to avoid application-created cycles. Disposal clears listeners, not registered providers or active state. Event failures propagate without rolling back selection.

Both string-width overloads iterate UTF-16 units without allocating a full temporary UTF-16 string. Valid pairs combine; lone surrogates remain representable. A malformed high/non-low pair consumes both units and adds the second unit's standalone width before calculating the first unit's properties. A trailing high surrogate uses standalone width and returns immediately. These are deliberate source behaviors, not Unicode normalization. Joined-width differences use signed arithmetic; accumulation overflow is a recoverable error.

Rust codepoints are u32, provider widths are typed zero/narrow/wide values, and packed properties preserve reserved width 3. Arbitrary negative/fractional/nonfinite JavaScript numbers require binding conversion outside this typed provider API. Proposed Unicode API gating belongs to the terminal wrappers and is not established by a passing service test.

## Reproduce the checks

```
node scripts/unicode-service-oracle.mjs
cargo test -p xterm-core --test unicode_service --test unicode_service_differential
cargo test -p xterm-core --lib unicode_service::tests
cargo clippy -p xterm-core --test unicode_service --test unicode_service_differential -- -D warnings
```

The oracle requires Node 24's built-in TypeScript transformation. It evaluates the actual upstream Lifecycle, Event, UnicodeService, UnicodeV6 and service test code in a VM after resolving imports and stripping types. Its assertion adapter preserves error-message substring checks rather than weakening `throws` into a generic error check. Ordinary verification compares both committed TSV fixtures and never rewrites them; use `--generate` only for deliberate regeneration. Both fixture hashes are recorded. `node --test scripts/oracle-fixture.test.mjs` verifies missing/drifting fixtures fail without replacement, CRLF checks preserve original bytes and only explicit generation writes data. No upstream file is modified.

Current observed Windows x64 evidence uses Node v24.15.0 and rustc 1.85.1. Original service baseline is 5 discovered, 5 executed, 5 passed, 0 failed/skipped/unrun. Rust has 15 directed tests and five differential tests passing in the post-Event-migration debug run. The extra directed case verifies exact callback error identity and typed cause preservation. One private accumulation-boundary unit test also passed before the shared error-type migration. The earlier 14 plus five tests passed release; the updated release/private-unit/Clippy rerun remains separately recorded while shared exports change. Differential data includes ten deterministic plus 4,096 seeded UTF-16 inputs under both V6 and a stateful custom provider, comparing width and exact provider call order, plus 531 registry operations comparing versions, active-provider width, errors and ordered notifications. Seed is `0x71de63a1`.

`results/unicode-service/oracle.json` records source and fixture hashes and exact counts. `debug-tests.log`, `unit-overflow.log`, `clippy.log` and `fmt.log` retain the earlier passing scoped run. `post-event-final-tests.log` records the current 15 plus five debug tests; later post-event logs retain each attempted rerun and must be read for their actual status, not assumed successful from file presence. Earlier failed shared-tree compilation attempts are retained separately as `shared-compile-attempt2.log` and `shared-compile-attempt3.log`; they are not current passes. Generated TSV fixtures exceed the normal source-file line limit because each row preserves an oracle input and its expected output/call history.

These tests execute Rust service logic, not JavaScript delegation. Native Linux/macOS, actual terminal/browser bindings, proposed-API gating and full application E2E remain separate acceptance gates.
