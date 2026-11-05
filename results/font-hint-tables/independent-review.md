# Independent task420 review

Task420 was accepted at `67cd67e6` against unchanged producer `26277170`. The isolated reviewer reported no concrete problems in the final state.

The review retained and required fixes for three actual source mismatches: missing stored-block payload, partial inverse-header NaN arithmetic, and exact8-bit buffered transitions into stored blocks. All are now source-generated native/browser regressions. A fourth preflight found no new source issues and withheld acceptance until the final792-case matrix was supplied.

Final independent checks reran the read-only Node oracle and current native binary, verified all14 producer hashes and compiled artifact hashes/sizes, confirmed the fixture is embedded in the compiled artifact, checked source/Rust three-engine receipts and both776-file source manifests. No compiler or browser matrix was independently rerun. Recorded actual matrices passed2Rustfunctions per engine, replaying792constructors and792addon records, with377accepted/415rejected source outcomes. The separate nontermination guard is not counted as a finite equivalence case.

Acceptance covers only eager cvt/fpgm/prep construction. Broader342, current-main integration, global quality, nativeOS and full terminal equivalence remain open. Earlier workspace2078 receipts cover older decoder revisions; final scoped174native and strictClippy receipts cover producer26277170. Aggregate fmtOS206 is retained, not converted to a pass.
