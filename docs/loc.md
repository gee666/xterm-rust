# Reproducible source line counts

Run with Node 24 or later, without installing packages:

```text
node --test scripts/count-loc.test.mjs
node scripts/count-loc.mjs --check-baseline
```

The generated [comparison table](../results/loc/report.md) reports original-versus-current-port files, physical lines, nonblank lines and raw source bytes for every category. [summary.json](../results/loc/summary.json) contains the same totals, tool/rule versions and source inventory hashes. Complete per-file inventories are [source.json](../results/loc/source.json) and [port.json](../results/loc/port.json).

This is a partial-port working-tree snapshot, not evidence that Rust implements the same functionality with fewer lines. Zero addon or browser-binding totals mean those directories were absent or not yet included at scan time. They do not mean those features cost zero lines. Refresh after integration stabilizes before making a final comparison. This snapshot does not measure speed, native executable size, JS download size, compressed WASM or functional equivalence.

## Counting method

`count-loc.mjs v1` applies exactly the same UTF-8 algorithm to both trees. Physical lines use LF or CRLF delimiters; a trailing delimiter does not create a phantom line, and final unterminated content counts once. A standalone CR is content, not a delimiter. A line is nonblank when JavaScript's Unicode-aware `trim()` leaves content. A UTF-8 BOM is retained as content for physical counting, but is whitespace for nonblank counting. Comments are included. These are not comment-stripped SLOC counts. Raw bytes and SHA-256 hashes cover the original bytes, without newline normalization.

The checked-in rules include Rust, JS/TS and declarations, CSS, HTML, shader languages and additional native source extensions. Configuration is counted separately, as are textual test fixtures. Structured JSON/YAML/TOML files inside explicit test fixture directories are test-data, including dependency-lockfile-shaped fixture names. Ordinary test tsconfig.json and real dependency lockfiles keep their configuration/exclusion rules. Binary fixtures, images, fonts and documentation retain byte/hash inventory entries, but are not assigned fabricated source line counts. Invalid UTF-8 source or configuration fails explicitly. Unknown production extensions fail for classification review.

Categories are exclusive. Rust files containing line-start test or test-conditional attributes are separated as whole-file mixed-code-tests when located outside test-only paths. This includes inline tests in native demo/checkpoint helpers. The annotation scan is not a Rust AST parser and does not produce an exact test-only line subtotal inside those files. Mixed file totals must not be called pure production or demonstration logic. Purpose totals separately retain the original location role, including mixed files, so a generated test belongs to category generated-code-data and purpose tests. Declarations, dedicated WASM binding crates and explicitly listed binding files are bindings-glue. Mandatory generated browser/pkg JavaScript and declarations are included as generated-code-data with purpose bindings-glue, not silently excluded as build output. Their WASM binary is inventoried by bytes without fabricated line counts. browser/xterm.mjs is handwritten binding glue; browser/xterm.css is production library CSS; browser/demo remains demonstration code. The xterm-web BrowserHost export and search-addon options ABI files are listed; other browser implementation files remain production-library. Inline JavaScript in a Rust file is already included in that file's physical lines. Mandatory generated wasm-bindgen snippet JavaScript is also counted separately as generated binding glue, so this intentional source/runtime-glue duplication stays visible instead of being silently removed. Addon implementation is addons. Actual user examples are demos; differential oracle/trace examples are tooling. The current rules recognize the shared `crates/xterm-addons` layout as addons as well as individual addon crates.

Generated-code-data includes standalone generated source/tables and production textual data. A generation comment near the start or an explicit override identifies generated files. Strings inside generators do not make the generator generated. Upstream UnicodeV6, UnicodeV11 and UnicodeProperties combine logic and data in single files. They are explicitly listed as mixed-code-data, with whole-file counts, rather than pretending the table lines can be isolated by a physical-line tool.

## Boundaries and provenance

`scripts/loc-rules.json` records the exact extensions, data overrides, mixed files and excluded directory names. Version control, dependencies, temporary/private runtime data, build outputs and report outputs are excluded. This includes nested addon `out-esbuild-test` directories created by original browser builds. The port scan excludes the original tree and pre-existing user swarm launchers. Custom output directories and a custom nested source tree are excluded dynamically. Selected source roots must be real directories, not symlinks or Windows junctions. Symlink/junction entries inside the tree also fail for review rather than following a path outside the tree. This is a stable-input scan, not protection against a hostile filesystem swapping paths during traversal.

Every admitted file has a normalized relative path, raw byte count and SHA-256. Reports also record excluded paths, rule hash, deterministic inventory hashes and the tested Node version. Inventory hashes cover classification metadata as well as file records, so changing category rules can change that hash without changing any source bytes; each file's raw SHA-256 remains separately recorded. Paths are sorted without locale-dependent collation. An actively edited working tree is not an atomic Git snapshot; each report accurately counts the bytes it read, but later changes require a fresh run. This records a supplied local reference, not an invented public revision.

The baseline subtotal uses only the original ten launch extensions and legacy grouping. `--check-baseline` reproduced all four baseline values: 396 files, 96,130 physical lines, 87,026 nonblank lines and 3,962,777 bytes. The broader report also includes configuration and textual test data, giving the original 635 counted files, 102,971 physical lines and 92,731 nonblank lines. This difference is a documented classification expansion, not a changed source implementation. The detailed categories reassign declarations, addons and data while preserving a separate legacy subtotal.

Custom roots and output are available:

```text
node scripts/count-loc.mjs --source path/to/reference --port path/to/port --output tmp/loc-other
```

The baseline check is optional for custom references, because the checked-in baseline describes this supplied xterm.js tree. Invalid flags, missing values and an output directory equal to a source root fail explicitly.

## Verification and limits

Fifteen Node tests cover selected-root junction/file refusal, inline-test mixed-file classification, mandatory generated browser glue, structured-fixture/configuration precedence, line boundaries, BOM-only and BOM-plus-newline source, exact byte arithmetic, category/purpose/extension reconciliation, generated markers and negative controls, mixed data, binary fixtures, unknown production extensions, nested exclusions, custom output stability, deterministic hashes, invalid flags, baseline mismatch failure and symlink/junction refusal. Command output is retained in `results/loc/tests.txt`.

Acceptance details are in [acceptance/loc.md](acceptance/loc.md), criteria 166 through 171. Complete generated JSON manifests may exceed 350 lines because they enumerate every file. The handwritten scripts remain below that limit. Native Windows Node execution is observed; no Linux/macOS run is claimed here.
