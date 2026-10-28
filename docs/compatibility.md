# Compatibility inventory

Overall xterm.js equivalence is **NOT PROVEN**. The compatibility index records implemented fragments and missing or unmapped contracts. Its own tests do not prove terminal behavior. In particular, a Rust helper does not establish a public Terminal, browser/WASM binding, renderer or addon implementation.

## Reproduce the index

Requires Node.js 22 or later and npm. TypeScript 5.9.3 is a development-only parser dependency with an exact lockfile. It is not part of the Rust runtime or distributed terminal.

```sh
npm ci --prefix tools/compatibility --ignore-scripts --no-audit --no-fund
node scripts/compatibility-inventory.mjs
node scripts/compatibility-inventory.mjs --check
node --test scripts/compatibility-inventory.test.mjs
```

Commands work from the repository root. The script resolves its inputs relative to its own path, so an absolute script invocation also works from another directory. Generation rewrites only `results/compatibility/*.json`. `--check` writes no artifact content and fails on a changed source, mapping, fixture, evidence reference or mapped evidence byte. Concurrent subsystem edits can correctly invalidate the snapshot; regenerate only after reviewing those changes. It does not edit the reference or execute Rust/JavaScript terminal tests.

The negative tests use a private `tmp/compatibility-validation/` snapshot and remove it at successful completion. They exercise stale artifacts, duplicate mappings, nonexistent modules/evidence and paths escaping the allowed output location. Alternative `--out=tmp/name`, `--mappings=tmp/name.json` and `--baseline=tmp/latest.json` options are for these tests, not alternate production inventories. Baseline negative tests alter only private pointer/summary copies.

Detailed acceptance and failure cases are in [acceptance/compatibility.md](acceptance/compatibility.md). Criteria 50 through 53 cover reference completeness, public contracts, honest suite accounting and integrity respectively. They do not satisfy the focused implementation criteria.

## What is enumerated

The supplied reference reports version 6.0.0. The index scans 776 non-generated reference files and hashes exact bytes, including fixtures. It does not assert that this local tree corresponds to a particular public Git revision. Provenance verification is maintained separately in [verification.md](verification.md) and `docs/baseline/reference-manifest.json`.

The current source has 209 production modules, including CSS and declarations, and 16 declaration files. The AST index contains 770 declaration/member records: 109 declarations, 183 option members, 38 event members and 440 other members. Counts include browser/headless duplicates because those are independent contracts. These are not counts of implemented Rust APIs.

The test index has 120 files: 73 unit files, 25 browser integration files, six benchmark files and 16 runner/support files. It retains 2,449 static test definitions, including registrations in shared renderer helpers. Static definitions do not equal expanded runtime cases or assertion counts. Loops, shared-helper injection, conditional registration and benchmark contexts have different execution semantics.

All fourteen addon directories are discovered from disk, not selected from a working subset: attach, clipboard, fit, image, ligatures, program-status, progress, search, serialize, unicode-graphemes, unicode11, web-fonts, web-links and webgl.

## Machine-readable contract

Every record is repository-relative and links the unchanged source. Exact paths, current counts, statuses and gaps are authoritative in these files:

| Artifact under `results/compatibility/` | Contents |
| --- | --- |
| [summary.json](../results/compatibility/summary.json) | Counts by family/category/status, focused-criteria gaps and limitations. |
| [manifest.json](../results/compatibility/manifest.json) | Every retained reference path, byte count and SHA-256. |
| [modules.json](../results/compatibility/modules.json) | Every common/headless/browser/addon/declaration/CSS module with implementation, tests, command, evidence, binding, remaining gap and focused acceptance references. |
| [apis.json](../results/compatibility/apis.json) | Every declared public contract/member/overload with module, owner, source line, exact signature, documentation, modifiers, option/event classification, experimental-documentation and runtime-gate flags, binding/evidence or explicit gap. |
| [suites.json](../results/compatibility/suites.json) | Every suite/support file, nested case titles and source lines, dynamic/conditional registrations, benchmark definitions, implementation/test/evidence mapping and explicit original/Rust execution accounting. |
| [fixtures.json](../results/compatibility/fixtures.json) | Every escape-sequence fixture/support file, exact bytes/hash and explicit Rust-runner gap. Includes disabled `.in_` inputs and expected HTML/text. |
| [addons.json](../results/compatibility/addons.json) | Every addon and all its modules/declarations/suites, with missing integrated-addon evidence visible. |
| [runtime-contracts.json](../results/compatibility/runtime-contracts.json) | Browser/headless source proposed-API guarded members and constructor-only options. |
| [evidence-manifest.json](../results/compatibility/evidence-manifest.json) | Exact hashes for mapped Rust code, tests, documentation and evidence at generation time. Presence/hashes are not pass certification. |

`tools/compatibility/mappings.json` is the reviewed implementation mapping input. Missing rows are emitted automatically with empty implementation/test/evidence lists, null command/binding and a stated gap. Mapped rows remain `partial`; no row is promoted to equivalence because a filename or similarly named type exists. Source paths, mapping uniqueness and every referenced implementation/test/evidence path are checked before output.

