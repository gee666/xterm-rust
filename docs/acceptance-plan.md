# Acceptance plan

The scope is the supplied xterm.js 6.0.0 tree, not a small headless terminal. Full equivalence is not proven. This plan records the acceptance requirements and task ownership boundaries. `PORT_REQUIREMENTS.md` remains the detailed authority.

## Grouped acceptance criteria

| Criterion | Group | Pass condition and required evidence |
| --- | --- | --- |
| 1 | Planning | Full intent in `docs/intent.md`; this plan covers every requirement; implementation, testing, documentation and comparison work have linked tasks and independent review. |
| 2 | Implementation | Portable, buildable Rust engine, headless APIs, actual WASM/browser contracts and all 14 addons. Every upstream behavior is implemented, without JS terminal-engine delegation or successful placeholders. Preserve attribution, UTF-16, number boundaries, streaming, callbacks, proposed APIs and lifecycle. |
| 3 | Compatibility | `docs/compatibility.md` and machine-readable inventory enumerate every source module, declaration, option, event, proposed API, addon and test suite. Each maps to implementation and executable evidence or an explicit gap; source completeness is checked. |
| 4 | Verification | Original suites execute unchanged as baselines. Every assertion is faithfully mapped and migrated. Differential traces compare cells/codepoints/widths/attributes, buffers/cursor/modes/scrollback, events/replies/API/lifecycle and all addons. Deterministic and seeded randomized malformed/chunked inputs include Unicode, resizing/reflow and alternate screens. Keep seeds, counterexamples and exact discovered/mapped/executed/pass/fail/skip/unrun counts. |
| 5 | Integration | Real native demo and Rust-backed browser E2E pass. Exercise rendering, keyboard/mouse/IME, selection, clipboard, accessibility, resizing, links, disposal and addons in supported Chromium, Firefox and WebKit. Actual native Windows/Linux/macOS build and runtime results are required; cross-compilation or CI configuration is not evidence of runtime success. |
| 6 | Quality | Warning-free fmt, Clippy, workspace tests, doctests, release builds and relevant features/targets/WASM/browser checks. Follow safe Rust, bounded functions/files, typed errors, RAII, callback/reentrancy and license rules. Review real code and evidence independently; every justified unsafe operation gets focused validation, with unsupported checks named. |
| 7 | Demonstrations | README gives copyable clean setup, tests, oracle/differential, browser and benchmark commands plus working native and Rust-backed browser demos. Both visibly exercise color, cursor movement, Unicode, resizing and input/output through Rust logic. Document exact oracle acquisition/provenance and manifest verification. |
| 8 | Comparisons | Checked-in scripts and `docs/comparison.md` compare identical validated workloads, dimensions, scrollback, Unicode setup and operations. Production JS and release Rust, warmup and at least ten measured repetitions, median/p95, units/ratios, startup inclusion, environment, raw data and timing method. Include ASCII, Unicode/graphemes, ANSI, scrollback, reflow/resize and applicable addon workloads. Browser rendering comparisons use equivalent browser harnesses. Report all required distribution files, native versus web distinctions, matched features and gzip/Brotli settings/totals; missing measurements are blocked, never zero. |
| 9 | Comparisons | One reproducible LoC method across both trees with files, bytes, physical/nonblank counts and labeled comment-stripped counts if available. Separate production library, addons, bindings/glue, generated data, tests, demos and tooling, including all required languages. Explain differences from `docs/baseline/source-loc.json`. |
| 10 | Evidence | `docs/verification.md` and tracked non-secret raw results contain exact commands, environment/tools, seeds/counts, logs/artifacts, independent findings and failures/blockers. Assert NOT PROVEN while any gap remains. Preserve source manifest and provenance; no invented commits, measurements or platform passes. |
| 11 | Input | Win32 input encoding preserves upstream key mappings, modifiers, UTF-16, formatting and all original assertions, with focused executable evidence. |

## Implementation and evidence task map

Existing component owners retain their files. These are ownership boundaries, not permission to skip unlisted supporting modules. Inventory task 22 must flag omissions, and the responsible family owner must include them or create linked follow-up work.

