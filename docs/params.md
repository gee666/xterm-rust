# Parser parameter storage

`crates/xterm-core/src/parser/params.rs` ports `source_repo/src/common/parser/Params.ts`. The public path is `xterm_core::parser::params`. This module has no runtime dependencies, browser calls, or unsafe code.

## API mapping

| TypeScript | Rust |
| --- | --- |
| `new Params()` | `Params::default()` |
| `new Params(maxLength, maxSubParamsLength)` | `Params::with_capacities` returning `Result` |
| `Params.fromArray` | `Params::from_array` |
| `ParamsArray` | `Vec<ParamValue>`, enum variants `Param(i32)` and `SubParams(Vec<i32>)` |
| `params`, `length` | `values()`, `len()`, `is_empty()` |
| `maxLength`, `maxSubParamsLength` | `capacity()`, `sub_capacity()` |
| protected `_subParamsLength` | `sub_len()` |
| `clone`, `toArray` | `Clone::clone`, `to_array` |
| `reset`, `resetZdm` | `reset`, `reset_zdm` |
| `addParam`, `addSubParam`, `addDigit` | `add_param`, `add_sub_param`, `add_digit` |
| `hasSubParams`, `getSubParams` | `has_sub_params`, `get_sub_params` |
| `getSubParamsAll` | `get_sub_params_all`, owned `BTreeMap<usize, Vec<i32>>` |

Mutating additions and construction return `Result` rather than throwing. Values below -1 fail with the original error text, unless capacity rejection happens first, matching the source ordering. Values above `i32::MAX` clamp. Rust uses integral `i64` input for additions and `u8` decimal digits. JavaScript fractional numbers, NaN, and infinities are outside this typed API. Browser binding conversion, if needed, must handle those separately. Passing digits above 9 returns an error instead of invoking the source's unchecked internal helper.

Storage capacity is private and cannot be mutated into an invalid layout. `values()` exposes only active parameters. Access beyond active length returns `None`, rather than exposing stale backing storage. This follows the source's documented requirement never to read beyond `length - 1`; it does not claim equivalence for contract-violating reads. Negative JS indices have no Rust `usize` representation.

`reset_zdm` on zero parameter capacity returns `ZeroCapacityZdm`. The JS source reports a logical length of one while its zero-sized typed array silently ignores the write, producing `undefined` values. Rust does not represent this invalid layout. Normal parser construction uses nonzero capacity. This boundary mapping must remain visible to bindings rather than being advertised as identical behavior for arbitrary JavaScript inputs.

Packed subparameter offsets deliberately retain the source's eight-bit endpoint behavior at 256 entries. The 256th append can cause subparameters to disappear from `to_array`. This surprising behavior is covered, not corrected. Rejected parameter/subparameter digits retain source rejection state until reset; clone retains that state as well.

## Verification

The 20 upstream `it` cases in `Params.test.ts` map to 20 tests in `crates/xterm-core/tests/params.rs`, with one additional boundary test. Tests retain every upstream expected result, including the two issue-2389 regressions, clamp behavior, clone state, defaults and limits. The test parser shim iterates bytes directly rather than reproducing the source test shim's awkward UTF-16 loop. Inputs in these cases are all ASCII.

`check-params.mjs` executes the unchanged original class after Node's TypeScript enum transformation and removal of its type-only import. It does not replace method bodies or assertions. A fixed xorshift seed of `0x2389abcd` generates 512 operation traces with 96 steps each. Capacities include zero subparameter capacity, one, three, and the 255/256 boundary. Operations cover valid/invalid direct additions, clamping, digit accumulation, reset, zero-default reset and clone. A separate 257-step trace reaches all 256 subparameters. Every operation records a value snapshot and error outcome. Total compared snapshots are 49,409. Committed fixtures make `cargo test` independent of Node and the source tree. The fixture is generated data, intentionally longer than 350 lines.

Run from the repository root with Node 24 or later and the workspace Rust toolchain:

```text
node scripts/check-params.mjs
cargo test -p xterm-core --test params --test params_differential
cargo clippy -p xterm-core --test params --test params_differential -- -D warnings
```

Use `node scripts/check-params.mjs --generate` only to regenerate after reviewing an upstream source change. The normal command asserts exact corpus equality and records source/corpus SHA-256 hashes, seed and counts in `results/params/oracle.json`. A mismatch retains its case and step in Rust assertion output and the full operation history in the TSV corpus. Test command output is in `results/params/rust-tests.txt`.

Observed here on native Windows, Rust 1.85.1, Node v24.15.0: all 22 Rust tests passed, including all 49,409 differential snapshots. `node scripts/params-baseline.mjs` also passed all 20 original upstream tests with actual Mocha 11.7.5 and Chai 6.2.2. It transforms TypeScript syntax, removes type-only imports and remaps runtime import URLs without editing assertions. Counts and hashes are in `results/params/original.json`; full output is in `results/params/original-tests.txt`. To reproduce dependencies without touching the reference, run `npm install --prefix tmp/params-deps --no-package-lock --no-audit --no-fund mocha@11.7.5 chai@6.2.2`, then set `PARAMS_TEST_DEPS=tmp/params-deps/node_modules` before running the script. If unset, it reuses `tmp/color-oracle/node_modules` created by the Color verification task.

## Focused acceptance follow-up

`tests/params_edges.rs` adds seven deterministic tests for the P1-P4 contracts in `docs/criteria/parser-storage.md`: exact default/zero capacities, owned snapshot independence, multiple leading arrays and capacity truncation, rejection-before-validation, cloned rejection state, clone continuation, validation-error digit targeting and exact saturation boundaries. `parser_table.rs` also checks table clone independence and every untouched entry after a multi-write.

The combined Params/table suite passed 35 tests in both debug and release, with warning-free targeted Clippy and rustfmt. Evidence is `results/params/edges.json` and sibling `edges-*.txt` logs. After the run-2 interruption, the debug suite passed again under `.cargo/config.toml` with debug information disabled and incremental compilation off; those settings change build artifacts, not assertions.

These checks cover Params and transition tables only. They do not establish whole-terminal equivalence, native Linux/macOS results, browser behavior, or the behavior of arbitrary JavaScript numeric values.
