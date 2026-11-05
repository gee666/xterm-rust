# Agamemnon raw input handoff

Task356 accepted5283e75, archived archive/Agamemnon16-raw-input-reviewed, and canonically integrated main e4945e8f24e41ef647dcb39e83964bfa6d35ed35. Source commit rebased to4aaa9e9. The batch includes only reviewed356 plus owner-confirmed docs-only357 daed4d1 and Galen d2e5bdc receipts.

Read docs/raw-input-data.md and results/raw-input356. Non-string admitted second InputEvent.data reads now retain their actual JS values through browser onData. CoreService shares disableStdin/buffer/scroll/user-input effects with typed input; raw logging preserves eager String-hint conversion, lazy trace split/map/charCodeAt receivers, custom map results, throw identity and nested input. Native UTF16 APIs remain typed; browser non-string values do not enter native on_data. Source engine delegation, new dependencies and unsafe code were not added.

Fresh scoped proof: source-integrity7, native core14/web9; all3 engines pass raw2 functions expanding90 cases, InputEvent4 with288 guard combinations, input callbacks3, input7, keyboard15, terminal wheel5 and writer5. A string-only negative-control producer fails both new raw functions in Chromium. After rebase containing354, full WASM library26 and raw2 pass per engine. Reviewer independently reran source7/native23/fmt/native+WASM Clippy; reviewed browser receipts without a new matrix.

Canonical default-feature gate passed1854 native core/web tests across346 summaries, zero failed/ignored, plus changed/affected/default workspace and applicable WASM checks. Fullfmt and source integrity also passed. Exact scope is in canonical-merge.json/log. This does NOT establish full input/accessibility319 or overall equivalence.

Remaining work belongs to319 and complete-suite327/final-audit328. Raw event fields outside declared primitive DOM types and complete keyboard/IME/focus/clipboard/accessibility source suites remain unproven. The old full-library viewport failure is repaired by accepted354 and passed in this refreshed library run; do not replay the old fixture workaround.

Owned temporary tree is only O:/xterm-rust-swarm-16/tmp/A, created as Agamemnon16 and safely moved for Windows rustfmt path limits. It has a physical copied reference, not a junction. Canonical helper removed the private target. Final cleanup must confirm zero links/redirects, no owned compiler/browser/server processes, clean tracked status and only whitelisted generated browser receipts before removal. Source/evidence branches survive removal. Never clean historical trees, original dependencies, credentials or private swarm files.
