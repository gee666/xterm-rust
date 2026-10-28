# Source inspection finding: tab width values

This finding is from source inspection while local builds are paused, not an executed test result.

`source_repo/src/common/services/OptionsService.ts` checks `tabStopWidth` with `value < 1` but retains the original value. It does not floor, clamp or require a finite number. `source_repo/src/common/buffer/Buffer.ts` then executes `i += rawOptions.tabStopWidth` at every loop iteration. The source creates JS object property keys from the resulting i values.

The current Rust `screen_navigation.rs` calls `positive_option` once, requires a finite integer and stores only usize stops. This cannot match all source-accepted values. NaN and positive infinity pass the source option check and terminate the loop after its first stop. Fractional1.5 creates fractional keys. A retained string8 produces string concatenation, not numeric addition. Positive objects can observe repeated default-hint coercion at the source addition and mutate options or terminal state. Rust currently coerces once using the Number hint while holding a mutable Buffer borrow.

Required checks must include widths NaN, positive infinity,1.5,2,8, a retained numeric string and an object with observed primitive conversion. Preserve exact property keys, next/previous-stop behavior, callback order and accepted/rejected option values. Native usize callers must keep their stable API through an explicit richer internal representation or additive JS-number lookup API, not unchecked casts. Public browser calls must retain actual accepted source behavior.

Zero and negative widths rejected by the normal option setter must remain rejected there. Do not run an unbounded source raw-width-zero loop to pretend it passed. If testing deliberately invalid raw mutation, bound the oracle externally and record nontermination as such.

No successful workaround, lint suppression or fabricated equivalence result closes this finding. Add source/Rust differential evidence and an independent review before claiming the tab-stop criterion complete.
