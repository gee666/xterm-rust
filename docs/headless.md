# Headless Rust API

`xterm_core::headless::Terminal` wraps the Rust `CoreTerminal`. It does not invoke the JavaScript engine. Full xterm.js equivalence remains NOT PROVEN.

## Running checks

```sh
node scripts/headless-oracle.mjs
cargo test -p xterm-core --test headless_upstream --test public_api_integration
cargo test -p xterm-core --test public_api_validation --test headless_options --test headless_modes
cargo clippy -p xterm-core --lib --test headless_upstream --test public_api_validation --test headless_options --test headless_modes -- -D warnings
```

The JavaScript oracle needs Node 24 and the dependencies installed under `source_repo/`. It bundles unchanged headless test and implementation sources into `tmp/headless-oracle/` with esbuild and runs Mocha. No reference test assertion is edited. `results/headless/upstream.json` records all 45 original case outcomes, source dependency hashes and tool versions. This baseline is JavaScript execution, not Rust coverage.

The native port has 45 corresponding cases under `tests/headless_cases/`, joined by `tests/headless_upstream.rs`. These preserve the original expected strings, callbacks, cells, modes, events, marker disposal and normal/alternate state checks. `scripts/check-headless-mapping.mjs` independently maps case names. A case-name map alone does not prove assertion completeness. The additional `public_api_integration` tests compare event-time state with unchanged source, including a reset from a bell callback between printed characters.

Observed scoped results include 45 native cases and five event-time regressions passing in `results/review-antigone/headless-upstream-run2.log`, plus scoped Clippy in `headless-clippy-run2.log`. Later checkpoint failures are retained separately. Active changes elsewhere in the shared engine can invalidate a previously green run, so use current-tree reruns for review. These observations are native Windows only.

## API mapping

| Source API | Rust API and representation |
| --- | --- |
| Constructor with options | `Terminal::with_options(Rc<OptionsService>)`; `new(cols, rows, scrollback)` is a convenience constructor. |
| `write` string / Uint8Array | `write`, `write_utf16`, `write_bytes`. UTF-16 units retain lone surrogates. |
| `writeln` | `writeln` / `writeln_bytes`, followed by a distinct CRLF write. |
| Completion callback | `write_with_callback`, `write_bytes_with_callback`, `writeln_with_callback` return callback errors. These native convenience calls are synchronous. |
| Public options object | A stable `&PublicOptions`, sharing the real OptionsService. Clones retain the same service, not copied values. |
| Options bulk assignment | `PublicOptions::assign`, sequential and partially committed on a later error. Public writes to cols/rows reject even unchanged values. |
| Public events | `on_*` returns cloneable Event handles. UTF-16 payloads are `Vec<u16>`. Render and resize payloads are named structs. |
| Buffer namespace | `buffer()` lazily caches an `Rc<BufferNamespaceApi>` over the live BufferSet. |
| Parser view | `with_parser` gives a transient borrowed ParserApi. Registrations remain in the engine. Window-option checks precede public CSI t hooks. |
| Unicode view | `unicode()` checks the current allowProposedApi value and returns a shared service handle. It exposes the forwarding service rather than JavaScript wrapper identity. |
| `modes` | Owned `ModesSnapshot`, including inverse hidden-cursor state and source mouse protocol projection. |
| Markers | Shared Marker identities in owned list snapshots; `register_marker(offset)` and `add_marker(offset)` preserve signed offsets. |
| Input | `input(&[u16], was_user_input)` is separate from display writes. |
| Dispose | Fallible explicit disposal propagates engine and addon callback errors. Drop is not a substitute for fallible explicit cleanup. |

Native dimensions and direct line indices are unsigned. Native scroll counts and marker offsets are signed i64. Shared browser Number validators preserve finite integral values beyond machine index bounds before adapter conversion. Native unknown option keys reject rather than creating detached JavaScript expando properties. These representation differences must not be reported as JavaScript ABI equivalence.

## Remaining integration work

The synchronous native write convenience API is not the source WriteBuffer scheduling contract. `parse_write_data` bridges actual engine continuation into the portable queue; tasks 128 and 118 own native and browser host scheduling. Pending parser handlers expose `take_pending` and `resume`. Queue integration must account for cleanup futures, callback order, empty data, errors and disposal, not just immediate writes.

Reentrant observation and direct reset now use a shared state handle while the engine releases state borrows before callbacks. Reentrant writes, resize, input and disposal still require dedicated source comparisons and queue-aware integration. All additional PA01-PA06 paths in `docs/public-api-acceptance.md` remain required. There is no full wrapper acceptance or platform/browser equivalence claim from the scoped cases above.
