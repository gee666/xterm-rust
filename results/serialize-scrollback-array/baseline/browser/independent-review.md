# Independent review receipts

Tasks349 and310 were independently accepted at source tip395823e in O:/xterm-rust-swarm-16/tmp/C. No tracked changes were made by either reviewer.

349 reviewer inspected actual packed row/cell enumeration, raw `_ext` versus computed `ext`, JS field shaping and borrow boundaries. Fresh checks passed4 Node functions,28 focused native functions, full cargo fmt, scoped native/WASM Clippy with warnings denied and whitespace checks. Current build-input/artifact/runner/declaration/reference hashes matched. The776-file reference manifest had0 mismatches. No fresh browser or full workspace job ran.

310 reviewer inspected serialize_addon/{mod,html,options,view}, shared Rust serializer changes, compatible JS facade and host/lifecycle integration. Fresh checks passed48 serialize functions plus2 diagnostics,4 Node functions, full cargo fmt and scoped native/WASM Clippy with warnings denied. Source/artifact/runner/declaration/reference hashes matched. No fresh browser matrix ran.

Both checked retained unchanged source120/120 and Rust120/120 receipts,40 per engine with0 failures/skips/unrun. Both retained47/48 shared comparison matches per engine and explicitly left the raw numeric failure as a failed/nonzero command outcome under346. Neither acceptance closes346/322, public raw object behavior315, actual GC, full-state/event/demo parity or global equivalence.

The fresh test repeats overlap earlier native/Node executions and must not be added as unique-case counts. Integration onto actual main and the canonical merge gate are separate work. Source ownership is preserved on swarm/Chase17; this receipt does not claim integration already happened.
