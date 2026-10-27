---
name: selection-model-check
description: Use when verifying selection coordinates, endpoint wrapping or scrollback trims against the read-only xterm.js SelectionModel.
---

# Selection model check

Run from the repository root:

```text
cargo test -p xterm-web --test selection_model
node scripts/selection-model-oracle.mjs
cargo clippy -p xterm-web --test selection_model --example selection_oracle -- -D warnings
```

Node 24.15 or later executes the original class and its 18 assertion bodies with a coordinate-only buffer fixture. The script compares 38,130 cases against the Rust executable, including 20,000 seeded cases using seed 0x051ec710. It uses the shared Cargo target directory and a three-minute subprocess timeout. Do not override the repository development/test profiles or create another build directory.

Read docs/selection-model.md for numeric boundaries, fixture limits and the eight-field result format. Results and retained counterexamples are under results/selection-model/. A model pass does not establish SelectionService or browser integration. Never edit expected values to hide a mismatch.
