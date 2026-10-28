# Serialize work in progress

Owner Vikramaditya. This is a handoff, not completion evidence.

## Earlier components

- Task 37 ANSI handler, styles, modes/margins and native buffer reader accepted after independent source comparison, commits `c7f8ab7` and `a4c5a50`.
- Task 103 HTML handler accepted; Joyce's task 134 adds raw UTF-16 options.
- Task 87 oracle accepted. Source baseline 44 tests/60 assertions; current expanded corpus 3350 outputs. Read current `results/serialize/oracle.json` and receipt status, not historical counts.

## Native task 88 reopened and corrected

Commits `8c93b02` and `137d573` contain the live wrapper, restore traces, header/default-concatenation and weak-ownership work. Thirteen native tests passed, but later review found two further source counterexamples. Task 88 was reopened instead of accepting those earlier greens as complete proof.

1. **No proposed-API guard.** Actual browser/public/Terminal.ts:97 and headless/public/Terminal.ts:87 buffer getters are ungated. The invented native guard and its self-authored expected-error assertion were wrong. They are removed. Original upstream assertions were not changed.
2. **Reset aliases the public view.** Namespace activation reinitializes the same captured BufferApiView after reset. A header getter calling reset serializes replacement blank cells, not text in an old raw BufferRef.

`node scripts/serialize-cached-probes.mjs` confirmed both behaviors from the source-hash-verified original bundle without a compiler/browser. Full fresh `serialize-live-oracle.mjs` now covers both public terminal classes with default false flags and the reset trace.

Latest implementation changes:

- `serialize/addon.rs` lazily shares the actual public namespace cache, holds weak state/cache links and reads the captured view after header callbacks. No proposed guard remains.
- `serialize/view.rs` implements BufferAccess over that public view. No state/resource borrow crosses host header getters or coercions.
- `serialize/mod.rs` transports typed BufferError causes through an Rc-backed error variant. Protocol errors keep value equality; transported failures keep identity.
- `serialize_live.rs` expects success with false flags; `serialize_live_reentrancy.rs` expects later false-flag calls to work; `serialize_live_reset.rs` asserts lazy activation and reset retargeting.
- Antigone added `Terminal::buffer_cache_for_addon()` and Rc<OnceCell<...>> cache ownership. Confirm this dependency is committed before committing consumers.

These corrections now passed controlled checks: fresh source oracle, 14 live native tests, warning-free six-target Clippy, scoped rustfmt, and 3350 exact outputs plus 14 HTML tests. Current source hashes and raw receipts are in `results/serialize/reset-current-evidence.json`. Historical failed/rejected attempts stay retained. Independent review of reopened 88 and a coherent commit are next.

## Commands and resource policy

Builds/bundlers/browsers use `scripts/build-slot.mjs` per `docs/build-slots.md`, existing profiles and shared target. Tiny cached-source fixture checks need no compiler. No local release/full matrix without separate approval.

Scoped native target list:

`cargo test -p xterm-core --test serialize_live --test serialize_live_callbacks --test serialize_live_reentrancy --test serialize_header_order --test serialize_ownership --test serialize_live_reset`

Run the same targets through Clippy with `-- -D warnings`. Lower regression command is `cargo test -p xterm-core --test serialize_differential --test serialize_html --test serialize_html_edges`. Source traces come from `node scripts/serialize-live-oracle.mjs` through a slot. The latest successful receipt directories are listed in `reset-current-evidence.json`.

## Browser task 131

Claimed by Vikramaditya and waiting for accepted 88. There is **no JS/WASM SerializeAddon class export yet**. Sophie needs it for the browser demo. `docs/serialize-browser-acceptance.md` SB01-SB08 and `docs/serialize-api-map.md` give exact scope/gaps.

Xavier reserved `crates/xterm-web/src/addons/serialize.rs`. Browser Terminal.shared is crate-visible, with native, selection and browser services. `shared_native()` and `selection_service()` are available. Request its root export from Xavier when real implementation exists. All terminal logic remains Rust.

Preserve JS getter/coercion phase order and number/marker behavior. usize/isize options are not a JS conversion layer. Public view aliases and missing-cell console warnings need actual browser paths. JS host retention/GC is separate from the explicit native Rust Drop/TerminalReleased boundary. Post 119 records that distinction; do not claim generic GC parity.

Overall full xterm.js equivalence remains NOT PROVEN. Missing implementation is work, not an external blocker.
