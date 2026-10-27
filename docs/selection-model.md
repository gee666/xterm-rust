# Selection coordinate model

`crates/xterm-web/src/selection/model.rs` ports `source_repo/src/browser/selection/SelectionModel.ts`. It operates on buffer cell boundaries only. It does not inspect wide characters or implement browser mouse selection. The upstream MIT notice is retained.

`Position` stores a u32 column and signed i64 row. Rows can become negative while handling scrollback trim. Initial selection length is u32, matching the upstream buffer's maximum u32 cell capacity. `Dimensions` requires nonzero columns and rows and receives the current scrollback base on every final-end query. This prevents cached select-all endpoints after resize. Invalid JavaScript fractions, negative columns, NaN and infinities require validation in the public binding; this native coordinate model does not accept them.

`SelectionModel` exposes the four source fields with Rust names. `clear_selection`, `are_selection_values_reversed`, `final_selection_start`, `final_selection_end` and `handle_trim` preserve the source branches. End and trim operations return a recoverable `CoordinateOverflow` if caller-supplied i64 row arithmetic overflows. Trim validates both rows before mutating either. This Rust boundary differs from JavaScript's floating-point row arithmetic outside the supported buffer range and is not a claim of equivalence for arbitrary Numbers.

One source distinction deserves care. A minimum word selection ending at an exact multiple of the column count uses the preceding row's right edge when the explicit end is absent or reversed. A forward explicit endpoint on the start row takes the other source branch and ends at column zero on the following row. Both branches are retained and tested.

## Checks

Run from the repository root with Node 24.15 or newer and the workspace Rust toolchain:

```text
cargo test -p xterm-web --test selection_model
node scripts/selection-model-oracle.mjs
cargo clippy -p xterm-web --test selection_model --example selection_oracle -- -D warnings
```

The Rust suite has 22 tests. Its first 18 map one-for-one to the 18 original `it` bodies. Four added tests cover the exact-wrap branch distinction, no-refresh trim and complete clear state, live select-all dimensions, and recoverable numeric overflow with atomic trim.

The Node oracle transforms the unchanged source class and test assertion bodies. Its imports replace Chai with Node strict assertions and replace the full MockBufferService with a coordinate fixture containing only `cols`, `rows` and `buffer.ybase`. The assertion bodies are unchanged. This is a scoped assertion-body execution, not a claim that the original Mocha runner or complete buffer integration passed.

A line-oriented Rust example, `examples/selection_oracle.rs`, receives the same 38,130 cases as the source model. Of these, 20,000 are seeded random cases using xorshift32 seed `0x051ec710`. The remaining 18,130 form a deterministic cross-product over dimensions, missing/reversed endpoints, minimum lengths, select-all and trims. Inputs include one-column terminals, exact wraps, negative transient rows and u32::MAX dimensions/lengths/trims. Each case compares final start/end, reversal, trim refresh, raw start/end, minimum length and select-all state. Clear operations occur in the seeded cases.

`results/selection-model/oracle.json` records reference and implementation hashes, Node/platform identity, original assertion-body counts, seed, case counts, command, corpus hashes and the result. The driver has an explicit three-minute timeout, keeps compile errors in `native-stderr.log` and retains the first mismatch with its seed and input in `counterexample.json`. A successful rerun does not erase an earlier retained counterexample. No test files or reference assertions are modified.

Observed locally on Windows: 22 Rust tests passed, 18 original assertion bodies passed, and all 38,130 JS/Rust cases matched. Exact rerunnable results live under `results/selection-model/`. Linux/macOS native execution, SelectionService text extraction, terminal integration and DOM/browser interaction are not established by these checks. Broader acceptance is specified in `docs/selection-acceptance.md` and remains unfinished.
