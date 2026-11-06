# Initial strict-check diagnostics

Observed during the first `cargo clippy -p xterm-web --target wasm32-unknown-unknown --lib --test wasm_composition_geometry --all-features -- -D warnings` invocation. This is a transcribed diagnostic excerpt from tool output, not a saved complete raw log. The raw path was overwritten by the later successful strict check, which is retained as clippy-wasm.log.

- `support/input_dom.rs:142`: field `host` is never read.
- `support/input_dom.rs:173`: function `units` is never used.
- `support/input_dom.rs:177`: function `event` is never used.
- `support/input_dom.rs:213`: function `next_turn` is never used.
- Cargo exited 101 because warnings are denied.

The final component regression checks that composed text still arrives after each geometry case, using these shared event/delivery helpers and the host's recorded data. No declaration was hidden by a lint allowance. The complete final clippy-wasm.log records a warning-free pass.
