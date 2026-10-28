# Independent Buffer implementation review

Blake reviewed task 27's Buffer screen state, navigation, options, resize/reflow orchestration, marker wiring and host cleanup adapter against the full read-only `common/buffer/Buffer.ts`. Shared reflow helpers were separately reviewed under task 62.

Observed independent checks on Windows:

- The full source baseline passed 61 Buffer cases, with unchanged source assertions.
- The Buffer differential replay passed 1,111 transitions. It includes 86 original resize calls, 1,024 seeded calls and a directed empty-wrapped-row regression.
- The 22 direct source assertion tests and 10 additional Buffer edge tests passed.
- Exact retained backing-memory byte assertions passed.
- These 34 Rust test functions passed in debug and release in `results/buffer/runs/2026-10-28T00-08-37.407Z/`.
- Eleven host/Buffer contract tests passed independently in `blake-host-and-queue-tests.log`. These drive actual TaskQueue scheduling, idle/priority cancellation, 101-row cleanup batches, explicit borrow recovery, final-owner cleanup and typed scheduling-error identity.
- Five raw Windows PTY option tests passed independently in `blake-windows-options.log`. Reviewed truthiness before numeric coercion, repeated build getters, short-circuit ordering, per-row presence checks and getter failure propagation against the source's cached-object reflow getter and live row-growth checks.

Review and the random corpus found a real defect. An empty wrapped group can produce negative source `linesToAdd` and `savedY`. The initial usize checked subtraction returned SizeOverflow. The owner now preserves the signed adjustment and i64 saved Y; the retained failing seed and directed source case pass without weakening source assertions.

Reviewed defaults and saved state, host dimensions versus last internal resize, null/space cell attributes, shared extended-attribute identity, wrapped ranges, text slicing, viewport/history growth and shrink, cursor clamping, all source PTY gates, wide/combined copy behavior, removal/insertion/trim ordering, marker invalidation, tab stops, cleanup batching/rescan/cancellation and typed error propagation. No unsafe code was introduced in these modules. The native owned scratch-cell result is documented rather than presented as JavaScript scratch-object identity.

The initial custom marker reentry edge expected an empty collection, which was not source behavior. An actual JavaScript supplemental oracle confirmed that disposal of a sibling changes its line to -1 and leaves its disposed entry in the collection. Rust now asserts the observed behavior. No original source assertion changed.

This is acceptance of the demonstrated base Buffer component. Same-buffer callback read/mutation across shared service resize/scroll checkpoints is explicitly separate task 122 and task 130. The owner began that refactor after the passing component checkpoints; its current unfinished working-tree compilation is not represented as a passing final build here. Full parser/addon/native-host integration is task 99. Workspace Clippy/fmt failures and source drift were retained in the run reports, not hidden. Neither component acceptance nor these Windows tests prove full terminal equivalence, browser behavior or native Linux/macOS execution.
