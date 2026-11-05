# Browser marker registration and handles

Full equivalence is NOT PROVEN. This adds working public marker registration and
live handles, not complete marker/decoration browser parity.

## Implementation

`crates/xterm-web/src/terminal/marker_api.rs` exposes `registerMarker`, `markers`,
and returned marker `id`, `line`, `isDisposed`, `dispose` and `onDispose`. Handles
read the actual native Marker. Buffer trimming, alternate-buffer activation and
native disposal therefore update the same state that parser operations use.
There is no proposed-API gate, matching the supplied public Terminal.

Registration retains the original JavaScript argument through both Number-hint
validation conversions and the later default-hint addition. That addition can
produce a String line. Negative and large finite integer offsets are not
narrowed to an integer Rust type. The buffer receiver is captured before the
addition, so an offset coercion that resets the terminal cannot redirect the
already selected call. No buffer borrow is held while coercion invokes JS.

The small JS helpers implement ABI coercion and Array projection only. They do
not run a JavaScript terminal engine. WeakRef caches preserve observable handle
and list identity without strongly retaining otherwise unreachable JS wrappers.
WeakRef is a browser built-in supported by the three tested engines, not a new
package dependency. The existing event adapter preserves receiver, subscription
collection, disposal-token behavior and thrown JS object identity.

`markers` is a live read-only Array view of the active native buffer's marker
list. A saved view follows native registration, trimming and clearing, and normal
and alternate buffers have distinct list identities. This is more limited than
the supplied source's physically writable Array. The public TypeScript contract
is ReadonlyArray, but raw JavaScript can still mutate the original. Such writes
must not be described as equivalent here.

## Remaining contracts

Task331 retains source mutable-array behavior, writable marker fields and the
extra internal Marker.register method, full registerDecoration/returned decoration
bindings with live raw option identity, render callbacks and error order. Public
marker fields are getter-only in this checkpoint. Decoration rendering through
native services does not prove those missing public bindings. GC/lifecycle
coverage beyond the listed cases, the unchanged complete original browser suite,
and native Linux/macOS evidence are not supplied by this checkpoint.

## Verification

The shared `browser_marker_cases.js` fixture runs against the unchanged supplied
browser Terminal and the actual Rust/WASM Terminal, without opening a fake core
or loading the source engine into the Rust test. The source runner and Rust tests
have independent expected captures. These are new source-derived assertions,
not a claim to execute an unchanged original suite.

Cases cover default/null/boolean/number/string/large offsets, invalid offsets,
Symbol and BigInt errors, original coercion exception identity and hint order,
String addition, cached handle/event identity, live Array membership/enumeration,
subscription collection and cancellation, native scrollback trimming,
idempotent disposal, listener throw identity, normal/alternate activation and
reset during default-hint coercion after capturing the buffer.

Commands use the existing reference Node dependencies and wasm-bindgen 0.2.100.
Set Cargo, Node and wasm-bindgen on PATH, TEMP/TMP to an owned tmp directory,
CARGO_BUILD_JOBS=2, PLAYWRIGHT_MODULE to the reference playwright/index.mjs,
REFERENCE_ROOT to the unchanged source_repo, and WASM_BINDGEN to the matching CLI.

```sh
node scripts/terminal-marker-oracle.mjs
WASM_TEST=wasm_terminal_markers node scripts/browser-host-check.mjs
cargo clippy -p xterm-web --target wasm32-unknown-unknown --lib --test wasm_terminal_markers -j2 -- -D warnings
rustfmt --edition 2021 --check crates/xterm-web/src/terminal/marker_api.rs crates/xterm-web/tests/wasm_terminal_markers.rs
```

Toolchain observed on this Windows host: rustc1.85.1, cargo1.85.1, Node24.15.0.
Source and Rust each passed all three scoped cases in Chromium148.0.7778.96,
Firefox150.0.2 and WebKit26.4. That is 9 source executions and 9 Rust executions,
with no failed, ignored, filtered or skipped cases in this scoped suite. Scoped
warning-denied WASM Clippy and changed-file formatting passed. This is not the
required whole-workspace/all-features/native-platform verification gate.

Durable reports are `results/browser-markers/source.json`, `rust.json`,
`checks.json`, `build.log` and `clippy.log`. Reports retain the original source
input hashes, browser versions, exported Rust test names, exact commands, counts,
source file hashes and generated artifact hashes. Generated WASM/binding copies
and the private target are disposable after review; their former absolute paths
in rust.json are provenance, not promises that those temporary files remain.
