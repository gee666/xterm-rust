# String builder acceptance contract

This is the testable contract for both classes in
`source_repo/src/common/StringBuilder.ts`. It supplements the broader workspace
criteria. Every row below is required, not a selection of convenient examples.
A passing existing test does not establish an untested row. Reference source and
assertions remain unchanged. Full terminal equivalence is still NOT PROVEN.

Criteria 145 through 152 map in order to SB-01 through SB-08 below. Task 75
closes the remaining implementation and proof gaps; initial port task 16 is not
proof that these expanded checks pass.

## SB-01: empty state, observation, reset and reuse

For each builder, immediately after construction length is zero and the returned
UTF-16 sequence is empty. Repeated reads do not mutate data. Repeated reset on
empty and nonempty instances clears length and contents without changing the
configured limit. After reset, appending must behave like a fresh instance.

Required operation traces:

- construct, observe twice, reset twice, observe;
- append `old`, reset, append `new`, observe `new` with length 3;
- append nonempty, reset, append empty chunks, observe empty;
- mutate after retaining an owned output snapshot; the snapshot must not change.

Both valid UTF-8 convenience input and raw UTF-16 input must follow this contract.
Borrowed Rust observations may prohibit mutation until the borrow ends. That is
an ownership representation, not permission to change output values.

## SB-02: ordered, lossless UTF-16 accumulation

`length` counts JavaScript UTF-16 code units, not UTF-8 bytes, graphemes or Unicode
scalars. `append` concatenates units in input order without normalization or
replacement. Empty chunks contribute zero length and no units.

Required checks:

- append `foo`, `bar`, `baz`, expect `foobarbaz`, length 9;
- append empty, `a`, empty, expect `a`, length 1;
- append `A😀\0e\u0301`, expect the same six UTF-16 units;
- exhaust every single unit from 0x0000 through 0xffff, including isolated high
  and low surrogates, NUL, BOM and noncharacters;
- split a valid surrogate pair before and after either unit; one joined output
  must equal the unsplit output;
- high/high/low, low/high, low/low, and high/NUL/low sequences must retain exactly
  their units, including across empty chunks and reset boundaries;
- checked UTF-8 conversion succeeds for valid pairs and returns an error, never
  replacement text, for an isolated surrogate. Exact UTF-16 observation remains
  available after that error.

## SB-03: strict payload-limit comparison

For a nonnegative integral limit L, an append succeeds exactly when the resulting
UTF-16 length is at most L. The return value is false on success. Rejection is
strictly greater than L, not greater than or equal. Limit metadata remains fixed.

For L in 0, 1, 2, 3, 5 and 100, test each reachable length L-1, L and L+1 with
both a single chunk and multiple chunks. Test empty appends below and at the
limit. At L=0, empty succeeds and every nonempty append rejects. A two-unit
astral scalar fits L=2 and rejects L=1 even though it is one Unicode scalar.
Raw UTF-16 and valid UTF-8 helpers must agree on equivalent units.

## SB-04: overflow clearing and recovery

A rejecting append returns true and clears the entire accumulated payload,
including all previously accepted chunks. The rejected chunk is not retained.

Required traces:

- L=5, append `abc`, append `def`, expect true and empty length 0;
- L=3, append `toolong`, expect true and empty;
- after breach, append `ab` without an explicit reset, expect success and `ab`;
- repeat breach twice, then append exactly L units, expect success;
- reset after a breach and append again, with no retained overflow flag;
- L=100, accept 100 one-unit chunks, reject the next unit, then accept an empty
  append and a new nonempty payload;
- a rejected chunk containing lone surrogates must not leak those units into the
  next payload.

Preflighting a rejected chunk to avoid allocating it is allowed only if the
return value, cleared state and all later observations remain identical.

## SB-05: numeric constructor boundaries and representation

The source constructor accepts a JavaScript number and stores it unchanged.
Its append comparison is `length > limit`. Check source behavior explicitly for
negative zero, zero, positive integers, positive fractions, negative fractions,
negative integers, NaN and both infinities. Required expected outcomes include:

| Limit | Operation | Expected |
| --- | --- | --- |
| -0 | append empty, then one unit | false, then true and empty |
| 1.5 | append one unit, then another | false, then true and empty |
| -0.5 or -1 | append empty | true and empty |
| NaN or positive infinity | append arbitrary finite payload | false, retained |
| negative infinity | append empty | true and empty |

The exact configured number, including negative zero and NaN, remains observable
through an equivalent binding or a documented Rust representation. Rust integer
limits must not wrap at usize::MAX. No truncation, clamping or sentinel conversion
may silently erase source cases. If the Rust API intentionally admits only a
subset, list that gap and do not mark this criterion satisfied. This is a known
coverage gap in the initial usize-only port, not an external blocker.

## SB-06: source assertion migration and full-state differential

Execute every discovered upstream test body with assertions unchanged, except
module and assertion-library imports in the temporary runner. All 17 current
cases must map one-to-one to Rust assertions with the same input values and
expected outcomes. A change in upstream discovery count requires review, not an
updated hardcoded passing count alone.

Use xorshift32 seed 0x51b01d, at least 264 traces and 16,480 operations, and the
additional directed cases above. Every operation compares the overflow boolean,
UTF-16 length and every output code unit, including after reset and rejection.
Include empty chunks, split pairs, isolated surrogates and all supported numeric
limit classes. Save the seed, input operations and complete first failing state.
A stored oracle is not current-source evidence unless regeneration/checking also
verifies source and test SHA-256 hashes. No empty fixture or skipped loop passes.

## SB-07: allocation and growth behavior

Appending 10,000 one-unit chunks yields exactly 10,000 units without reconstructing
the entire accumulated output on every append. Code inspection must establish
amortized linear accumulation. Observe at the end and repeatedly without changing
state. Do not assert timing thresholds as correctness checks. Rejected huge
chunks must not require materializing a second copy solely to decide the limit.
Rust allocation failure remains the language/runtime boundary; do not promise
successful allocations larger than the process can represent.

## SB-08: integration, quality and evidence

The workspace exports one shared builder implementation. OSC/DCS/APC consumers
must use it or document a behavior-specific reason not to. No copied competing
limit helper or successful placeholder counts as integration. Callback payloads
must preserve exact UTF-16 and remain valid for the documented borrow lifetime.

Run source oracle checks, all scoped Rust tests, rustfmt and warning-free Clippy.
Record exact commands, tool/platform versions, source and fixture hashes, test
names/counts, seeds, failures and independent review findings under
`results/string-builder/`. Native Windows results are not Linux/macOS results.
Workspace and cross-platform integration checks remain required under their own
criteria. Component acceptance does not establish whole-terminal equivalence.

## Existing evidence and outstanding work

- Initial implementation: task 16, `crates/xterm-core/src/string_builder.rs`.
- Existing source cases: `tests/string_builder_upstream.rs`, all 17 current cases.
- Existing differential/edge cases: `tests/string_builder_differential.rs`, four
  tests and the 264-trace corpus.
- Existing evidence: `results/string-builder/{oracle,verification}.json` and logs.
- Followup task 75 implements all single-unit checks, every partition of the
  directed surrogate sequences, boundary/overflow recovery matrices and exact
  JavaScript numeric limits. Current corpus is 278 traces and 16,648 operations.
- OSC and DCS import the shared builder; APC aliases the OSC adapter. The scoped
  parser integration run exercises 48 tests covering exact-limit/overflow,
  lone-surrogate and sync/async payload handling. It does not replace full
  terminal/browser integration requirements.
