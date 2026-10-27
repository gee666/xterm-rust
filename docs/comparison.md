# Original and Rust comparisons

## Lines of code

See [the counting method and commands](loc.md) and the generated [original-versus-current-port source table](../results/loc/report.md). Raw classified inventories, hashes and baseline checks are under `results/loc/`.

The current report is a partial implementation snapshot. It is not a matched-functionality code-reduction claim. Rerun after all implementations and integration changes stabilize.

## Speed and distribution size

No speed or artifact-size measurements are reported by the LoC tool. Those measurements have a separate owner and acceptance gate, task #48, and must satisfy PORT_REQUIREMENTS.md using validated matched workloads, release builds, warmup, at least ten repetitions, median/p95 and explicit native/JS/WASM distribution components. Missing measurements are not zero and no speedup is claimed.
