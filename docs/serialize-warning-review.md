# Serialization warning review

Task359 was accepted by an isolated reviewer against commit `c479f37` on 2026-11-05. The reviewer independently ran the two focused native test binaries, 7 tests passed, and exact `cargo fmt --all -- --check`, which passed.

The reviewer inspected the changed core and browser code, checked the retained source/Rust warning probes in all three browsers, and verified both baseline missing-reset failures. The original serializer and test-suite hashes matched the evidence. The retained unchanged suite reports show120/120 source and120/120 Rust passes across the three engines. The reviewer did not launch browsers; its browser conclusions use retained traces.

No concrete problems were found. The accepted contribution is late ANSI rows, length, cursor and attribute reads after missing-cell warnings, including propagated callback failures. Task346 still owns raw numeric/coercion, replaced host methods, palette mutation, pinned-line identity and actual GC behavior. Task322 still owns full search/serialization closure. Overall equivalence is NOT PROVEN.

The producer's complete browser run, native tests and lint commands are described in `docs/browser-serialize.md`; raw evidence is in `results/serialize-browser/` and `results/serialize-reentry/`. These browser results are from the focused branch, not a claim of a fresh post-integration browser pass.
