# HTML raw row subtraction error phase

Task556 repairs HTML explicit-range subtraction errors within still-open346 and322. Overall equivalence is NOT PROVEN.

The original BaseSerializeHandler evaluates endRow - startRow before calling HTMLSerializeHandler._beforeSerialize. HTML does not use the difference, but JavaScript still converts end first, then start, with the number hint. A Symbol, mixed BigInt/Number or nonprimitive conversion result can throw. User conversions can throw their original value, replace the next conversion, reset/resize or invoke a nested serializer. Those effects precede the HTML header and first cell-column conversion. Coordinate property reads still occur before subtraction.

The Rust browser adapter previously skipped this operation and converted the column or start coordinate first. It now reuses the caught host subtraction helper from accepted549. Numeric primitives need no host call because their subtraction cannot invoke a callback or throw. Serialization itself remains in Rust. No dependency or unsafe code is introduced.

The shared scripts/serialize-html-difference-cases.mjs fixture adds three groups with20 subcases to the existing93-case union. It covers primitive conversion getters/calls/valueOf/toString, actual engine errors, thrown identity, untouched column/header callbacks, reset/resize, nested serialization and replacement of the next operand. The runner evaluates exactly the same function against original and Rust terminals, keeping the original assertions intact.

Successful arbitrary-object/BigInt traversal still requires source comparisons, raw ring lookup, endpoint identity, increments and later errors at their original phases. This repair deliberately does not claim that converting raw coordinates once reproduces those operations. Full host replacement, palette identity, GC, family coverage, platform quality, comparisons and complete-suite proof remain open in their existing tasks. Equal infinite row bounds can hang the source and are not run.

Verification results will be added after execution. New temporary paths are owned by Bardiya under O:/xterm-rust-swarm-16/tmp/worktrees/BardiyaHtml556. Canonical main is not modified here. Accepted549 is privately copied only as a prerequisite; those four copied commits must be excluded from556 pickup.
