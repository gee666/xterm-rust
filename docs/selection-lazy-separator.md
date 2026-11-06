# Lazy selection separator reads

Task571 is distinct from accepted task557. The initial resize candidate was already fixed in557, so no resize patch or duplicate tests were applied.

`SelectionHost::options` eagerly converted `wordSeparator` to a UTF-16 string for every selection input. A throwing `Symbol.toPrimitive` or `toString` therefore stopped normal clicks, dragging and mouseup even though the source never used the separator on those paths. It also performed an observable conversion when a double click selected the current link.

The DOM host now exposes `word_separator` separately. Word lookup requests it only after the link shortcut and coordinate check. Word dragging requests it only for a defined endpoint in word mode. Non-word dragging passes an unused empty slice to the existing typed native algorithm. No native word algorithm or source reference was changed. The Rust-only `SelectionDomOptions` loses its eager field; component hosts implement the new fallible method.

This is a bounded correction. The typed word algorithm still converts separators to strings and does not implement source-timed dynamic `indexOf` calls, getter reentry during each neighbor lookup, or arbitrary raw numeric selection coordinates. Other option snapshots also retain separate getter-order gaps. These remain full task321 obligations. Full selection/link equivalence is NOT PROVEN.

## Verification

The source and Rust runners share one public program. The separate source runner preserves the accepted teardown runners unchanged and keeps this fixture's generation and source-map checks independent.

- Twelve source cases passed in each of Chromium, Firefox and WebKit. They cover normal click/drag, mouseup, shift extension and triple click with throwing primitive/string hooks and a counted hook. Each restores a normal string and checks actual word selection. Embedded source-map contents and before/after input hashes match the original TypeScript files.
- The identical Rust test and fixture against old main14139449 failed its whole-trace assertion in Chromium. Normal clicks and triple clicks retained no selection after a poisoned conversion, mouseup lost its event, and the counted hook ran on unused paths. Eight poisoned DOM exceptions were reported as page errors. These errors are retained, not mislabeled as a zero-error control. Firefox and WebKit were not run for old code.
- Fixed `wasm_selection_lazy_separator` passed one test function with all twelve cases in each engine, with zero page errors. `wasm_selection_separator_host` passed two functions in each engine, checking ignored separator failures, link priority and live string word-drag changes through real SelectionDom.
- Retained `wasm_selection`, `wasm_terminal_selection` and `wasm_selection_teardown` passed eight, one and one functions per engine respectively, with zero page errors. The last function expands eighteen cases. Accepted tasks557/564 were not privately copied or rebuilt.
- Native `cargo test --locked -j2 -p xterm-web --all-targets --all-features` passed 434 functions across 191 executable summaries. Native WASM-only targets and examples include zero-test executables. This is not a full workspace or native OS matrix pass.
- Native web all-target/all-feature Clippy and WASM web library plus five relevant browser test targets passed with warnings denied. Web-member formatting passed. Exact `cargo fmt --all -- --check` failed with Windows OS206. The raw failure remains recorded.

Commands, artifact identities, source and Rust traces, gates and input hashes are under `results/selection-lazy-separator`. Old and fixed compiler, served-WASM and binding hashes differ; their shared program, source-generated fixture and public Rust test remain identical. Both compiler artifacts contain the exact program and fixture bytes. The reference manifest's 776 original files and 776 physical-copy files match after execution. Runtime pins become historical after owned cleanup. No compiler/browser rerun is implied by independent inspection of retained receipts.

Full native platforms, unchanged original suites, dynamic raw separator contracts and workspace gates remain separate requirements. Full equivalence is NOT PROVEN.