| Family | Tasks | Ownership and integration |
| --- | --- | --- |
| Intent, workspace and planning | 1, 2, 6 | Cornelia intent; Lukeni acceptance plan; Chloe root/core manifests, exports and workspace contracts. |
| Common data/utilities | 3, 4, 5, 7, 12, 13, 14, 16, 20, 21 | Collections, colors, lifecycle, UTF decoders, CircularList, XParseColor, events, string builders and charsets. Existing owners retain their modules. |
| Cells and buffer lines | 11, 24, 52 | AttributeData/CellData and BufferLine, plus independent BufferLine oracle/differential verification. Coordinate mutation and packed-data contracts before integrating Buffer. |
| Unicode | 10, 23, 41 | Shared `unicode.rs` provider/property contract, default V6, UnicodeService/API, Unicode11 and grapheme addons. |
| Keyboard/options | 9, 19, 25, 51, 54 | Win32 encoding, legacy keyboard, Kitty implementation 51 and independent Kitty oracle/review 54, plus options defaults/validation/events. |
| Scheduling/platform | 58 | TaskQueue cancellation and scheduling, Platform, WindowsMode buffer behavior, Version and remaining common Types/TestUtils mappings. |
| Mouse protocols | 55, 56, 57 | Tuah 55 owns independent Rust differential/review, Sophie 56 owns the oracle and Sophie 57 owns MouseStateService implementation. Excluded from task 29 to avoid duplicated ownership. |
| Parser | 8, 15, 17, 18, 26, 53 | Params, OSC/APC/DCS, main escape parser 26 and independent VT500 transition tables/constants 53. Define async pause/continuation contracts without dropping synchronous behavior. |
| Buffer/terminal/core | 27, 28, 29, 30 | BufferSet/Buffer/reflow/ranges/markers/BufferService; InputHandler/CoreTerminal/WriteBuffer/Async; remaining services/common public APIs; headless/public terminal integration. |
| Browser/WASM | 31, 32, 33, 34, 35 | WASM/bindings/public browser terminal; DOM rendering/themes/decorations; input/IME/clipboard; selection/scrolling/accessibility; links/scheduling/support. Core never depends on browser adapters. |
| Search and serialize | 36, 37 | Actual Rust buffer integration, source APIs, events and faithful original/addon assertions. |
| Image and WebGL | 38, 39 | Full image protocols/storage/rendering and GPU rendering/atlases/context lifecycle. Browser proof is required. |
| Fonts/shaping | 40 | Ligatures and web-fonts, including real loading, joiners and disposal. |
| Browser utility addons | 42 | Attach, clipboard, fit and web-links. Coordinate bindings with task 31, without assuming browser logic is implemented elsewhere. |
| Status addons | 43 | Program-status and progress protocols, states, callbacks and lifecycle. |
| Inventory/baseline/differential | 22, 44 | Source completeness and trace verification. Baseline reference must remain unchanged. |
| Demos/E2E/CI | 45, 46, 47 | Runnable native/browser examples, actual end-to-end execution, quality and native platform evidence. Task 46 links both integration criterion 5 and suite-execution criterion 4. |
| Comparisons/final audit | 48, 49, 50 | Reproducible speed/size, classified LoC and final evidence/gap audit. |

## Shared contracts and files

- `docs/workspace.md` defines Rust 2021, MSRV 1.85 and core exports. Ask Chloe before touching the manifests or `crates/xterm-core/src/lib.rs`.
- Use `crates/xterm-core/src` for portable code and `crates/xterm-core/tests` for assertions. Browser adapter task 31 owns its crate manifests/bindings. Each task must publish concrete file ownership before edits.
- Events use the shared event/lifecycle modules, including synchronous callback ordering and disposal. Unicode providers share `unicode::{UnicodeVersionProvider, UnicodeCharWidth, UnicodeCharProperties}`.
- Agree parser/buffer/terminal/browser contracts early through linked posts and direct messages. No agent should create a competing terminal engine, event system or width implementation for convenience.
- Each implementation task owns faithful tests and scoped differential evidence, not just production code. Tasks 44, 46 and 47 integrate those results and check interactions; they do not replace component assertions.
- Results go under tracked `results/`; temporary files and diagnostic logs under root `tmp/`. Keep local credentials and swarm runtime data out of commits. Never modify AGENTS.md or source assertions.
- Add task dependencies once a shared prerequisite is known. Waiting is not an external blocker. An owner may develop an independent contract in parallel, but must not report integrated runtime behavior before it exists.

## Review and completion

Every task requires an independent reviewer who inspects actual files and executes the documented commands. Review evidence names paths, commands and results. Coverage votes judge whether all linked tasks, when complete, cover the criterion; a covered vote is not a claim that implementation already passes.

All active criteria must have linked work. Finish only when all tasks are accepted or deferred under agreed external blockers, all criteria are satisfied or blocked, and no review remains. A blocker records the missing resource/decision, attempts and exact unblocking condition. Missing implementation is work, not a blocker. Native platform access, credentials or unavailable browser runtimes may be external limitations only after recorded attempts.
