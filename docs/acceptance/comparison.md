# Speed and size acceptance

Task 48 implements these checks. They refine criterion 8, not replace LoC criteria 166-171. These are required measurements and failure conditions, not results. Missing engine/addon/browser implementation is unfinished work, not a benchmark value or external blocker.

## C01. Identical work and result validation

- Check in one workload manifest used by JS and Rust. Each case records exact input bytes or byte hash, dimensions, scrollback, Unicode provider/version/settings, operation list/count and expected final-state hash.
- Cases include printable ASCII, supplementary/combining/grapheme text, mixed ANSI controls/attributes, scrollback overflow, alternate buffer, grow/shrink resize and wide-cell reflow. Include matched search, serialization and applicable image/rendering/addon operations rather than silently reporting core-only results as full-featured.
- Define whether writes are bytes or UTF-16 strings and identical chunk boundaries. The Rust and JS paths must perform the same conversion and synchronization. Await asynchronous completion before stopping the clock.
- Validate complete relevant outputs before timing: cells/codepoints/widths/attributes, cursor/buffer/modes/scrollback and replies for terminal workloads; result spans/decorations for search; exact serialized bytes/HTML for serialization; pixels/frame completion for rendering. A wrong-output workload fails and produces no speedup ratio.
- Input count, byte count and operation count must be independently recomputed from the manifest. Test zero/empty workload, partial/truncated input, a deliberately mutated expected output, differing terminal dimensions, wrong Unicode version and unawaited async callback rejection. Reject incomparable inputs instead of adjusting only one side.
- Browser rendering comparisons execute equivalent Rust-backed and original browser pages, viewport/DPR/font/palette/options and visual workload. Native parser timing versus browser rendering must never be called a speedup.

## C02. Timing and aggregation

- Use optimized release Rust and original production JS. Record exact build commands, rustc/cargo/Node/browser versions, lockfiles, CPU/OS/architecture, target, features and flags. Keep the shared dev/test disk policy; do not change release optimization to manipulate comparisons.
- Record warmup separately. Run at least ten measured repetitions per case after warmup for both engines, and explicitly count browser/JS warmup. Setup/compilation/font/fixture loading is outside steady-state timing unless the case is explicitly labeled startup-inclusive.
- Record raw durations with a monotonic clock, timing units and work counters. Define the exact median and p95 algorithm, including even sample counts and percentile interpolation/indexing. Test known small arrays, repeated values, unsorted input, nonfinite/negative durations and insufficient repetitions.
- Report bytes/second or operations/second consistently. Avoid divide-by-zero ratios and distinguish missing samples from genuine measured zero duration. Retain outliers and failed iterations with reason; do not silently select the fastest runs.
- Repetition order and environment must avoid systematically warming only one engine. Document clock granularity, process startup inclusion, thread/process count and whether browser measurements include animation-frame/GPU completion.
- A machine-readable runner receipt must expose timeout, nonzero exit, signal, malformed JSON, missing case, mismatched hash or repetition count. Negative runner tests must fail closed. Never replace failure with zero throughput.

## C03. Production artifact inventory

- Build original production browser and headless bundles, Rust release native library/example artifacts, and Rust browser/WASM distributions. Record exact paths and raw bytes, not tool estimates or unstripped debug files.
- Web distribution inventory lists WASM, generated binding JS, bootstrap, CSS, required fonts/data/workers/shaders and mandatory assets separately and as a total. Include all languages and dependencies required at runtime.
- Publish core-only and matched-feature distributions. Enumerate optional addons and dependencies included/excluded on both sides. A Rust native executable is not directly comparable to JavaScript download bytes; keep native and web categories separate.
- Record stripping, LTO, codegen units, panic strategy, optimization level and release flags. A missing output or required supporting asset fails the distribution case instead of being counted as zero bytes.
- Deduplicate only actual shared distribution files and document the rule. Test repeated references, empty files, nonregular files/symlinks, missing entries, traversal/absolute paths and a mandatory asset not referenced by the manifest.

## C04. Compression and reproducibility

- Apply identical gzip and Brotli versions/settings to corresponding web distributions. Record compression level/mode/window and byte totals for individual files and the summed mandatory distribution. Do not compare concatenated archive bytes with a sum without labeling the difference.
- Rebuilding and recounting the same content must retain hashes and counts. Sort manifests deterministically and exclude generated reports, caches, dependencies/build trees not actually shipped and private runtime files.
- Test compressed round trips against exact original bytes, including binary WASM, CRLF text, Unicode binding code and an empty asset. Compression failure must propagate with a nonzero status, not silently omit an asset.
- Keep raw machine-readable manifests and measurements under tracked results, with exact reproduction commands in docs/comparison.md. Separate current partial-port measurements from eventual matched full-port results. Retain prior failing runs and no speedup claims while output equivalence or scope validation fails.
