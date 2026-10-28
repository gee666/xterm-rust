# Unicode browser binding work

This is an implementation contract, not completed evidence. Native providers are implemented in task 41. Complete JavaScript-facing Unicode handling and addon classes still require Rust-backed WASM bindings. The original Unicode11 and grapheme Playwright suites must run against those bindings, not just component helpers.

## Required binding names

Expose `Unicode11Addon` and `UnicodeGraphemesAddon` constructors with working `activate(terminal)` and `dispose()` methods. The public terminal addon manager must own their callbacks and disposal. Activation must use the actual gated Unicode service belonging to that terminal. Unicode11 registration does not select version11; grapheme registration selects15-graphemes and disposal restores the exact old version.

`terminal.unicode` enforces the live proposed-API gate and returns a fresh view over that same service, as the source getter does. The view exposes `versions`, `activeVersion` get/set and `register(provider)`. It must not maintain an independent registry or execute source_repo terminal logic.

## Lossless names and callbacks

JavaScript names are UTF-16 sequences, including lone surrogates and NUL. The current native UTF-8 name contract is insufficient for this binding. Do not use lossy conversion, drop entries, encode private fake identifiers into the registry, or silently replace invalid names. Add coherent lossless core name APIs and update the grapheme addon's saved version to use them. If existing native signatures need adjustment, coordinate one atomic migration and update callers/tests. Do not introduce successful fallback values merely to preserve an incompatible signature.

Preserve canonical numeric-name Object.keys ordering, all other insertion order, duplicate replacement, the active provider snapshot, and empty-name sentinel behavior. Include lone-high/lone-low names, mixed surrogate pairs, NUL, literal escape-looking names and collisions with valid strings. A builtin provider and a registered JS provider must share the same selection state.

Provider callbacks may throw arbitrary JsValue payloads. Core must represent fallible foreign providers without depending on js-sys and without panic, throw_val, swallowed errors, or fake width/property values. The browser layer retains the original thrown value and callback receiver. Release every service borrow before JS getters or callbacks so they can reenter registration/selection.

`charProperties` returns a JavaScript number, not necessarily an already truncated u32. Preserve its original f64 value for the next foreign callback's preceding argument, while extracting flags through JS ToInt32 semantics. Include -0, NaN, infinity, negative values and 2^32+k. Keeping only packed u32 bits loses observable callback input even when widths happen to match. Native builtin properties remain the normal packed-number path.

The source reads provider.version once for the registry key and again when selecting a first provider. A getter can throw or return a different value on its second read. Preserve committed registry/selection state at each failure and source callback order. Read wcwidth and charProperties at call time, not as permanently cached functions, unless source behavior proves caching equivalent. Source shape and numeric conversion boundaries must be explicit rather than inferred from a passing builtin-provider test.

## Evidence

Run valid custom providers, thrown getter/method payloads, reentrant selection and malformed UTF-16 names through actual Rust/WASM terminal APIs. Keep all original addon registration/version/width assertions intact in Chromium, Firefox and WebKit. Component tests in `crates/xterm-web/tests/unicode_addons.rs` remain separate evidence until real terminal/addon loading is tested. No claim of full Unicode browser support is justified while this contract is unfinished.
