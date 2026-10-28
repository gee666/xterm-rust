# Touch clock and movement acceptance

This refines task 139's portable policy from `source_repo/src/browser/scrollable/touch.ts`. The Microsoft MIT notice must remain. Task 108 separately owns actual DOM listeners, targets, event fields, dispatch ordering, animation scheduling and real touch-to-scroll integration. A native policy pass is not a browser gesture pass. No original touch-specific test suite was found; original assertion counts must therefore not be invented.

## TP1. Start and rolling movement

- `TouchData::new` retains the exact initial target, timestamp and page coordinates, with one initial history sample.
- `last()` returns the most recently committed page coordinates. `translation(new_x,new_y)` subtracts that sample without changing it. DOM must dispatch movement before calling `commit_move`; preparing a movement must not eagerly mutate history.
- The rolling window contains four samples. On the fourth and later moves, discard exactly the oldest sample before appending. Initial coordinates/time used for hold/distance decisions remain unchanged after rolling samples are discarded.
- Independent records remain independent, including the old record retained while a reentrant callback replaces an active-map entry. The portable model must not require a map borrow during event dispatch.
- Cover zero/positive/negative/large/fractional coordinates, backward/equal/large/fractional timestamps, and nonfinite arithmetic. Do not clamp or substitute zero for NaN or Infinity.

## TP2. End classification and velocity

- A hold strictly shorter than 700ms with both absolute displacements strictly below 30 is TAP. At exactly 700ms and above, the same distance condition produces CONTEXT_MENU.
- Exactly 30px in either axis is movement, not a tap or hold. Use the last committed move position, not the touchend event's position. The initial-to-last distance remains independent of the rolling velocity window.
- Movement produces inertia computed from the last and first samples still in the four-sample window. Velocity is absolute displacement divided by timestamp difference. Each direction is positive only when the displacement is strictly positive; zero uses the source negative branch.
- Equal timestamps yield the source NaN/Infinity results. Backward timestamps yield the source signed velocity before later friction. Nonfinite hold values retain JavaScript comparison behavior.
- The end helper takes separate hold-query and inertia-start times because the source reads Date.now separately. It returns a decision without clearing or mutating the record. DOM uses its active-touch-count snapshot to decide whether an inertia result should actually be scheduled.

## TP3. Inertia and tap counter

- Each frame subtracts 0.005 times elapsed milliseconds from both velocities before computing displacement. A positive remaining velocity contributes direction times velocity times elapsed milliseconds. Otherwise that axis contributes exactly zero.
- Continue if either remaining velocity is positive. Emit the final zero-displacement frame before stopping. Preserve backward/equal/nonfinite timestamps and Infinity/NaN comparisons; do not add an arbitrary speed limit or retry loop.
- Inertia time comes from Date.now-compatible values, not the requestAnimationFrame timestamp. Cancellation and target snapshots are DOM task 108 responsibilities.
- Tap count is one only when currentTime minus lastTapTime is strictly greater than 400ms; otherwise two. Exactly 400ms and a first synthetic tap near timestamp zero produce two. Counts do not grow beyond two.
- CHANGE and CONTEXT_MENU reset the remembered tap timestamp to zero before ignored-target filtering. START/END do not reset it. Inertial CHANGE bypasses ordinary dispatch and does not reset tap time. The portable tap helper supplies tap/reset operations; task 108 must place them correctly.

## TP4. Source oracle and integration evidence

- Freshly transform and execute the unchanged supplied Gesture class. Mock only its DOM/clock/scheduler environment, not its movement, hold, tap or inertia method bodies. Record the source hash and explain every environment substitution.
- Persist directed boundary cases and seeded traces. Compare Rust and JavaScript per-operation last coordinates, movement deltas, end classification, velocity-derived frames and tap counts, retaining NaN, infinities and negative zero as distinct encoded values.
- Normal verification checks existing fixtures; regeneration is explicit. A mismatch fails and retains seed/case/step/input plus both values. Do not regenerate to accept a failing implementation.
- Run native debug/release tests, targeted fmt and warning-free Clippy. Record tool/platform versions and exact counts. Shared compilation failures remain failures, never inferred passes.
- Verify task 108 consumes these exact helpers with real DOM targets and owned requestAnimationFrame scheduling. Its browser tests must check field presence/flags, nested/ignored/duplicate targets, mutation and reentry, cancellation/disposal, touch capability, actual scrolling and all supported engines. Native-only evidence cannot close those browser gates.
