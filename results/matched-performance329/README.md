# Actual matched headless checkpoint

Task577 retains a broad measured checkpoint on integrated terminal sources `14139449`, equal to task333's `6cfd03d7` crate freeze. Task329 remains open for final integrated refresh. Full port equivalence is NOT PROVEN.

- `measurements.json` is the synchronous production-core experiment. It calls the actual original internal, deprecated `CoreTerminal.writeSync` with builtin synchronous handlers only. It is not a public API or browser rendering claim.
- `public-callback/measurements.json` is the first actual public-write-callback latency experiment. Its queue scheduling differs from the native synchronous entry point. Do not present those latency ratios as parser speedups.
- Both modes use identical seven input/options/chunk and complete expected-output hashes. Each engine/mode has 14 warmup samples and 70 measured samples. Every sample validates full RICH2 state, ordered events/replies and exact addon outputs.
- `paired.json` and the public phase's separate copy retain actual paired captures completed before any measured repetitions. Each mode's manifest pins its workload and expected output.
- `summary.json` recounts raw samples/statistics and limitations. Rust's synchronous medians were higher in all seven cases. No speedup is claimed, and no run was retried to improve its numbers.
- `javascript-provenance.json` and `rust-provenance.json` identify actual producer inputs, release commands, toolchains, individual source/artifact pins and sixteen Node runtime scripts. The public phase's runtime corresponds to commit `f837222d`; synchronous mode adds only an explicit alternative source entry point, not a different terminal engine.
- `producer-original-*` receipts come from accepted task568's actual unchanged upstream production builds. Four required bundles were copied and independently rehashed, without copying/linking dependencies or borrowing targets.
- Native worker build/test/Clippy logs and nine rejection controls are retained. Native addon tests passed 234 across 58 result summaries. Node protocol/adapter controls passed 65. Six importer controls exercised genuine bundle reuse and failure cleanup. None of these control timings is a performance result.
- `aggregate-fmt.log` preserves required full formatting failure with Windows OS206. `addons-fmt.log` is a member pass, not a replacement for that gate.
- `*.executed.mjs.txt` preserves the actual temporary drivers. Generated JSON/logs intentionally exceed source line limits where needed to retain complete raw measurements, full state, producer metadata, manifests and errors. They are evidence, not hand-written implementation modules.
- `proof-pins.json` hashes the 58 frozen measurement/build/control files. The public phase also has an independent 20-file archive manifest. This README and later review/cleanup receipts are outside that frozen raw set.

The experiment did not isolate a CPU or stop other coordinated machine jobs. Engine order was fixed, JavaScript then Rust. Observed clock granularity was 100 ns for both clocks, not a clock accuracy measurement. Native Windows evidence does not establish Linux/macOS results, browser rendering, all addon functionality or final source identity after later application changes.

Historical absolute runtime paths become unavailable after guarded owned temporary cleanup. Use `docs/matched-headless-performance.md` to rebuild/reimport genuine artifacts and regenerate pins; do not trust vanished paths or relabel a debug/test bundle as production.
