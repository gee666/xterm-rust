# Browser serialize acceptance

Task 131 owns the JS-callable addon, not just a Rust helper. Reference `source_repo/addons/addon-serialize/typings/addon-serialize.d.ts`, `src/SerializeAddon.ts`, `src/SerializeAddon.test.ts` and `test/SerializeAddon.test.ts`. The complete checklist in `docs/serialize-acceptance.md` still applies. All cases below need source/Rust inputs, outputs or thrown values, exact counts and retained failures. No browser binding currently exists.

## SB01 Callable class and real host

Export a constructible SerializeAddon with activate, serialize, serializeAsHTML and dispose. Activation receives the actual Rust-backed browser Terminal; no production code runs the original JS terminal engine. Match default/explicit arguments, reactivation and before-load errors. Exercise the real Terminal.loadAddon manager's dispose replacement, repeated disposal and terminal disposal. An algorithm-only test is not callable API evidence.

The source buffer API is ungated in both browser/public/Terminal.ts:97 and headless/public/Terminal.ts:87. Default allowProposedApi false and true-to-false changes must not reject serialization. Do not infer a gate from UnicodeApi or stale typings.

## SB02 JS property and exception order

Preserve optional/nullish behavior, primitive property boxing and original thrown values. Reflect.get on an unboxed primitive is not equivalent to source property access. Do not stringify a thrown object into a new Error.

ANSI serialize reads options.range for its condition, then again for the range argument. Normal buffer access occurs before evaluating that second range read. Read excludeAltBuffer only after normal serialization, then inspect the current active buffer. Read excludeModes after normal/alternate serialization. Getter side effects or exceptions at those points must not be moved ahead of earlier serialization phases.

HTML captures the public active buffer view and palette source before onlySelection and range property reads. Range wins over selection and scrollback. includeGlobalBackground is read at header generation, not before buffer/range selection. Evaluate foreground and background as separate options.theme lookups, retain both raw values, then apply default-hint concatenation in source order. Font family and font size also use concatenation, not String-hint or Number-hint coercion. Check custom Symbol.toPrimitive, valueOf/toString, Date, BigInt, Symbol, null, undefined, false, zero and thrown sentinels. Raw header strings must retain lone UTF-16 units.

## SB03 Number and marker boundaries

Do not silently cast arbitrary JS numbers to usize/isize. Cover negative zero, negative values, fractions, NaN, both infinities, safe-integer boundaries and array-length boundaries. A valid source array length is not the same as a valid buffer row. Reversed/fractional span errors preserve RangeError, while missing next-line wrap reads preserve TypeError and its exact message.

Read marker-versus-number values at the same property access points as source. Cover disposed/trimmed markers and line getters with side effects or exceptions. Explicit range overrides scrollback and excludes final cursor movement. HTML startCol applies only on the first row; final columns follow source dimensions or selection, not an invented endCol option.

Use the already ported public number-indexing helpers where needed. Source BufferApiView.getLine delegates to CircularList, including signed ring arithmetic and stale/wrapped slots; source public cell reads use packed typed-array indexing. Fractions and negative rows cannot be approximated by truncation or clipping. Compare exact source results rather than assuming malformed coordinates must all throw.

## SB04 Public view alias and reentrancy

Retain the actual cached public BufferApiView identity. A raw Rc<Buffer> snapshot is wrong when reset emits activation and reinitializes that view. Capture numeric range boundaries before header hooks, but read cells through the view after those hooks. Alternate activation leaves a captured normal view normal; reset may retarget it to the replacement normal buffer.

Cover header getters performing write/queued write, resize wider/narrower, reset, alternate activation, reading buffer.normal, addon reactivation, option replacement and nested serialization. Preserve which operations are synchronous in the browser. Native InputHandler.parse probes are not proof of browser write event-loop timing.

No mutable or immutable Rust state/resource borrow may span a user getter, coercion, callback or await. Recoverable failures must propagate. Do not turn contention into missing cells or successful empty output.

## SB05 HTML selection and themes

Read the live Rust selection service, including absent, empty, multiline, end-exclusive and trimmed selections. Range takes precedence. Native headless selection absence is not a substitute for a browser selection implementation.

Before open, the source may use its headless palette fallback; after open, use the actual browser ThemeService ANSI palette. Cover low/bright ANSI, the color cube and grayscale, overrides/restoration, OSC colors, foreground/background options, underline colors/styles, inverse and all decoration combinations. A copied stale theme snapshot must not hide source-visible reentrant updates.

## SB06 Diagnostics and UTF-16

Preserve complete UTF-16 output, including malformed strings and whole-cell HTML escaping behavior. Source missing-cell traversal emits console.warn with row/column information before continuing. Do not silently discard that side effect or route it through an option filter that source console.warn ignores. Test warning count/text and error-before-warning ordering for out-of-range columns and header-induced width changes.

## SB07 Host lifetime and cycles

Keep a loaded JS addon usable after the caller releases its separate terminal variable, as the source addon retains its host. Native Rust Drop/TerminalReleased is an explicitly documented native ownership boundary, not proof of this JS behavior. Test real GC/WeakRef behavior where supported and preserve an honest unrun label otherwise.

Avoid Rust/JS cross-heap reference cycles through manager records, callback stores, options or addon host references. Inspect actual ownership, not merely whether a Rust closure captures self. Explicit disposal must release listeners/resources and retain the source's subsequent-call behavior. Do not claim GC parity from a normal explicit-dispose smoke test.

## SB08 Original and differential browser evidence

Run the unchanged original addon browser suite against the Rust-backed class in Chromium, Firefox and WebKit. The observed original 120 source cases are only a baseline. Record discovered, mapped, executed, passed, failed, skipped and unrun counts separately for Rust. Preserve source assertions; keep failures and source/runtime versions.

Add deterministic and seeded operation traces comparing exact ANSI/HTML, complete source/restored Rust cells, codepoints, widths, styles, cursor, scrollback, wrapping, both buffers, modes, margins, events and errors. Account explicitly for source losses such as final DECSTBM homing the cursor; compare against restored source state instead of discarding that case or inventing a fix. Retain seeds and counterexamples.

Run through the actual demo/module export path as well as isolated bindings. Include browser screenshots or DOM/pixel assertions for visible restored content, selection/theme integration and disposal. Build success, native fixture parity and original JS suite success do not prove any of these Rust browser gates.
