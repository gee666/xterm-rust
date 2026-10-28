# Parser storage acceptance criteria

These are behavioral gates, not a claim that testing proves mathematical equivalence. The reference is the unchanged xterm.js 6.0.0 tree. `PORT_REQUIREMENTS.md` remains controlling. Every failing assertion must fail its gate; no ignored tests, weakened expectations or swallowed oracle mismatches count as evidence. Missing evidence is unrun, never passed.

## P1. Construction, representation and borrowed reads

- Default construction allocates capacity for 32 parameters and 32 subparameters, with zero active lengths and no entries in `to_array`.
- Explicit capacities are respected, including zero, one, three, twelve, 255 and 256 as applicable. Subparameter capacity greater than 256 returns the source error text. Construction must not panic on these boundary inputs.
- Array conversion preserves ordered parameters and their attached subparameter arrays, drops empty arrays, skips the first leading subparameter array, and honors default capacities rather than growing past them.
- `values()` exposes exactly the active parameters. `get_sub_params(index)` borrows the exact attached values; `has_sub_params` agrees with presence. `get_sub_params_all` returns independent owned copies indexed by parameter position.
- Access outside active length returns `None`. Negative indices cannot be represented by Rust `usize`. Stale underlying source typed-array storage is outside the source's documented active-range contract and must not leak through Rust borrowed access.
- Rust `ParamValue` must preserve scalar versus subparameter-array identity and ordering. Mapping is documented in `docs/params.md`.

Evidence: all construction, array round-trip, getter and clone assertions in original `Params.test.ts`; Rust `params.rs` tests with matching expectations; capacity and getter checks in the seeded corpus; focused added boundary tests where originals do not cover the boundary.

## P2. Parameter mutation, bounds and rejection state

- `add_param(-1)` and subparameter -1 store the default sentinel. Values below -1 fail with the source error text only when the source would validate them; capacity rejection and subparameters without a parent take precedence over validation.
- Direct integral additions at `i32::MAX`, `i32::MAX+1`, and the documented larger `i64` boundary clamp to `i32::MAX` without negative wraparound.
- Decimal accumulation replaces a -1 sentinel with the first digit, appends subsequent digits by multiply-by-ten, and saturates at `i32::MAX`. Digit accumulation with no current parameter is a no-op because that is the source behavior, not a missing implementation.
- Once parameter capacity is exhausted, later parameter digits and subparameter additions must not change accepted storage. Once subparameter capacity is exhausted, later subparameter digits must not change the last accepted subparameter.
- Switching back to a new parameter must use parameter digits, not the prior subparameter target. Rejection state must remain active for exactly the source lifetime; reset clears it.
- Subparameters cannot exist without a preceding parameter. They attach to the latest accepted parent and share one bounded subparameter capacity across the sequence.
- All original length-restriction cases and issue-2389 regressions retain their exact expected arrays, including the 32-parent overflow sequence and a subparameter split across chunks.

Evidence: original `addParam`, `addSubParam`, length, invalid-value, clamp and issue-2389 cases; Rust ports; explicit small-capacity rejection tests and all intermediate seeded oracle snapshots.

## P3. Reset, clone and packed-offset boundaries

- `reset` empties both logical stores, clears both rejection flags, and resets digit targeting. Subsequent operations must behave like a fresh object with the same capacities.
- `reset_zdm` restores one zero parameter, no subparameters, cleared rejection state and parameter digit targeting.
- Clone preserves capacities, logical values, packed offsets and all digit/rejection state. Mutation of a clone must not mutate the original, and continuing digits after cloning must match the source clone.
- Packed subparameter start/end offsets retain the original 16-bit storage behavior. The 255th and 256th insertions and a following parent insertion match fresh upstream snapshots, including the source's disappearing-subparameter boundary result. Do not silently repair the source protocol behavior.
- Zero-capacity `reset_zdm` returns the documented `ZeroCapacityZdm` error and leaves Rust storage valid. This is an explicit typed boundary difference from JS's invalid logical length and undefined element. It must not be advertised as byte-for-byte equivalence for arbitrary JS calls.

Evidence: original reset/clone assertions, focused Rust continuation/ownership tests, the 257-step 256-subparameter oracle trace and intermediate snapshots.

## P4. Original suite, differential reproducibility and numeric binding boundary

- Run all 20 original `Params.test.ts` tests with their original assertions and real Mocha/Chai. Record discovered, executed, passed, failed, skipped and unrun counts, source/test hashes, runtime and dependency versions.
- Port every original expected result, not merely the test names. Run migrated assertions against Rust. Keep the source unchanged.
- Execute seed `0x2389abcd`, 512 traces of 96 operations plus the 257-step boundary trace. Compare all 49,409 intermediate arrays and error outcomes; a mismatch fails verification and names case, step and full trace. Do not replace fixtures merely to accept a mismatch.
- Fresh upstream execution must match both committed fixtures and Rust outputs. Retain source and corpus hashes and the full trace corpus.
- Typed Rust additions use integral `i64`; digits are validated `u8` decimal values. Document this rather than pretending arbitrary JS Number behavior was proven. Browser bindings must not coerce NaN, infinities, fractional values or invalid digits through unchecked Rust casts. Full browser equivalence requires separate binding-layer coverage of the original JS numeric contract.
- Rerun targeted rustfmt, Clippy with `-D warnings`, debug tests and release tests on the integrated crate. Record shared-build failures explicitly; a module-level green check is not a green workspace claim.

