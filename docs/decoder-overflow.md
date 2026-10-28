# Source decoder target-overflow counterexample

This is a retained source defect, not passing Rust equivalence evidence.

`node scripts/decoder-overflow-source.mjs` uses an existing cached, unchanged headless reference bundle. It checks every recorded source hash, records the bundle hash and runs each case in an owned child with a two-second hard deadline. It does not invoke a compiler, bundle code, modify source_repo, replace decoder/parser behavior or weaken assertions. Observer wrappers forward original arguments and results and emit at most two repeated empty-print witnesses.

Observed native Windows/Node24.15 source result in `results/decoder-overflow/2026-10-28T06-50-54-833Z-11424/`:

| Writes after a pending high surrogate | Physical target | Logical count | Source outcome |
| --- | --- | --- | --- |
| 4095 ASCII units | 4096 | 4096 | Callback completes, cursorX16 |
| 4096 ASCII units | 4096 | 4097 | Repeats PRINT[4096,4096], child times out |
| 131072 ASCII units | 131072 | 131073 | Repeats PRINT[131072,131072], child times out |

The original decoder silently discards the final out-of-bounds target write but returns its logical count. The parser classifies the missing typed-array element using the non-ASCII PRINT table slot. Its printable-span scan makes no progress, invokes the original print handler with start=end, then sets the index back so the same iteration repeats. Source cursor state after the physical prefix remains unchanged across both captured witnesses.

The cached artifact was not freshly rebuilt during this probe. Source hashes match the saved provenance, and the current read-only parser body contains the same no-progress path. A later fresh-build reproduction remains separate work.

Rust fixed-target decoders must preserve dropped writes and logical counts. That primitive agreement does not make a finite Rust DecodeBufferOverflow error equivalent to source nontermination. Task209 must retain this counterexample, describe any chosen safety deviation, and avoid claiming full equivalence for this boundary while it differs. No native malformed-input check may silently turn this source counterexample into a successful no-op.
