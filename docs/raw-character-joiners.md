# Raw character-joiner service

Task335 replaces the browser joiner's eager array and integer validation with raw JavaScript storage. The portable service still accepts typed Rust handlers. Both use one line and attribute traversal. The browser merge and string-to-cell conversion run in Rust; small JavaScript imports perform property access, strict assignment, comparison, arithmetic, method calls and receiver-record construction.

The first handler's outer container and inner ranges remain live during merging and conversion. Array-like results work without an Array requirement. Insertion retains the new range object; merging mutates existing objects. Repeated property accesses, JavaScript comparison coercion, strict setters and partial mutation survive. Handler and merge failures go to console.error as the original value, then later handlers continue. Conversion and outer collection failures escape without logging. A throwing logger also escapes unchanged.

Browser callbacks receive the source joiner record as this, with id and handler properties. Subsequent calls read the live handler property. Deregistration reads the live id outside Rust borrows. Native typed handlers retain their existing Rust representation.

## Reproduction

Run from a worktree with Node24.15.0, Rust1.85.1, a private Cargo target and Cargo jobs limited to two. Put TEMP and TMP in the owned O: tree. Source dependencies can be reused without a dependency junction by setting SOURCE_PACKAGE_JSON to the installed reference's package.json.

```sh
SOURCE_PACKAGE_JSON=/absolute/reference/package.json node scripts/joiner-raw-oracle.mjs
cargo test -j2 -p xterm-web --test character_joiner_upstream
cargo clippy -j2 -p xterm-web --all-targets --all-features -- -D warnings
cargo clippy -j2 -p xterm-web --target wasm32-unknown-unknown --lib --test wasm_joiner_raw --test dom_font_browser --test wasm_font_terminal -- -D warnings
WASM_BINDGEN=/absolute/wasm-bindgen PLAYWRIGHT_MODULE=/absolute/playwright/index.mjs WASM_TEST=wasm_joiner_raw node scripts/browser-host-check.mjs
```

Use wasm-bindgen0.2.100, matching Cargo.lock. Repeat the last command with WASM_TEST=dom_font_browser, wasm_font_terminal and wasm_font_shaping for the existing renderer/font regressions. The browser runner closes its browsers and server on completion or failure. No source terminal engine runs in the Rust tests.

The oracle freshly compiles the unchanged CharacterJoinerService source and all24 original unit tests. It also verifies36 directed operation cases and256 randomized cases using seed0x335C0DE. Raw source hashes, fixture hash and exact source counts are in results/joiner-raw/source.json. The native replay test checks27 previously captured original service calls. These counts describe different checks and must not be added as a single assertion total.

The shared raw cases cover retained identities, wide-cell offsets, array-likes, falsy heads, frozen containers, getter/setter/coercion failures, console error identity, live handler registration, receiver-record mutation, primitive endpoints and all nine merge branches. The WASM tests compare the complete resulting JSON, including mutation traces, against the source fixture and replay the same27 original captured calls with raw browser storage. A separate real Terminal test checks mutation visible in the caller's range, joined DOM spans, conversion exception identity and handler error logging through the active renderer.

## Remaining work

Full equivalence is NOT PROVEN. Task348 owns replacing the DOM row factory's eager typed projection with raw, source-timed consumption. Accessors reached after service conversion, endpoint arithmetic, zero-length and malformed ranges, and out-of-bounds row behavior still need that work. No full raw DOM row parity, complete font metadata parsing, pixel equivalence, native Linux/macOS runtime or full original browser suite claim follows from these focused tests. Task342 retains complex font metadata, and the full suite and audit tasks remain open.
