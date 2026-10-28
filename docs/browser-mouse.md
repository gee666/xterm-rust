# Browser mouse policy

Task 85 implements the pure calculations and report state from these unchanged
reference modules:

- `source_repo/src/browser/input/Mouse.ts`
- `source_repo/src/browser/services/MouseCoordsService.ts`
- `source_repo/src/browser/services/MouseService.ts`

Task 33 and its DOM subtask 110 own actual event listener installation, document
migration, focus/selection/CSS changes, pointer capture, touch listeners and
browser callbacks. Passing these policy tests is not browser integration proof.

## Rust contract

`input::mouse_coords::MouseGeometry` is a geometry snapshot, sampled from the
current element's owner window. It stores rectangle offsets, decimal-parseInt
padding, character-size validity, CSS cell dimensions and CSS canvas dimensions.
`relative_coords` subtracts rectangle and padding. `coords` returns one-based
cells, with the half-cell selection offset and extra selection end column.
`report_coords` returns clamped, floored pixels and zero-based cells. Invalid
character metrics return None. NaN and infinities are not silently normalized.

The DOM adapter must preserve sampling order. Source getCoords does not read
rectangle/style after finding invalid character size. Source
getMouseReportCoords samples relative coordinates before checking character
size. This matters for layout reads, errors and moved documents.

`input::mouse_wheel::WheelState` retains wheel fractions and touch accumulation.
It implements the original sensitivity multiplication order, all delta modes,
DPR/device-pixel conversion, threshold 50 and trackpad multiplier 0.3, negative
remainders, sign changes and missing/zero/nonfinite metrics. `touch_start` clears
only touch state. `reset` clears both accumulators. Touch returns a signed whole
line count, and `cursor_sequence` supplies the matching normal/application
cursor sequence. The adapter emits once per absolute touch line. Passive wheel
emits one cursor key regardless of its consumed magnitude, as source does.

`input::mouse_policy::MousePolicy` holds last-report and accumulator state with
short RefCell borrows. `route` consumes a PointerInput and optional report
coordinates. It applies source event-type override, button priority/fallback,
wheel veto and accumulation, local Alt gating and Alt stripping. `trigger`
performs range/combination checks, one-based conversion, grid/pixel move
comparison, source protocol restriction and exact UTF-16 encoding.

A successful trigger returns true even if the encoder suppresses output. That
still commits the last event for debouncing. Rejected events do not commit it.
Output failure preserves the prior last event. The output callback runs before
cache commit and without held policy state borrows, allowing nested reports.

Outputs distinguish Binary from UserData. UserData must call the real core data
event with wasUserInput=true. Binary uses the core binary event instead. Return
and propagate adapter errors; do not hide a callback failure behind success.
`MouseRouteError::Output` retains the original error and exposes its source.

MouseStateService remains the single shared protocol/encoding implementation.
No duplicate mouse encoder runs in the browser crate. Signed DOM button values
such as -1 use MouseButton::Raw and JS-style low-bit masking rather than being
clamped to NONE. Raw values equal to a named code compare by numeric value.

## Source and Rust assertion mapping

The original two Mouse getCoords cases map directly to upstream_clicked_cell
and upstream_viewport_bounds in `crates/xterm-web/tests/mouse_policy.rs`, with
all original input/expected coordinate assertions retained.

The original MouseService trigger cases NONE, X10, VT200, DRAG, ANY, three
coordinate-encoding loops, and eventCodes with modifiers execute unchanged in
the JS baseline. Every original _triggerMouseEvent invocation is captured,
including its pre/post event, result, full output and binary/data channel.
Rust replays all 1,562 invocations in original case order, resetting state at
the same case boundaries. This includes the Alt-strip case and report calls
from the moved-document case. Replaying those calls does not prove that Rust
installed or removed the original DOM listeners.

The original Alt strip assertion also maps to a direct route test in
`tests/mouse_routing.rs`. Additional routing tests cover all supported button
mask priorities, missing-buttons fallback, negative buttons, overrides, invalid
pairs, veto/zero/shift/missing-coordinate paths, empty-output caching, reset,
output errors, nested output and mutable custom encoders whose field changes
affect subsequent move debouncing. The latter case also runs as a separate
directed JavaScript assertion against the real source method. Source selection enable/disable, Alt cursor CSS
and moved-document listener assertions remain actual DOM work under 33/110.
They are not counted as migrated DOM assertions merely because the JS baseline
passed.

The source oracle uses seed 0xb0a5e and records these additional operation rows:

| Corpus | Rows | Compared observations |
| --- | ---: | --- |
| Original trigger calls | 1,562 | protocol/encoding, dimensions, full event before/after, accepted, output/channel |
| Geometry | 1,024 | padding text, bounds/metrics/validity, selection cells, report cells/pixels |
| Wheel | 2,522 | mode/flags/metrics/options, output amount and partial accumulator, resets |
| Touch | 516 | metric/translation/application mode, accumulator, every cursor output, resets |

Geometry includes NaN, infinities, signed zero, negative dimensions, fractions,
invalid/missing padding and exact ECMAScript whitespace cases. Wheel includes
all modifiers and modes, directed values immediately below/at threshold 50,
partial reversal, zero deltas, missing metrics and reset. Touch cases preserve
source invalid-height and signed-line behavior while keeping gesture counts
finite so the source's output loop terminates.

Native tests compare all output code units and event fields. Floating-point
comparisons treat NaN as NaN and otherwise compare bits, including negative zero.
The checked-in TSV files retain each input and complete expected result. A
mismatch reports its row, which is a reproducible retained counterexample.

## Reproduce

Prerequisites are Rust 1.85+, Node 24+, and dependencies installed in the
unchanged source reference using its lockfile. The source runner uses the
original esbuild and Mocha packages; no new production dependency was introduced.

```sh
node scripts/check-browser-mouse.mjs
```

The checker rebuilds the reference's generated JS using its existing build
script, runs all 15 original tests with assertions intact, verifies every
fixture, runs 12 native Rust tests, focused rustfmt, warning-free Clippy and a
WASM compilation check. `--generate` on `scripts/browser-mouse-oracle.mjs`
regenerates expected data intentionally. Inspect hashes and fixture differences
before accepting changes.

Every checker attempt has an immutable `results/browser-mouse/runs/<id>/`
receipt. `latest.json` points to its summary. Setup, command, timeout and warning
failures remain failures; later checks are marked unrun. Tool/platform versions,
source hashes, seed, counts, log paths and disk measurements are retained.
Builds use the shared nonincremental profile and target directory. No alternate
build tree is created.

Current evidence distinguishes observed native Windows behavior, WASM compile
results and unrun browser/Linux/macOS runtime checks. Failed shared integration
snapshots are retained rather than replaced with a historical green claim.
Chromium, Firefox and WebKit DOM checks belong to task 33/110 and project E2E.
Full terminal equivalence is NOT PROVEN.
