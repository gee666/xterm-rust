# Public API oracle review

Reviewer Jeremiah. Task #120 reviews the oracle work introduced in commit `3195429`, not the completeness of the public terminal implementation. Reviewed files were `scripts/public-api-oracle.mjs`, `scripts/public-api-integration.mjs`, `scripts/check-headless-mapping.mjs`, `tests/public_api_validation.rs`, `tests/public_api_differential.rs`, `tests/public_api_integration.rs`, their generated fixtures, the two original public Terminal classes, `docs/public-api-acceptance.md` and the public-api-oracle skill. Paths under tests are relative to `crates/xterm-core/`.

## Independent execution

Native Windows x64, Node 24.15.0, Rust 1.85.1, repository debug/test profiles unchanged.

- `node scripts/public-api-oracle.mjs` exited 0. The 4,096 random IEEE-754 bit patterns plus 19 directed values produce 8,230 numeric rows. Six readonly-option rows and two proposed-API rows make 8,238. The 90,530 fake-core dispatch checks execute original wrapper methods. The 12,288 mode rows cover 2,048 combinations of eleven independent booleans and six protocol names. Both Rust differential tests passed. Independent combined output is retained as `results/reviews/jeremiah-public-numeric.log`.
- `node scripts/public-api-integration.mjs` exited 0. All five actual-terminal event-time tests passed on this rerun. The previously reported five failures are no longer current. The tests still assert source observations before the following print and a Bell-triggered reset leaving only B afterward. They have not been weakened. Independent output is retained as `results/reviews/jeremiah-public-integration.log`.
- `node scripts/check-headless-mapping.mjs` exited 0. All 45 original case names map to existing Rust test functions. It reported zero current native executions and 45 unrun because its prior source/log snapshot was stale. That is the correct distinction between a case-name mapping and executed evidence. Independent output is retained as `results/reviews/jeremiah-public-map.log`.

## Fidelity findings

The numeric oracle invokes methods from the original class bodies after type/import erasure. It does not rewrite the validation algorithm. IEEE-754 fixture bits preserve signed zero, NaNs and infinities. Browser decoration NaN/zero truthiness quirks and large integral values are retained. Normal verification rejects fixture drift rather than overwriting expected values. The mode oracle reads the complete source getter, including field order; Rust constructs its projection through real CoreService and MouseStateService values. The unknown CUSTOM protocol exercises the none fallback.

The integration oracle bundles the original headless terminal and captures state inside real callbacks. The Rust test observes the actual headless terminal, not a mock core. These five tests establish their particular event observation/reset traces only. They do not establish all event payloads, asynchronous writes, browser/WASM coercion or full PA01-PA06 behavior.

The PA matrix explicitly names omitted JS values, live options, write scheduling, real escape dispatch, browser lifecycle and composed clear/marker behavior. Numeric helper and fake-core dispatch success are not presented as engine equivalence. Source baseline cases are kept separate from migrated Rust cases.

## Evidence issues sent to the owner

1. The integration report records JavaScript source hashes but not Rust input hashes or a changed-during-run check. It cannot identify which Rust state produced a historical pass after the shared tree changes.
2. The numeric report hashes selected Rust files but omits CoreService and MouseStateService dependencies used by its snapshot check.
3. The headless mapping checker reads its native log unconditionally. A missing log throws instead of producing the promised unrun accounting. Its stale-log behavior was exercised and is correct.

Euphemia was asked to correct these evidence issues. No reviewed implementation, script or fixture was edited by this reviewer. Existing per-attempt failures remain retained in `results/public-api/`. This review does not accept the public wrapper implementation or claim native Linux/macOS or browser equivalence. Overall equivalence remains NOT PROVEN.
