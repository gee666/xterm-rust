# Private DSR raw option callbacks

The source InputHandler.deviceStatusPrivate handles CSI ? 996 n by reading rawOptions.vtExtensions, optionally reading colorSchemeQuery, defaulting null or undefined to true, then testing truthiness. The event fires synchronously before parsing continues. Additional parameters do not change this first-parameter dispatch. Primitive bags must not undergo numeric or string coercion.

Before task 534, replies.rs called TerminalState::mode_option during a mutable TerminalState borrow. A raw bag or nested getter could not inspect, reset, resize or replace terminal state without a Borrowed failure. The shared command now resolves both getter phases outside terminal-state borrows, using the existing option_flag policy. Only then does it enqueue ColorSchemeQuery for the ordinary synchronous dispatcher. Other DSR commands retain the original state-only implementation. The now-unused mode_option forwarding helper is deleted, not suppressed. No unsafe code or dependencies were added.

## Source and native corpus

`scripts/dsr-option-oracle.mjs` executes the unchanged headless public Events/onData test and compares a generated 468-case fixture against the actual supplied source. The two access phases, two buffers, nine values and thirteen actions cover optional/nullish defaults, primitive boxing, truthiness without coercion, reset, buffer activation, resize, cursor/mode changes, disableStdin, replacement of the raw bag, original throws, mutation before throws, and color-query observer reset/throws. Each trace continues with private CPR and printed text to expose early aborts and live continuation. Getter and event observations plus final snapshots compare both buffers, every cell and attribute, cursor, dimensions, margins and relevant modes. Event order and thrown identity are separate checks.

The generated fixture exceeds 350 lines because each line retains one complete source case. Both snapshot implementations are existing shared helpers. The native store and callback closures retain weak state/store references to avoid cycles.

## Verification status

Source generation passed 468 cases and one unchanged headless assertion, with all 776 reference hashes verified before and after. Native compilation, old-production negative control, scoped quality gates and unchanged six-case browser DSR subset are pending. No native Linux/macOS, full original suite, CI, release, performance or overall-equivalence pass is claimed.

## Commands

Run from an isolated worktree with bounded compiler/browser grants, Cargo -j2 and owned temporary paths on O:.

```sh
node scripts/dsr-option-oracle.mjs
cargo test --locked -p xterm-core --test dsr_option -j2
node scripts/build-demo.mjs --debug
node scripts/e2e-upstream.mjs --suite core --grep 'DSR: Device Status Report' --no-build --timeout-ms 180000
```

Source dependencies are required only for the original oracle and browser suites. Set XTERM_SOURCE_ROOT for the source-only oracle if the supplied read-only reference is outside this worktree. Preserve original assertions and reference hashes. Full equivalence remains NOT PROVEN.