Evidence: `scripts/params-baseline.mjs`, `scripts/check-params.mjs`, `tests/params*.rs`, `results/params/`, `docs/params.md`. Browser boundary coverage belongs to the browser binding task, not the native Params result.

## T1. Numeric state/action constants and packed representation

- All 17 parser states retain exact original numeric discriminants 0 through 16. State count is 17 and must not be treated as a valid state.
- All 18 actions retain discriminants 0 through 17. Action/state conversion rejects every other `u8` value.
- A transition packs action in the high byte and next state in the low byte. Every valid action/state pair round-trips. Invalid packed action/state values return `None`, not an invented transition.
- `NON_ASCII_PRINTABLE` is 0xA0. `PAYLOAD_LIMIT` is 10,000,000. These protocol constants are not runtime configuration.
- No unsafe enum transmutation or unchecked indexing is needed for decoding an untrusted packed value.

Evidence: `typed_constants_and_packed_roundtrip`, exhaustive enumeration in `parser_table.rs`, original `Constants.ts` hash and oracle execution.

## T2. Complete VT500 rules and Unicode classification

- Every one of the original 4257 packed table entries matches fresh execution of the unchanged upstream initializer, not a manually selected subset.
- Look up all 17 states across all 256 raw byte values. Existing entries decode to the exact oracle action/state. Out-of-allocation slots return `None` rather than panic.
- All codepoints at or above 0xA0 use the source pseudo-character slot, including U+00FF, U+0100, astral input, U+10FFFF and arbitrary `u32` input. Values below 0xA0 retain their exact byte slot in every state.
- Ground printable/executable behavior, global ESC/C1 rules, CSI entry/params/intermediates/ignore, OSC termination, DCS hook/passthrough/unhook, APC entry/intermediates/passthrough/end and SOS/PM ignoring are all present in the full oracle comparison.
- Representative named assertions must also confirm CSI SGR dispatch, OSC BEL termination and DCS/APC ESC termination, so a generator accidentally emitting unrelated but internally consistent data is visible.
- The production Rust parser uses native Rust table data, not the JavaScript engine. The main escape parser must consume this shared table rather than maintain a divergent duplicate.

Evidence: `scripts/vt500-oracle.mjs`, `exhaustive_vt500_oracle`, `unicode_classification_and_known_transitions`, `results/vt500/oracle.json`; main-parser integration evidence is owned by task 26.

## T3. Custom-table mutation, regeneration and evidence

- A custom table of requested length initially contains zero, meaning Ignore/Ground. Default replacement changes every allocated entry. Untouched entries retain prior values.
- Preserve the source raw-table path. The parser retains its supplied TransitionTable object and observes changes through a retained alias, including changes made by a callback before the next lookup. A deep snapshot must not masquerade as that alias.
- Source raw Uint16Array writes use JavaScript numeric/index coercions, ignore out-of-range assignments and preserve earlier writes in a multi-code operation. Checked/atomic Rust convenience helpers may coexist only with explicit names/contracts; they do not prove the raw source behavior.
- Unknown packed actions perform no action while retaining their next-state byte. Unknown states remain observable until the next source lookup. Missing entries coerce to packed zero. String-end ESC combines the transition with the source bitwise OR, including an otherwise unknown resulting state.
- Raw custom PARAM actions must preserve source signed arithmetic and Int32 assignment behavior, including code32 accumulating -16, instead of silently routing through a decimal-only validation error. Test source Number boundaries and state/index coercions supported by the raw interface.
- A retained raw backing-array view has its original identity. If the table's backing array is replaced, subsequent parser lookups use the replacement while the old retained array remains detached from that table object. Do not keep two synchronized authoritative copies.
- Independent newly constructed tables and the immutable generated default do not share mutation. Any deep-copy helper remains explicitly distinct from a shared table handle; document the Rust representation rather than inventing a source clone API.
- Normal oracle verification fails on any Rust-data or test-fixture drift. Generation is explicit and cannot edit the reference. Preserve MIT attribution, source/constants hashes, exact raw mutation traces, 4257-entry data checks, scoped quality logs and failing cases.
- The generated fixture length exception covers data only. Neither the fixed VT500 oracle nor the original checked/atomic helper tests establish raw custom-table equivalence. Task 178 is open until the coherent raw contract and its differential tests pass. No whole-terminal equivalence claim follows from these checks.

Evidence: existing VT500/custom checked-helper tests are retained as their original subset; task178 adds actual source raw-table/alias/coercion/unknown-action-state/PARAM assertions and updates `docs/parser-table.md` with the implemented mapping.
