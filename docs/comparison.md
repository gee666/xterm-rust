# Original and Rust comparisons

Overall equivalence is **NOT PROVEN**. The historical task303 measurements below cover source counts and original core-only production ESM artifact sizes. A newer [frozen release checkpoint](release-distribution-checkpoint.md) measures actual native/Rust-WASM artifacts, all14 original addon bundles, dependency notices and generated-glue LoC. It explicitly retains the missing Rust WebGL export, unavailable core-only WASM build and original ligatures browser-packaging gap. Neither snapshot is a matched-functionality comparison, a Rust size reduction or a speedup claim.

## Source lines

The [classified table](../results/loc/report.md), [summary](../results/loc/summary.json) and per-file [original](../results/loc/source.json) and [port](../results/loc/port.json) inventories use the same physical/nonblank UTF-8 line algorithm. Comments are included. These are not comment-stripped SLOC counts. The method and classification boundaries are documented in [loc.md](loc.md).

Run from the repository root with Node 24 or later:

```text
node --test scripts/count-loc.test.mjs
node scripts/count-loc.mjs --check-baseline
```

The task 303 snapshot starts from main `3ee576a9b01c774d2305936ec9e44993df7f0b5d`, with the current intent and comparison-tool changes. It is not a final stable integration snapshot. The source legacy subtotal still matches all four original baseline values: 396 files, 96,130 physical lines, 87,026 nonblank lines and 3,962,777 bytes. The broader classified original scan has 635 counted files and 102,971 physical lines. Extra configuration and textual fixtures explain the broader total.

The port includes substantial retained textual regression fixtures, generated tables, tests, tooling, browser glue and mixed production/test files. Do not compare its grand total against only the original production TypeScript. That task303 snapshot had no generated `browser/pkg` distribution, so those historical counts omit generated WASM-binding JS and snippets. The newer frozen checkpoint includes them. Refresh after all production features and runtime glue are integrated. Missing generated files are not evidence that glue costs zero lines.

`headless/lib-headless` is excluded as generated original build output, matching the unchanged upstream production build script. A regression test checks that this output cannot inflate source counts.

## Observed original production ESM sizes

Native Windows execution used Node `v24.15.0`, esbuild `0.28.1`, zlib `1.3.1-e00f703` and Brotli `1.2.0`. The commands below ran unchanged source scripts in an isolated O: reference copy:

```text
node bin/esbuild.mjs --prod
node bin/esbuild.mjs --prod --headless
```

Both exited 0. Source inventory SHA256 before and after the builds was `319645846cc7e05738b9283d15a6497ec88bd2c9cb723e25c6986122ca533ece`. This identifies the supplied reference inventory, not an invented upstream public revision. The supplied package-lock SHA256 was `1d4fe6fb8f91e404df5608b4421008116ac9d5d37b038c9b16d261246d5d5e0e`. Existing installed dependencies were reused without reinstalling them; their observed esbuild version is recorded separately from the lock hash. No claim is made that a fresh install was executed.

| Distribution | Raw bytes | gzip bytes | Brotli bytes |
| --- | ---: | ---: | ---: |
| Original browser core ESM, CSS and license | 353,969 | 87,484 | 73,248 |
| Original headless core ESM and license | 140,385 | 34,597 | 29,398 |

The browser JS bundle alone is 344,533 raw bytes; required CSS is 8,154 and the license is 1,282. The headless JS bundle alone is 139,103 raw bytes. All fourteen optional addons are excluded from these two core-only distributions. Development sourcemaps, declarations and package-manager metadata are not in these direct-ESM runtime totals. There are no external mandatory fonts, workers or shaders for these original core-only bundles.

Compression applies gzip level 9 and Brotli quality 11, generic mode, window 22 to each file separately. Totals sum these independent compressed files, not a compressed archive. Each compressed payload is decoded and compared byte-for-byte with its input. Hashes and per-file totals are in [original-sizes.json](../results/comparison/original-sizes.json); the exact selections and release flags are in [original-manifest.json](../results/comparison/original-manifest.json). [original-build.json](../results/comparison/original-build.json) preserves command outcomes, tool versions and source identity. No native LTO, stripping or Rust optimization flags apply to these JS bundles.

## Reproduce the original measurement

Use a physical isolated reference copy, such as the one made by the worktree helper. It must have no `node_modules` entry. Dependencies must already be installed in the supplied original reference. Do not copy credentials or install another dependency tree just for this command.

```text
node scripts/measure-original-size.mjs --source source_repo --dependencies P:/workshop/Windows_Worshop/oira666/xterm-rust/source_repo/node_modules --staging O:/xterm-rust-swarm-16/tmp/original-size-owned --output results/comparison
```

