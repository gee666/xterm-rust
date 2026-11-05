# Serializer collection review

An isolated reviewer accepted task382 at `dc9034c` and independently reran `node scripts/serialize-browser-check.mjs --gc` in Chromium148.

All four phases passed against both source and Rust. Counts were24 discovered,8 executed,8 passed,0 failed,0 skipped and16 unrun for Firefox/WebKit. The reviewer confirmed exact two-collection phases, separate page evaluations, a valid retained/released reachability control and no sleep or retry-until-pass loop.

The reviewer checked the actual Rust fixture loaded verified WASM without a source-engine fallback. Build-input identity,776 unchanged reference files, server cleanup and removal of tmp/serialize-gc were independently verified. No concrete problems were found. The generated GC report now includes this independent rerun.

Normal serialization matrices and original suites remain separate. The accepted scope is Chromium wrapper retention and collection, not native heap reclamation, all open-DOM lifecycles, Firefox/WebKit collection or full346/322 closure. Overall equivalence is NOT PROVEN.
