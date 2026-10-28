# Task155 source review preflight

This is source inspection only, not task acceptance. The new legacy Rust replay and immutable differential rerun remain unexecuted by Victor while the resource guard is below1GiB.

Inspected `keyboard.rs`, `keyboard_policy.rs`, `keyboard_default.rs`, the shared getter Source helper and original `source_repo/src/common/input/Keyboard.ts`. Reviewed the documented uppercase-data generation and getter corpus method. The current structural split keeps public native signatures and shares actual policy rather than duplicating it for live events.

Observed alignment:

- Saved modifier reads are Shift/Alt/Control/Meta before one switch keyCode read. Only bounded integral f64 values enter special cases; fractions/nonfinite values retain default-branch behavior.
- Mobile arrow comparisons read key separately in the source order. Backspace, Tab, Enter, arrows, Insert and Page branches preserve conditional field reads. F1-F4 SS3 and modified CSI routing retain their original forms.
- The default Control path repeats keyCode at each range test and final arithmetic. ToUint16 occurs after arithmetic, not when reading the field. Alt mapping reads Shift only when a mapping exists, matching optional-index short-circuiting.
- Printable fallback retains separate key truthiness/length/payload reads. The final payload can differ from the tested key. Ctrl+Shift code selection keeps its own late code read.
- Dead-key code.slice occurs before the later Shift getter. The copied unit survives a code mutation by that getter. Empty slices still produce an ESC key with cancel=true rather than absence.
- The changed final Alt-letter code can be non-ASCII, so using pinned full Unicode uppercase data is necessary; Rust's compiler Unicode version or ASCII-only uppercase would not match the source.
- Error propagation uses the generic source error directly. No callback is called while a Source mock RefCell borrow is held. Native borrowed adapters use Infallible instead of a panic path.

No source defect found in the declared primitive-field domain inspected. This does not prove all branches or table data. Required before acceptance: execute the47,598-row Rust getter replay, original61 tests/156 assertions,251,452 immutable comparisons, prior Rust tests and warning-denied checks, then inspect exact logs and hashes. Actual DOM getter and engine integration belong to151/33/141. Inherited object maps and dynamically decorated values remain separately tracked and must not be hidden by this review.
