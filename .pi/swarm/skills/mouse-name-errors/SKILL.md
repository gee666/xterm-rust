---
name: mouse-name-errors
description: Use when checking lossless primitive-string mouse mode errors through the actual Rust browser service.
---

# Mouse mode name errors

Coordinate the current bounded compiler/browser lane before running commands.
Use an isolated O: worktree, Cargo -j2, and TEMP/TMP under its tmp/ directory.

Run `MOUSE_CASE_FAMILY=name-errors node scripts/mouse-mode-source.mjs --generate`
to capture five shared cases from unchanged source Terminal in Chromium, Firefox
and WebKit. Set XT_REFERENCE_ROOT to the read-only supplied reference with its
existing dependencies. Review new captures, then rerun without --generate to
check drift. Do not alter source assertions.

Run `WASM_TEST=wasm_terminal_mouse_names node scripts/browser-host-check.mjs`
with WASM_BINDGEN=0.2.100 and PLAYWRIGHT_MODULE pointing to the existing tools.
This executes five Rust wasm-bindgen-test functions in each engine. It is not
full mouse registry or full Terminal equivalence.

For a direct ABI control, supply an existing generated Rust test package with
`MOUSE_NAME_CONTROL_DIR`, set `MOUSE_NAME_EXPECT=old` or `fixed`, and run
`node scripts/mouse-name-control.mjs`. Old must mismatch all five unchanged
shared cases; fixed must match all five. BROWSERS defaults to chromium. The
receipt separately records zero Rust test functions and exact surrogate error
code units. Never relabel these direct calls as Rust test execution.

Retain source/compiler/bindings hashes before cleanup. If swapping owned source
for the old control, use writeFile, verify Cargo fresh=false, restore exact
initial bytes in finally, then require final compiler identity and rerun real
Rust tests. A stale copyFile mtime is not valid negative or restoration proof.
