# Mouse mode string-name errors

The live browser `mouseStateService` rejects unknown primitive-string mode names
with the original JavaScript UTF16 code units in `Error.message`. Native mode
lookup, active state, parsed modes and event dispatch remain in Rust. A tiny JS
error-construction boundary avoids Rust scalar-string replacement of lone
surrogates. The browser Rust setter parameter is now `js_sys::JsString` rather
than `&str`; JavaScript binding names are unchanged.

The native service currently exposes only ASCII builtins through this browser
facade. Its lookup may use a scalar string because no builtin can contain a lone
surrogate. Only unknown-name errors use the original JS string. Dispatch errors
still use the existing identity-preserving conversion. Invalid names do not
change state or emit events. Reentrant listener mutations preceding an exception
remain committed.

This closes only primitive-string unknown-name text. Raw custom registries,
inherited keys, nonstring property-key coercion, borrowed receivers and patched
service methods remain task305. Nonstring names are not accepted by this typed
browser setter. There is no claim of general registry equivalence.

## Checks

Windows execution with rustc/cargo1.85.1, Node24.15.0 and wasm-bindgen0.2.100:

| Check | Chromium148.0.7778.96 | Firefox150.0.2 | WebKit26.4 |
| --- | --- | --- | --- |
| Unchanged-source generation and drift check, each phase | 5/5 | 5/5 | 5/5 |
| Real Rust test functions, initial and final phases each | 5/5 | 5/5 | 5/5 |
| Existing mode test functions | 5/5 | 5/5 | 5/5 |
| Existing builtin-method test functions | 4/4 | 4/4 | 4/4 |
| Existing wheel-receiver test functions | 3/3 | 3/3 | 3/3 |
| Fixed direct-ABI case matches, zero Rust test functions | 5/5 | 5/5 | 5/5 |

No failed, ignored or filtered Rust tests in these matrices. The exact old
735e114a mouse API, compiled with the same new test inputs, mismatches all five
shared cases through direct Chromium ABI calls. These are five counterexample
attempts, not five failed Rust test functions. The minimal counterexamples retain
U+FFFD in old error text where source and fixed contain U+D800. Both preserve
the initial modes and Error class. Old Firefox/WebKit controls were not run.

The driver restored exact fixed source bytes in finally. Cargo reports
fresh=false for both old and restored builds, their compiler hashes differ,
and the restored compiler, post-bindgen WASM and bindings equal the initial
fixed hashes. The fixed direct control uses this identical initial package.
Native mouse tests pass24/24, including an independent execution. Scoped strict
native all-target/all-feature, WASM-library and four mouse WASM-test Clippy pass.
All five package format checks pass. The exact workspace format command fails
with Windows OS206. Full workspace/platform gates remain open.

Identical shared cases are designed to run against unchanged source Terminal and Rust Terminal.
They cover 16 explicit names for both setters, all 2,048 lone surrogate code
units for both setters, 128 deterministic names from seed `0x6d6f7573`, an
8,197-code-unit name, exception identity during listener reentry, retained state
and events, parsed protocol/encoding changes and reset. The generated cases
assert complete error text, not a normalized or lossy rendering.

Commands, source/artifact pins, actual results and the old counterexamples are
retained in `results/mouse-name-errors/`. The initial and final reports retain
original generated paths; `artifact-links.json` maps their hashes to durable
compiler/package copies. `restoration.json` binds the old control to the exact
restored source and compiler. `native-pins.json` identifies independently runnable
24-test executables. `control-driver.mjs` is the exact script run from
`tmp/name-negative.mjs`, retained for inspection and replay in that location.

From a private worktree with the existing reference tools configured:

```sh
MOUSE_CASE_FAMILY=name-errors node scripts/mouse-mode-source.mjs
WASM_TEST=wasm_terminal_mouse_names node scripts/browser-host-check.mjs
MOUSE_NAME_EXPECT=fixed MOUSE_NAME_CONTROL_DIR=results/mouse-name-errors/artifacts/fixed BROWSERS=chromium,firefox,webkit node scripts/mouse-name-control.mjs
```

Set XT_REFERENCE_ROOT, PLAYWRIGHT_MODULE and WASM_BINDGEN to the existing
reference and matching0.2.100 CLI. Coordinate resource grants, use Cargo-j2,
and keep CARGO_TARGET_DIR/TEMP/TMP in the owned O: worktree. The existing mode,
builtin and wheel targets use the same browser runner. No source assertion was
modified; all776 baseline reference hashes match before and after execution.
Full port equivalence remains NOT PROVEN.
