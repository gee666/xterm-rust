---
name: mouse-coordinate-zero
description: Use when checking mouse coordinate IEEE zero signs against unchanged source functions and Rust native or WASM policies.
---

# Mouse coordinate zero signs

Run `node scripts/mouse-coordinate-zero-oracle.mjs` to replay the 3,249-case source fixture without Cargo, a bundler or a browser. `XT_REFERENCE_ROOT` selects a separate read-only reference. Use `--generate` only after reviewing source changes. The script writes the owned result `results/mouse-coordinate-zero/source.json`.

Run `cargo test -p xterm-web --test mouse_coordinate_zero -j2` and the same command with `--release`. Run `WASM_TEST=mouse_coordinate_zero node scripts/browser-host-check.mjs` for Chromium, Firefox and WebKit when a browser slot is granted. All number comparisons preserve zero signs using bits; NaN payloads are not compared.

See `docs/mouse-coordinate-zero.md` for scope and current execution status. This is not the raw registry305 or full mouse suite. Use the current integration owner's resource grants, owned O: targets/TEMP/TMP and bounded runner cleanup. Preserve failures and source hashes before deleting scratch.