`tools/compatibility/behavior-criteria.json` supplies explicit focused criterion IDs and checklist documents. Rules match exact source paths and, where provided, contract/case patterns. An empty `behaviorCriteria` list carries `acceptanceGap`, not an umbrella criterion substituted for a missing branch checklist. A source-level rule links the checklist for that source; it does not certify that every listed criterion has an assertion for every case. New focused mappings require review of the actual checklist. The acceptance registry maintained by planning is `results/acceptance/criteria-index.json`. The generator imports only explicit full source-file paths from a focused criterion's `source` field, not task titles, directories, globs or similarly named files. It also consumes the owners' exact utility-addon, font-addon, Unicode-addon and browser-input mapping files, all hashed in the evidence snapshot. Documentation-only source fields without explicit path rules remain unresolved.

The inventory preserves exact declaration signatures and their documentation, including nested callbacks/unions/object types, optional markers, readonly/static modifiers and overloads. It records documentation mentioning experimental/proposed APIs separately from actual runtime calls to `_checkProposedApi`. In the current browser and headless wrappers, `unicode` is guarded and `cols`/`rows` are constructor-only. Their Rust public binding behavior remains unproven.

## Scoped implementation and language differences

The detailed per-module records link implementation and commands. These documents explain current Rust bindings and important differences:

| Subsystem | Mapping and evidence guide | Still not established by the helper checks |
| --- | --- | --- |
| Lifecycle | [lifecycle.md](lifecycle.md) | Rust drop adds cleanup; JavaScript throw/Rust panic boundaries and terminal disposal require their own checks. |
| Collections | [collections.md](collections.md) | Idle scheduling, callback mutation, pending deletion errors, NaN ordering and inherited/prototype keys have explicit gaps. |
| Text decoding | [text-decoder.md](text-decoder.md) | Streaming WriteBuffer/parser/Terminal cooperation and bindings. |
| Unicode 6 | [unicode-v6.md](unicode-v6.md) | Complete service/public-view/addon/browser integration. |
| Cells/attributes | [cell.md](cell.md) | Public BufferCell API and full BufferLine/terminal integration. Signed JS flag values need binding conversion. |
| Colors | [color.md](color.md), [xparse-color.md](xparse-color.md) | Real canvas fallback, themes/rendering and terminal OSC integration. Typed channels do not accept arbitrary JS numbers. |
| Params | [params.md](params.md) | Fractional/nonfinite JS input and zero-capacity resetZdm have documented typed-boundary differences. |
| String builders | [string-builder.md](string-builder.md) | Numeric policy and complete shared parser/terminal integration have separate criteria. |
| Keyboard/Win32 | [keyboard.md](keyboard.md), [win32-input.md](win32-input.md) | WebIDL event conversion, browser routing and native adapter behavior. |
| Charsets | [charset.md](charset.md) | External mutable map aliases and live InputHandler designation/print/save/restore checks. |
| OSC/DCS/APC | [osc.md](osc.md), [dcs.md](dcs.md), [apc-port.md](apc-port.md) | Rust futures require an executor to poll pending cleanup. Outer parser and Terminal/browser ordering need integrated tests. |

Bindings must deliberately preserve UTF-16 units and lone surrogates, undefined versus empty values, JS signed bitwise representation, Result/error timing, shared object identity and explicit disposal. These differences are recorded rather than hidden by idiomatic Rust names.

## Execution evidence is separate

For original unit files, `suites.json` resolves `results/upstream-baseline/latest.json` to its unique run summary before copying per-source counters, including original pending cases and corrected imported-test registration attribution. It never silently falls back to the legacy fixed summary or a historic pass if the latest attempt failed or is incomplete. Before labeling counts recorded, it validates pointer/run identity, successful commands without timeouts/resources/errors, before/after reference checks, the retained source manifest against current bytes, package-lock identity, raw report SHA-256, nonnegative per-suite accounting and exact sums/raw statistics. Invalid evidence fails generation before artifacts are written. It does not manufacture counts from static definitions. Absent original browser/benchmark evidence remains null with a reason. Rust full-suite counts also remain null unless independently accounted elsewhere; scoped evidence links do not imply that every source assertion in a larger suite was migrated. In particular, CellData migration does not discharge the rest of BufferLine.

The index does not run native Windows/Linux/macOS, browser engines, performance workloads or interactive demos. It does not certify unsafe code or full end-to-end terminal state. Follow [verification.md](verification.md) for observed commands, raw execution evidence, review findings and remaining gates.

## Reference exclusions and update procedure

The walker excludes `.git`, `node_modules`, `out`, `out-test`, `out-esbuild`, `out-esbuild-test`, `lib` and `coverage` directories, plus the exact generated path `source_repo/demo/dist`. It does not exclude arbitrary directories named `dist` elsewhere. These are VCS/dependency/generated outputs in the supplied reference. Symlinks fail explicitly instead of disappearing from the index. Generated build output does not count as new source coverage. The scanner includes all other reference files in its hash manifest; production modules and suites are filtered by their documented source locations/extensions.

When implementation or acceptance changes, inspect the source and its full assertion path, update the explicit mapping/checklist references, generate, run `--check` and the seven inventory tests, then obtain independent review. A newly added or removed production module changes the generated path set. A changed signature, test title, fixture byte or evidence file changes generated content/hashes. Both make `--check` fail. The declaration count anchor in the test suite also requires an explicit reviewed update if the reference changes.
