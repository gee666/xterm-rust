---
name: keyboard-dispatch-check
description: Use when verifying browser keydown, keyup, keypress and input state/effect ordering against the original CoreBrowserTerminal method bodies.
---

# Browser keyboard dispatch verification

Run `node scripts/keyboard-dispatch-oracle.mjs` from the repository root with Node supporting stripTypeScriptTypes and Rust tools on PATH. CARGO and RUSTC select explicit tools. Reuses the keyboard oracle's TypeScript erasure without new dependencies.

The script extracts unchanged CoreBrowserTerminal input method bodies and the modifier helper, executes them with ordered host-effect capture and real original protocol encoders, builds the Cargo keyboard_dispatch_oracle example, and compares full return/state/effect traces against that binary. Matrices cover named keys, flags, modifiers, modes, composition veto, custom veto, third-level boundaries and 50,000 seeded mixed event operations. The browser Mac override and encoding platform are independent. Exact counts, seed and hashes appear under results/keyboard-dispatch; failures retain counterexamples with prior operations. Build failures remain separately recorded.

Run `cargo test -p xterm-web --test keyboard_dispatch --test keyboard_dispatch_callbacks --test keyboard_dispatch_getters` and focused warning-denied Clippy. Tests cover the source's Mac/Windows printable keypress cases, UTF-16 modulo, duplicate suppression, callback failures and synchronous callback reentry. Detailed branch contracts are docs/keyboard-dispatch.md.

Do not label this source-method trace runner as the original complete Terminal browser suite. Actual DOM events and complete terminal/CoreService effects require task33's Rust-backed browser tests and task94's integration review. When unrelated shared modules prevent Cargo integration, explicitly use `node scripts/keyboard-dispatch-oracle.mjs --focused`. It compiles actual policy sources and tests with rustc and Clippy-driver, creates small wrappers under root tmp, adjusts only two temporary Kitty submodule paths, and removes all its build products. It reuses the shared ryu-js dependency output, not a duplicate Cargo target. Separate focused reports cannot satisfy workspace or actual browser gates. CLIPPY_DRIVER and RYU_JS_RLIB select explicit tools/artifacts when needed. No sleeps or silent fallback. Honor shared low-artifact Cargo configuration.