The paths above reproduce this Windows environment; replace them with the corresponding isolated reference, existing dependency and owned scratch paths on another system. Staging must not exist and must be disjoint from the source and durable results. The runner creates a temporary dependency junction or directory symlink, invokes the two unchanged upstream production builds with 60-second deadlines, checks unchanged source inventories, stages runtime files, and writes the three JSON reports. It removes only its created link and staged files in `finally`. Generated upstream `lib`, `out-esbuild`, `out-esbuild-test` and `headless/lib-headless` remain in the owned reference copy until that worktree is cleaned.

An initial manual attempt pointed at an integration reference without installed esbuild and failed with `ERR_MODULE_NOT_FOUND`. It produced no measurement. Using the actual retained installed dependency directory resolved it. The failure is retained in [original-first-failure.json](../results/comparison/original-first-failure.json).

## Measure other release distributions

Stage exactly the files shipped to users, then run:

```text
node scripts/distribution-size.mjs --manifest distribution.json --root path/to/staged-distributions --output results/comparison/sizes.json
```

The manifest schema is illustrated by `original-manifest.json`. Each distribution needs a unique `id`, normalized relative `directory`, `kind` of `web`, `headless-js` or `native`, `profile` of `release`, explicit `scope` and `addons`, plus `build.commands`, `build.sourceSha256`, `build.toolchain` and `build.flags`. Each file has a normalized relative `path` and nonempty `role`. Every regular file under that distribution directory must be selected or listed in `excluded` with a nonempty reason. Include all mandatory WASM, binding JS/snippets, bootstrap, CSS, fonts, workers, shaders and licenses.

Missing or undeclared assets fail. Linked roots or entries, absolute/traversal paths, conflicting declarations, nonregular entries, malformed JSON, invalid flags and missing release metadata fail rather than contributing zero bytes. Repeated references to the same file path count once within a distribution. Distinct paths are never deduplicated merely because content hashes match. Separate distributions each include their own mandatory files. The output must be outside the input root. Native artifacts receive raw counts only; a native executable is not a JS download.

Build metadata in the generic manifest is supplied by the caller. Size accounting cannot establish that an arbitrary artifact was built from the claimed source or implements the required behavior. Preserve actual build receipts, source identity and matching output validation alongside each final measurement.

## Tests and unfinished measurements

```text
node --test scripts/distribution-size.test.mjs scripts/measure-original-size.test.mjs scripts/count-loc.test.mjs
```

All 34 tests pass on Windows Node 24. [tests.txt](../results/comparison/tests.txt) preserves the output. Coverage includes binary/Unicode/CRLF/empty compression round trips, deterministic recounts, duplicate references, missing/undeclared files, exclusions, traversal, junction refusal, native/web distinction, CLI failure statuses, source mutation detection, build failure propagation, stale-size invalidation and owned link/staging cleanup. The first test run had 33 passes and 1 failure in empty-directory fixture cleanup. It is retained in `tests-first-failure.txt`; the fixture was corrected and its abandoned owned temporary directory was removed. No source assertion was changed.

Still required for final acceptance, not zero and not an external blocker:

- A final stable integrated refresh of the frozen Rust native and browser/WASM artifact measurements, with actual strip/LTO/codegen/panic/optimization flags and all mandatory assets.
- Genuine core-only and matched-feature browser distributions. The current Rust package lacks WebglAddon and a supported core-only build; original all-addon declared bundles also include a node-platform ligatures import that browsers cannot resolve.
- Final stable integrated LoC including generated runtime glue. The frozen checkpoint now includes generated binding JS, snippets and declarations, but later implementation changes require recounting.
- Final integrated timing refresh after remaining functionality changes. The [matched headless checkpoint](matched-headless-performance.md) now retains actual production/release adapters, seven paired workloads, warmups, ten measured repetitions and raw median/p95 data. Its public-queue and synchronous-core modes are distinct; neither measures browser rendering or proves full parity.

Task 568 retains the [frozen release checkpoint](release-distribution-checkpoint.md), including actual bytes, per-file hashes, failures and reproduction commands. Task 333 remains open for final release distribution and LoC refresh using these tools. Task 326 supplies the [matched benchmark protocol and fail-closed runner](benchmark-protocol.md), with shared input/state validation and tested median/p95 aggregation. Its synthetic negative tests are not performance measurements. Task 329's [actual headless adapters and measurements](matched-headless-performance.md) validate complete terminal/event/reply and addon outputs on the pinned integrated checkpoint. Rust was slower in the synchronous medians; public-callback latencies include a different queue and must not be sold as engine speedups. A final refresh remains required when measured production sources change. [acceptance/comparison.md](acceptance/comparison.md) remains the complete comparison contract. Partial green inventory tests do not satisfy it by themselves.
