# Utility addon acceptance

This checklist controls task 42. A portable unit test does not establish browser integration. Every row needs a named assertion and observed execution result before completion. Source files and assertions remain unchanged. Overall addon equivalence is NOT PROVEN.

## Attach

Reference `source_repo/addons/addon-attach/src/AttachAddon.ts`, typings and `test/AttachAddon.test.ts`.

- Constructor sets binaryType to arraybuffer before activation, including a socket that is not open. Default bidirectional and explicit true register both terminal streams. Only explicit false disables both streams.
- Each activation registers message, close and error listeners. Data listeners are ordered after message and before close/error. Incoming strings preserve UTF-16 units. Incoming ArrayBuffer bytes go to the terminal byte write path, including empty messages and malformed UTF-8 across separate messages.
- onData sends a string only when OPEN. onBinary masks every UTF-16 unit with 255, including NUL, values above 255, surrogate pairs and lone surrogates. Sends preserve event order.
- CONNECTING throws the exact source message; OPEN sends; CLOSING warns once per attempted send without sending; CLOSED throws; unknown state throws. Socket send and terminal write errors propagate.
- Close, error and explicit dispose remove all registered socket and terminal listeners. Repeated dispose is safe with idempotent adapter handles. Disposal inside a callback does not hold a mutable borrow or suppress remaining upstream dispatch behavior. Rust Drop releases registrations. Avoid terminal/socket ownership cycles. Multiple activate and activate-after-dispose retain the source registration behavior and do not silently auto-dispose old subscriptions.
- Port both original browser assertions, string foo and binary UTF-8 foo, through a real WebSocket and a Rust-backed terminal. Add socket-state, unidirectional, repeated activation and disposal browser checks. Do not count a fake socket as WebSocket integration.

## Clipboard

Reference `source_repo/addons/addon-clipboard/src/ClipboardAddon.ts`, typings and `test/ClipboardAddon.test.ts`.

- Activate registers OSC 52. Dispose removes that registration; retained in-flight provider promises still settle and send replies as upstream does. Repeat activation matches source replacement of the saved disposable, rather than implicitly removing previous handlers.
- Split on every semicolon but use only fields zero and one. Missing separator consumes the sequence without provider calls. Empty selection, p and c are passed unchanged to a custom provider. Extra fields do not affect the second field. Exactly ? queries. Empty payload and malformed base64 clear the clipboard.
- Queries call readText before returning. Synchronous providers reply synchronously; asynchronous providers reply only after resolution. Replies are exactly ESC ] 52 ; selection ; UTF-8-base64 BEL and use terminal.input with wasUserInput=false, not write. Rejected read/write and encode errors propagate. Decode exceptions alone are converted to clearing, as explicitly required by the source.
- Encoding uses TextEncoder-compatible UTF-16 replacement for lone surrogates, astral characters and NUL. Decoding uses forgiving base64 padding/ASCII-whitespace rules, accepts nonzero unused bits as atob does, rejects other whitespace/nonalphabet/padding errors, replaces malformed UTF-8 by maximal subparts and consumes one leading BOM as TextDecoder does. Cover empty input and every byte value.
- BrowserClipboardProvider forwards all selections to navigator.clipboard, with observed permission success and rejection. Preserve all ten original browser assertions for read/write, p/c/default selection, invalid/empty values, clear and non-ASCII. Record upstream WebKit skip separately; do not claim a Rust pass for it.
- Native custom codec/provider tests and JavaScript differential traces supplement, not replace, browser permission and lifecycle tests.

## Fit

Reference `source_repo/addons/addon-fit/src/FitAddon.ts`, `browser/shared/Constants.ts`, typings and `test/FitAddon.test.ts`.

- Before activation, missing element, missing parent, missing render dimensions, zero cell width or zero cell height returns no proposal. A measured hidden terminal follows actual source behavior; do not force a fabricated default.
- Styles come from each node's ownerDocument.defaultView, falling back to window only when absent. Test iframe-owned nodes. Read height/width with decimal parseInt, clamp parent sizes to zero and treat invalid/empty CSS as zero. Padding uses decimal parseInt or zero, including negatives, fractions, signs, CSS units, exponent text, leading JS whitespace and huge values.
- Scrollbar width is zero for scrollback=0 or showScrollbar=false. Otherwise use explicit width, including zero, or default14. Test absent scrollbar object and absent individual properties.
- Subtract top/bottom and right/left padding and scrollbar before division; floor and clamp to minimum2 columns/1 row. Preserve fractional cell dimensions and JS NaN/Infinity behavior. fit rejects NaN but otherwise forwards dimensions and adapter errors exactly. A second fit at the same dimensions still calls resize and does not throw.
- Dispose has no observable state change in the original, so fit/proposal after dispose still works. Rust must not invent invalidation.
- Port all nine browser cases, including no terminal, default/width/small/hidden proposals and default/width/small/same-dimensions fit, without weakening bounds. Add DOM removal, zero measurement, custom scrollbar and iframe cases.

## Web links

Reference `source_repo/addons/addon-web-links/src/{WebLinksAddon,WebLinkProvider}.ts`, typings and `test/WebLinksAddon.test.ts`.

- Preserve the exact strictUrlRegex, including only lower-case http/https and all-upper-case HTTP/HTTPS, initial/final punctuation exclusions, IPv6 brackets and internal tilde. Custom RegExp preserves its source/flags and receives g if absent. Matching must preserve JS UTF-16 indices. Test zero-length custom regex behavior explicitly rather than introducing an unbounded loop.
- Candidate validation uses WHATWG URL, followed by the source reconstructed authority prefix check with case folding. Cover username, username/password, default/nondefault ports, uppercase, percent encoding, Unicode host normalization, IPv6, malformed hosts, unsupported schemes and trailing punctuation. A Rust URL approximation is not proof of WHATWG equivalence.
- Requested y is one-based. Pull current line trimmed-right; expand upwards only when current is wrapped and does not begin with ASCII space. Stop on missing line, first unwrapped line, any ASCII space, or the source 2048 UTF-16 expansion boundary. Preserve source top-index behavior when expansion reaches missing lines. Expand downwards through wrapped lines until space, missing line or 2048 boundary. Other whitespace does not substitute for ASCII space at these checks.
- Map JS string indices through full cells, combined strings, empty width1 cells and width0 continuations. Length is UTF-16, not Unicode scalar count or bytes. Correct an empty final cell only when the following wrapped line begins width2. Handle offset0, exact cell/end/line boundaries, negative remainder and missing line. Return one-based inclusive end and one-based start ranges exactly.
- Hover passes the computed range; leave passes event and URI. Custom activate handler receives the original URI. Default opens a blank window, clears opener before navigation, handles opener setter exceptions as upstream, and warns on blocked window. Dispose removes provider registration, including repeated calls.
- Port all country-TLD and .com-country generated cases from the unchanged test file, including all seven path variants. Preserve exact wide/combining/wrapped-offset assertions and all five uppercase/default-port assertions. Real hover must underline Rust-rendered cells, not just return a policy match.

## Evidence gates

For each addon record discovered/ported/executed/passed/failed/skipped/unrun counts, source hashes, exact commands and browser versions. Deterministic differential corpora retain a seed and mismatches. Run fmt, Clippy with warnings denied, crate tests, workspace tests and relevant WASM build. An independent reviewer inspects actual source and tests and executes them. Missing browser adapters, WASM runtime, browser engines or native OS evidence stay explicit gaps. Task42 cannot be accepted on portable tests alone.
