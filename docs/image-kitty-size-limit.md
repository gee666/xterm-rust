# Raw Kitty size limit

Task465 repairs the `kittySizeLimit` constructor and activation boundary. Full image-addon parity remains open in311/324.

Source `ImageAddon` copies constructor options twice without converting this value. It keeps the first copy. Disabled Kitty support never converts it. Each enabled activation converts it once in the Kitty handler constructor, after earlier renderer, storage, control and IIP registrations. Reset, disposal and payload streaming do not repeat that conversion. Each new activation converts the retained first-copy value again.

The browser binding now stores the raw value and returns its identity through the existing read-only `_opts` diagnostic view. The typed native `ImageOptions.kitty_size_limit` API is unchanged. The live browser coordinator converts the raw value immediately before constructing `BrowserKittyProtocol`, without holding a state borrow across foreign code. A thrown value prevents Kitty registration without rolling back earlier activation work.

A small JavaScript expression evaluates the source's encoded-limit arithmetic, including number-hint conversion, mixed-BigInt rejection and exact operand spelling. Some browser engines include the operand text in generated errors. A temporary binding receiver supplies that expression; it holds no terminal or protocol state. The native controller has a shared `new_with_encoded_limit` constructor for this precomputed number. Its typed constructor computes the same cap natively and delegates to that constructor. Initial-cap policy, payload decoding, limits, placement, storage and pixels remain Rust-owned. This repair adds no dependency or unsafe code.

This does not complete source's writable private `_opts` ABI, other raw options, arbitrary numeric decoder settings, full core `_inputHandler` access, garbage-collection proof, original639-family assertions or img01..48. Invalid numeric decoder settings and source hangs retain their existing separate safety/compatibility obligations. They are not relabeled as equivalent by this boundary repair.

Independent review accepted465 at `342ea3ef`. It checked the pinned inputs and original hashes and independently reran both native functions. It audited the retained browser matrices without launching builds or browsers. See `results/image-kitty-size-limit/review.md`.

## Verification

The new six-function browser regression file covers20 raw constructor values, two copy identities, disabled activation, repeated enabled activation, six nonfinite or nonnumeric activation-only forms, exact conversion errors, prior IIP registration after failure, reset/dispose reentry and14 numeric forms through real streamed Kitty payloads and pixels. Existing42 addon functions remain unchanged.

The initial implementation used multiplication by one as a number-hint conversion. Chromium passed48/48, but Firefox and WebKit passed47 and failed the exact-error assertion. Firefox named a different operand, while WebKit described multiplication by one as numeric conversion rather than mixed-BigInt multiplication. The initial failures are retained, not relabeled as passes. The implementation now evaluates the exact source expression. No error assertion was relaxed.

The fixed target passes48/48 in Chromium148.0.7778.96, Firefox150.0.2 and WebKit26.4. The control restores all seven production files to archived451 `a618ff27`, retains new tests and the accepted visibility fixture, and completes43PASS5FAIL in Chromium. Exactly the five new boundary functions fail; the numeric payload function and all42 existing functions pass. Finally restores all seven fixed files byte-for-byte. A subsequent full matrix again passes48/48 in each engine, with zero ignored or filtered tests.

Two native test functions each replay all235 retained original Kitty traces and7129 snapshots. The first uses the unchanged typed constructor. The second uses the host-evaluated cap with a deliberately zero typed decoded-size option, proving that the provided cap controls streaming. This is protocol/state/reply evidence, not native pixel rendering. Scoped native and WASM Clippy pass with warnings denied. All five workspace packages pass individual formatting checks. The exact aggregate `cargo fmt --all -- --check` still fails Windows OS206. Full workspace and native OS matrices were not run by465.

Raw reports, initial failure diagnostics, control audit, source pins and command logs are in `results/image-kitty-size-limit/`. Historical temporary artifact paths identify actual builds; they are not permanent installations.

```sh
cargo clippy --locked -j2 -p xterm-web --target wasm32-unknown-unknown \
  --lib --test image_addon_browser -- -D warnings
WASM_TEST=image_addon_browser CARGO_BUILD_JOBS=2 node scripts/image-canvas-check.mjs
node results/image-kitty-size-limit/negative-control.mjs
cargo fmt --all -- --check
```

Use `docs/image-canvas.md` for matching reference/dependency/bindgen settings. New temporary work is owned under `O:/xterm-rust-swarm-16/tmp/worktrees/Sara465`. The branch starts directly at archived451 `a618ff27`. Do not replay451, the private455 ancestor or448 prerequisites when integrating465. Only commits after this base belong to465. Full Rust xterm.js equivalence is NOT PROVEN.
