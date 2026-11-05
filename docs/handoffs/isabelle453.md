# Isabelle453 handoff

Task453 is accepted. Full342 and overall port equivalence remain NOT PROVEN. This continues the same swarm2.

The private branch is swarm/Isabelle453. It is stacked on accepted442 tip5f01a73e. Only apply the NEW453 chain6762ac34,19b42e65,2349c98c,6379ba2a and the following review/cleanup receipt. Do not replay standalone pre-stack aa444356 or442 ancestors. Main was not changed by Isabelle. Integration is delegated to309, after the frozen448 batch.

The source validates final glyf/loca descriptors and captured head format before hmtx, using the current refreshed payload reader. Required loca entries are numGlyphs plus one, but absent maxp causes no entry reads. Glyph body/offset values remain lazy. Compression and zero-read boundaries are shared with hmtx in font_outline.rs. No new dependency, unsafe code or JS parser delegation.

Preserve442 owned cmap and refresh state, fixed descriptors and absence of unused DeferredTable.reader/load. Preserve420 hint currentpayload/originaldescriptor hook,438 GDEF/avar and447 deferred GPOS afterGDEF/preGSUB.457 owns CFF/CFF2 branch separately; TrueType pair precedence stays first. font_bytes is342 lines on the tested442+453 combination. Use minimal coherent unions and retest them.

Evidence is in docs/font-outline-validation.md and results/font-outline. The612-case source/native/WASM corpus has43 accepts/569 rejects, including128 seeded cases0x10CA453 and early CPAL callback mutation controls. Four actual-addon scenarios cover rejection/cache/recovery and lazy body/offset acceptance. Both WASM functions passed Chromium148.0.7778.96, Firefox150.0.2 and WebKit26.4 with zero failures/ignored/filtered. All178 native addon tests across36 targets and strict native/focused-WASM Clippy passed. Bypassing the outline hook failed383 cases; restored production passed. Fullfmt failedOS206; five complete member checks passed.

The isolated reviewer inspected the source delta, reran612 native/source cases and four source-addon scenarios, and verified13source pins/native executable and20upstream source hashes. Its first request was only browser execution. After the actual matrix, resumed review accepted and independently verified all four committed compiler/generatedWASM/bindings/CSS hashes, source/test/Cargo.lock pins, and reran native612. No compiler/browser jobs were launched by reviewers.

Temporary ownership is ONLY O:/xterm-rust-swarm-16/tmp/worktrees/Isabelle453. No junction was intentionally created. Browser servers/browsers closed through runner finally. No owned heavy jobs or main-ref operations remain after409/2996 RELEASE. Native test binaries and raw browser-host output are temporary. All required source, regression inputs, concise logs and four actual browser artifacts are committed before cleanup. Original P: source/dependencies are never cleanup targets. Final preflight/removal and original776-file rehash receipts follow.
