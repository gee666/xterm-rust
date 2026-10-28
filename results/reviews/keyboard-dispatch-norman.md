# Independent keyboard dispatch review

Read the original CoreBrowserTerminal key methods and current Rust dispatcher/read-host contracts. Inspected callback ordering, Cell state, tri-state returns, keypress charCode/which branches, Meta modifier-only events, third-level short-circuiting and getter failure tests. Static event encoding and host-effect comparisons are scoped separately from mutable DOM properties.

Executed on Windows:

- `node scripts/keyboard-dispatch-oracle.mjs` passed 165328 ordered source-method comparisons, seed 1145656144, normal Cargo build rather than focused harness.
- `cargo test -p xterm-web --test keyboard_dispatch --test keyboard_dispatch_callbacks --test keyboard_dispatch_getters` passed 14 tests, none ignored.
- `cargo fmt --all -- --check` failed on active shared-tree formatting, including my in-progress DOM adapter changes. No workspace quality pass is claimed. Those files are being formatted before their own review.

The source-method runner executes actual upstream method bodies with structural collaborators, not the complete original browser integration suite. Review accepts task94's pure dispatcher scope only. Task151 tracks the discovered real DOM adapter gap where pre-snapshotting key fields can run getters before custom veto or ignore callback mutations. Actual browser/terminal and full equivalence remain unproven until those separate gates pass.
