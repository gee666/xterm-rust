# Native mouse registry name ordering

Task 501 repairs the native `MouseStateService::protocol_names` and `encoding_names` ordering. These Rust inspection methods represent `Object.keys` on the source's private protocol and encoding registries. They are not new upstream browser methods.

Canonical array-index names sort numerically before other names. The accepted indices are decimal strings from `0` through `4294967294`. `4294967295`, leading zeros, signs, fractions, exponent notation, whitespace and non-ASCII digits remain ordinary names in insertion order. Replacing a registration does not move it. Reset and disposal retain registrations.

The crate-private `object_keys::ordered_names` helper reuses the existing UnicodeService algorithm. `UnicodeService::versions` now calls that helper without changing its contract. Mouse lookup storage and callback ownership are unchanged. Sorting occurs only when returning a name list, not when adding or selecting registrations. No dependency, unsafe code, browser registry or second mode authority was added.

## Evidence

Base `5088f8fdd475e55b2408e9f2e7bfaf92e83ca656`, native Windows, Rust/Cargo 1.85.1 and Node 24.15.0. Raw task results are in `results/mouse-registry-names/`.

- `node scripts/mouse-state-oracle.mjs --generate`, followed by the default drift check, executed eight unchanged source tests and retained the existing 12,226 encoding/restriction traces at seed `0x6d0a5e` with unchanged corpus hash. The new name fixture contains three sequences, 178 registration operations and 187 snapshots for each registry, including 128 seeded operations at seed `0x51a7e501`. Source and assertions were not edited. Fixture generation is intentional only after inspecting source changes.
- `cargo test --locked -j2 -p xterm-core --all-targets --all-features` passed 1,571 tests across 248 summaries, with zero failures, ignored tests or filtered tests. This includes the new three source-fixture tests, two helper unit tests and existing Unicode regressions.
- `node scripts/check-mouse-state.mjs` passed 29 mouse integration tests in debug and 29 in release, source fixture drift checks, both existing source oracles and warning-denied focused Clippy. Its immutable attempt is `results/mouse-state/runs/2026-11-05T22-06-47-514Z-750500bf-62d2-4212-9908-84c5e10f0be0/`.
- Restoring the exact base `mouse_state.rs` while keeping the new tests produced three runtime assertion failures, not compilation failures. A `finally` restored the complete fixed file byte-for-byte. The final three tests passed. See `negative-old.log`, `negative-driver.log`, `restoration.json` and `final-registry.log`.
- `cargo test --locked -j2 --release -p xterm-core --test unicode_service --test unicode_service_differential` passed 20 tests. This includes existing key-order, registry-operation and seeded UTF-16 assertions.
- `cargo clippy --locked -j2 --workspace --all-targets --all-features -- -D warnings` passed on the native host. `cargo clippy --locked -j2 --target wasm32-unknown-unknown --workspace --all-features --lib -- -D warnings` passed. These are compile/lint checks, not browser runtime evidence.
- Exact `cargo fmt --all -- --check` failed with Windows OS206 argument length. Separate `cargo fmt -p PACKAGE -- --check` passed for all five workspace members. This is not a full-workspace formatting pass.
- All 776 manifest files in both the original reference and private physical copy matched before the checks. After-check integrity and independent review receipts are retained alongside the results.

All heavy work used one granted serial Cargo `-j2` lane with an owned O: target and TEMP/TMP. No browser job or main-ref operation was performed.

## Remaining gaps

This is own-string registration ordering only. It does not implement JavaScript prototype mutation through `__proto__`, inherited registry lookup, symbols or raw object keys, lossless arbitrary UTF-16 native names, raw callback return values, borrowed receivers, browser method replacement or the missing complete live mouse registry facade. Task 305 and [mouse registry gaps](mouse-registry-gaps.md) retain those obligations.

Full workspace runtime tests, native Linux/macOS, complete source/browser/demo suites and overall port equivalence are not established by this task. Full equivalence remains NOT PROVEN.
