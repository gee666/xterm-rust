# Oversized HTML starting columns

Task467 addresses one raw serialization gap in346. A primitive numeric `startCol` at least `u32::MAX` cannot enter the source's `col < endLineColumn` loop over the real Rust-backed buffer. It must still run the HTML header, emit the first empty row fragment, and traverse subsequent rows from column zero. The old adapter instead rejected the coordinate before the header.

The browser adapter reuses the shared serializer's empty-first-row sentinel for these values. The wasm32 column boundary is an implementation bound, not user policy. No valid native buffer column can reach it. This does not authorize ignoring arbitrary replacement buffer methods or mutable JavaScript line lengths, which remain346 work. It also does not generalize finite fractional or negative columns below that boundary.

The repair shares the existing HTML engine. Native integral APIs, row lookup, captured end columns, warning callbacks, palette handling and output assembly do not change. Primitive numeric huge columns are not source hangs: their first comparison is false. This is distinct from equal infinite row bounds whose increment may not progress.

## Regression cases

`scripts/serialize-browser-cases.mjs` contains two shared actual-source/actual-WASM probes:

- `html-oversized-start-column-reentry` uses six boundary values, including a previously supported control immediately below the boundary, fractional values above it, the safe-integer maximum and the largest finite double. Both header growth and shrinkage check exact HTML, getter order, captured end columns, warnings and full public-cell snapshots.
- `html-oversized-start-column-header-throw` checks that the original header-thrown object, not an eager coordinate RangeError, reaches the caller for each value.

The source-only Node receipt supplements these browser tests. It is not Rust or browser proof. The browser fixture retains all previous directed cases and32 seeded12-operation cases with seed `0x05e21a11`. No original assertions or reference files change.

## Verification status

Results are retained under `results/serialize-oversized-column/`. Baseline source `f29a86b9` matches65/68 cases per Chromium148, Firefox150 and WebKit26.4. Both new probes and the existing raw ANSI numeric probe fail. Fixed source `be1f294b` matches67/68 per engine. Both new probes now match; the unchanged ANSI failure remains because this branch deliberately does not replay456. The runner retains exit1 rather than hiding that mismatch.

The unchanged original suite passes120/120 against source and120/120 against Rust across the three engines, with zero failures, skips or unrun cases. Both browser runs verify all776 original files before and after, with zero mismatches. The two captured source identities hash1660 build inputs. `browser-proof.mjs.txt` records the bounded source/package snapshot swap, identity assertions and finally restoration. No browser or source package was silently rebuilt during proof.

All15 existing native serialization executables pass64 tests. Strict focused native serialization and WASM library Clippy pass with warnings denied. Web-package formatting passes. Exact workspace formatting fails Windows OS206 and remains a failed required gate, not a pass. Full346/322, workspace/all-feature/release, native Linux/macOS and overall equivalence remain NOT PROVEN.

The compiler allocation followed post409 comment3142's next-release order and465 release3158. Compiler467 released at3184. Browser467 followed3181 and464 release3188, was confirmed by3199, and fully released at3212. No heavy jobs or reviewer grants remain.

Reproduce in an owned worktree with the current team allocation:

```sh
CARGO_BUILD_JOBS=2 node scripts/build-demo.mjs --debug
SOURCE_ROOT=/unchanged/reference node scripts/serialize-browser-check.mjs --suite
cargo test --locked -j2 -p xterm-core --test serialize_html --test serialize_html_edges --test serialize_html_numeric_rows
cargo clippy --locked -j2 -p xterm-web --lib --target wasm32-unknown-unknown -- -D warnings
cargo fmt -p xterm-web -- --check
cargo fmt --all -- --check
```

`quality.json` retains the full15-executable native command and both strict Clippy invocations. The build logs retain Rust/Cargo/wasm-bindgen versions and exact artifact hashes. After the bounded proof verified the restored fixed package, generated shared report copies and the global latest-build pointer were restored to their branch state. The task-specific `fixed/build-pointer.json` and immutable build report identify the actual fixed package; do not treat the restored global pointer as its provenance. The retained failure is not a claim about current integrated main, which advanced independently while this branch was tested. Final integration must preserve the coherent fixture union,72 cases after446/456 and these two new cases, rather than overwriting another producer's assertions.
