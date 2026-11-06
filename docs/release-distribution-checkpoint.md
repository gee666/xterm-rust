# Frozen release artifact checkpoint

Overall equivalence is NOT PROVEN. Task568 retains this measurement checkpoint. Task333 remains open for final integrated, genuinely matched distributions and a final source refresh. Sizes below do not establish working addon behavior, complete browser compatibility or a reduction in download size.

## Source and build identity

The Rust implementation is frozen at integration producer `6cfd03d7`, based on main `224d1b33`. This was not yet the canonical main revision when measured. The measuring branch also contains the Node accounting changes and a supplemental license notice, committed at `800b9e26`. No Rust implementation was changed for these measurements.

All1,965 Cargo/crate inputs have digest `19ffb731fe479eb0630bf8b01c74464c7be91ae039ab4641ab11e56d18b7cf95`. The release WASM module has SHA256 `94a7d311a39091127fd5f88eb8f33df24b6dce2ee124b97ff27a64144e997d83`. Exact files, hashes, build commands and output paths are retained under [results/distribution333](../results/distribution333/).

Native Windows x64 execution used Rust/Cargo1.85.1, LLVM19.1.7, Node24.15.0 and wasm-bindgen0.2.100. The runner made ordinary Cargo release defaults explicit for both builds: opt-level3, LTO false, codegen-units16, debug0, strip none, panic unwind, incremental false, overflow checks false and debug assertions false. The WASM target uses its supported target panic implementation. There was no wasm-opt, post-link stripping, acceptance-hooks feature or size-tuned alternate implementation. The private regular target was on O:, with Cargo jobs limited to2. The profile environment and exact compiler artifact messages are retained, rather than inferring settings from an artifact filename.

The unchanged upstream production commands built browser core, headless core and all14 addon bundles in a physical reference copy. Esbuild0.28.1 and existing installed dependencies were reused. No install ran, no credentials were copied, and the original reference was not built in place. All776 reference-manifest hashes matched in both the original and copied reference before and after the builds.

## Observed artifact totals

Web totals sum independently compressed files with gzip level9 and Brotli quality11, generic mode, window22. Each compressed payload was decoded and checked against its input. No archive compression or cross-file deduplication was used.

| Artifact set | Files | Raw bytes | gzip bytes | Brotli bytes |
| --- | ---: | ---: | ---: | ---: |
| Original browser core ESM, CSS and license |3|353,969|87,484|73,248|
| Original headless core ESM and license |2|140,385|34,597|29,398|
| Original declared core plus all14 addon ESM bundles and notices, not browser-compatible as emitted |36|1,077,496|305,184|251,557|
| Rust incomplete monolithic browser distribution,13 addon constructor exports |103|5,015,252|1,521,066|1,056,789|
| Rust native core example and shipped notices |20|1,574,087|not applicable|not applicable|
| Rust core rlib artifact and shipped notices, not a standalone SDK |20|13,954,531|not applicable|not applicable|

The native executable alone is1,544,192 bytes, SHA256 `6e510d5c1f738fa05e97b893b2506fc2892ae365701d45a208ca3c795337980d`. The rlib alone is13,924,636 bytes. An rlib needs dependencies and a compatible Rust toolchain to link; its row is an artifact inventory, not a complete redistributable SDK. Native executable bytes are not comparable to a JavaScript download. Windows platform runtime prerequisites are not embedded in these counts. No Linux/macOS artifact or runtime result is claimed.

The release example actually ran `crates/xterm-core/tests/fixtures/native-demo.txt` and completed all21 state assertions with disposal. This is a focused release runtime check, not the whole workspace quality or terminal-equivalence suite.

### Rust browser byte closure

| Component | Files | Raw bytes | gzip bytes | Brotli bytes |
| --- | ---: | ---: | ---: | ---: |
| WASM engine and compiled addon code |1|4,677,043|1,446,980|998,634|
| Generated binding JS |1|245,719|29,678|23,800|
| Required generated inline snippets |80|52,624|24,669|19,810|
| Public bootstrap |1|1,817|743|609|
| Required CSS |1|8,154|2,725|2,124|
| Shipped license and notice set |19|29,895|16,271|11,812|

Every generated runtime JS/snippet file is included, not merely the top-level binding module. The distribution also ships the project's complete text notice set, including optional fixture notices. That is an explicit, slightly broader packaging choice, not evidence that fixture fonts are embedded in WASM. No mandatory external font, worker or shader file was found in this library closure. Fonts supplied by an application or local-font access remain application inputs. The demo client itself is not in the library download totals. The two generated declaration files are development metadata excluded from runtime download sizes, but included in generated-source LoC.

### Why these are not matched-feature comparison results

The Rust binding JS exports13 addon constructors. `WebglAddon` is absent. Constructor presence also does not establish correct activation, rendering, disposal or source parity. Task306 still owns the actual WebGL integration. The requested14-addon set is recorded separately from observed exports. The first measurement metadata mistakenly called the requested set14 exported bindings. [The retained correction](../results/distribution333/rust/addon-label-correction.json) records the error, preserves the initial metadata, and verifies that every artifact hash and byte total remained unchanged. No rebuild or recompression was needed to correct those labels.

The Rust browser crate has no supported core-only web build. Its default distribution must not be compared against only original core and called a core-only size result. A final core-only browser measurement and a functioning matched addon distribution remain missing, not zero and not an external blocker.

