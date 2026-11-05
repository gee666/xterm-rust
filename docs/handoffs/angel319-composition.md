# Angel input continuation

Task395 is independently accepted and integrated as a4e43ca600cfd00e9a60fc8bbe87986db95785c7 on actual main. It fixes compositionend ordering: classList.remove must precede Rust flag/range capture. A throwing removal must leave composition active, and a nested compositionstart must supply the outer end's range. Old code demonstrably sent `a` where the unchanged source sent `B`.

Read docs/composition-end-order.md and results/composition395 before repeating work. New four original-body scenarios pass in all three engines, within the existing callbacks target5 functions per engine. Existing input7, liveKeyboard15, a11y25 and Terminal a11y7 also passed per engine. Native27, unchanged clipboard/IME11 cases16assertions plus252412 comparisons, accessibility32871 snapshots/65536 BMP classifications passed. Reviewer independently reran source3/native27/callbacks5 per engine/scoped formatting and Clippy.

Canonical integration rebased onto accepted Base64 fd16b12/4f7c595; xterm-web343 native tests across129 summaries passed, then affected native/default workspace/applicable WASM checks. Eight owned file hashes stayed exact. Full-workspace formatting still hits Windows OS206. This is scoped Windows evidence, not native Linux/macOS or complete parity.

Task319 was claimed for the family audit and will be released unfinished after this scoped contribution. It still owns complete unchanged input/accessibility suites, every decorated event and DOM getter/value, actual full-Terminal clipboard/IME/focus delivery, error/reentry/timer lifecycle and mapping. Prior343/356/378 already recovered raw keyboard tables, raw InputEvent payloads and keypress numeric fields. Do not recreate them.

Owned temporary tree is O:/xterm-rust-swarm-16/tmp/worktrees/Angel319, branch swarm/Angel319. Its source_repo is a physical copy, not a junction. No browser/server/compiler remains owned. The canonical helper already removed target. Preserve this receipt commit and raw integration log before removing the tree. Whole-tree cleanup requires clean tracked status, an explicit untracked whitelist, no matching process command lines and zero reparse points. Do not touch other O: paths, historical source/dependency links, user files or credentials.

Overall equivalence NOT PROVEN.