The unchanged original ligatures production command deliberately uses `platform=node`. With the installed lru-cache11.5.0, its emitted ESM has a static `node:diagnostics_channel` import. [The import audit](../results/distribution333/original-imports.json) records the actual bundle hash and parsed top-level imports. That aggregate is not directly loadable as a browser ESM distribution. We retained the actual declared artifacts instead of modifying the original build or presenting a custom browser bundle as its output. A browser-compatible original ligatures distribution is another prerequisite for a genuine all-addon browser comparison.

No size ratio, saving or speedup is published from these unmatched rows.

## Dependency notices and retained failures

Production source maps identified five actually bundled upstream dependencies: sixel0.16.0, inwasm-runtime0.1.2, xterm-wasm-parts0.4.1, opentype.js2.0.0 and lru-cache11.5.0. Lru-cache uses BlueOak-1.0.0, not MIT. Their notices are included in the original all-addon total. Addon licenses are included individually, even where identical. The serialize addon lacks its own LICENSE in the supplied reference, so its root xterm license is shipped and recorded.

The installed inwasm-runtime package omits LICENSE. Its package metadata declares MIT and its distributed source identifies `Copyright (c) 2022, 2026 Joerg Breitbart` and `@license MIT`. `licenses/inwasm-runtime-MIT.txt` preserves that attribution with the standard MIT permission text. The runner permits this exact reviewed package/version/banner combination only and records both source and notice hashes. Unknown missing licenses or changed attribution fail. The notice is byte-preserved in Git.

Two real first attempts built all16 production commands successfully but failed during the subsequent dependency audit. One exposed file-URL source-map entries naming original TypeScript not shipped by npm. The other exposed the omitted runtime license. Both failed receipts remain under `original-first-failure.*` and `original-second-failure.*`. Neither emitted a successful size report. The corrected source-map package lookup and exact supplemental notice have negative tests. Original sources and assertions were not changed.

The final lightweight accounting controls passed46 Node test functions, with zero failures, skips or todos. These include compression round trips, mandatory/undeclared files, source mutation, prior-success invalidation, missing licenses, file-URL dependency maps, changed supplemental attribution, snippet/target links, actual release artifact selection and observed addon-export labels. This tests the accounting machinery, not terminal parity.

The successful original runner removes its own dependency link and staged files in `finally`. The Rust runner also removes its staged files, refuses linked runtime/target paths, validates actual compiler output paths and profiles, and checks source stability. Generated packages, reference build outputs and private Cargo targets remain owned disposable artifacts until review completes, then require separate inspected cleanup.

## Source lines including generated glue

The same checked-in UTF-8 physical/nonblank line counter scanned both trees. Comments are included. These are not comment-stripped SLOC or a claim of less code for equivalent behavior.

| Category | Original physical lines | Port physical lines |
| --- | ---: | ---: |
| Production library |28,590|71,153|
| Addons |15,738|15,303|
| Binding/glue source |4,823|2,899|
| Generated code/data, including generated runtime glue |154|21,423|
| Mixed code/data |528|0|
| Mixed production code and inline tests |0|9,689|
| Tests |38,811|127,368|
| Textual test data |3,643|520,924|
| Demos |4,116|983|
| Tooling |3,370|39,179|
| Configuration |3,198|928|
| Total |102,971|809,849|

The broader original scan counts635 files. The port scan counts2,469. Large retained regression corpora explain much of the port grand total. Do not compare that total against only original production TypeScript. [The per-file inventories and nonblank counts](../results/distribution333/loc/) retain the full classification, hashes and exclusions. The original legacy subtotal still matches396 files,96,130 physical lines,87,026 nonblank lines and3,962,777 bytes. The original classified inventory digest remains `319645846cc7e05738b9283d15a6497ec88bd2c9cb723e25c6986122ca533ece`.

This is the frozen checkpoint, not final whole-port LoC. New implementation, integration and generated artifacts require a final refresh under task333.

## Reproduce

Use a fresh owned checkout of the frozen source with a physical `source_repo` copy. Generated `browser/pkg` must not already exist; the Rust runner refuses prior snippets rather than silently including stale files. Keep staging under the checkout's `tmp/` or another explicitly owned scratch parent. Use a durable child of `results/` for Rust output. The commands below are portable except that the two placeholder paths must name the local existing dependency directory and a new owned scratch directory.

```sh
node --test scripts/distribution-size.test.mjs scripts/measure-original-size.test.mjs scripts/measure-rust-size.test.mjs scripts/count-loc.test.mjs
node scripts/measure-original-size.mjs --source source_repo --dependencies PATH_TO_EXISTING_NODE_MODULES --staging NEW_OWNED_JS_STAGE --output results/distribution333/original --addons all
node scripts/measure-rust-size.mjs --staging NEW_OWNED_RUST_STAGE --output results/distribution333/rust
node scripts/count-loc.mjs --check-baseline --output results/distribution333/loc
```

Cargo/Rustc, Git and the matching wasm-bindgen CLI must be in PATH, or use their existing explicit executable overrides. The Rust runner rejects target/profile/RUSTFLAGS overrides that would undermine the declared build. Run it in an owned regular worktree, with adequate RAM/disk and the project's coordinated resource grant. Tests, builds and subprocesses have bounded deadlines. Cleanup is not permission to delete someone else's artifacts or traverse reference links.

Timing measurements remain separate task329 work. No timing result is inferred from build durations. Full native platforms, source/assertion mapping, whole-browser/addon behavior, quality gates, final-source refresh and genuine matched distributions remain required.
