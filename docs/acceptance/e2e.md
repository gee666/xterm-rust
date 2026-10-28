# Actual native and browser E2E acceptance

Task 46 owns execution, not merely test files. Tasks 45 and 31 provide the built native example and Rust-backed browser application. Original baselines are owned by task 44. A configured workflow, mocked DOM, direct provider test, cross-compile or successful startup cannot pass these checks.

## E01. Native application round trip

- Start the documented built example using the same commands a user follows from a clean checkout. Retain executable hash, argv, environment, OS/architecture and process exit/timeout details.
- Send printable input, CR/LF, backspace, controls, cursor-moving CSI, erase/insert/delete and SGR. Assert the library's resulting cells/attributes/cursor and the visible output or supported display representation, not a fixed startup escape string.
- Send UTF-8 bytes split at every byte of representative two/three/four-byte characters and a malformed sequence, then verify codepoints and widths. Exercise Unicode provider selection and combining/wide text.
- Resize smaller and larger while output and input are active; verify buffer/scrollback/cursor/reflow invariants, alternate-screen entry/exit and return to normal state.
- Verify interactive input reaches Rust, Rust replies/events reach the host, callback order is retained and cancellation/EOF/errors shut down child services and restore host state. Test repeated open/dispose, noninteractive invocation, invalid dimensions and terminal reset.
- Run actual runtime checks independently on native Windows, Linux and macOS. Retain concrete platform identity and artifacts. Missing native execution stays unrun, never a pass inferred from portable code or CI YAML.

## E02. Browser identity and visual matrix

- Build the actual WASM/browser distribution, load the Rust-backed demo and identify the module and binding hashes. Assert no xterm.js terminal engine is used by the Rust terminal. Original pages may run only as separately identified oracle pages.
- Run Chromium, Firefox and WebKit where the source supports them, with actual engine versions and logs. Use the same original project settings, dimensions, scrollback, font, DPR and renderer options. Explicitly label an unavailable engine; never omit it silently.
- Port the original `test/playwright/{Terminal,InputHandler,Parser,CharWidth,Renderer,MouseTracking}.test.ts` and shared renderer assertions without dropping expected values. Retain exact discovered/mapped/executed/pass/fail/skip/unrun counts and supporting fixtures.
- Assert initial canvas/DOM ownership, terminal open before/after addon loading, focus/blur/textarea positioning, body versus iframe documents, repeated open/dispose and absence of surviving listeners/canvases after disposal.
- Render default/palette/truecolor/inverse/dim/bold/italic/underline/strike/blink/invisible text, selection and decoration layers, joined/combined/wide cells and cursor styles. Compare actual pixels or DOM drawing outputs and retained screenshots with explicitly defined tolerances, not only nonblank screenshots.
- Resize and change DPR/font/theme/options after rendering. Assert geometry, atlas invalidation, redraw ranges, scrollback/viewport behavior and no stale cells. Verify suspended rendering/visibility and synchronized-output transitions when applicable.

## E03. Browser interactions and accessibility

- Generate real browser keyboard events including printable/control/modifier/function/navigation keys, Mac Option variants where applicable, cancellation/custom key handlers and protocol modes. Assert exact emitted data and source event ordering.
- Exercise IME composition start/update/end, surrogate text, delayed update/cancellation, textarea movement, focus changes and disposal while a callback is pending. Record actual input payload and no duplicate text.
- Exercise mouse press/release/move/wheel/touch under each supported tracking/encoding mode. Assert exact replies, selection suppression, coordinate boundaries, modifiers and scroll-wheel routing.
- Test selection forward/backward, word/line/column, wrapped/wide/combining text, trim/scroll/resize updates, select-all/clear and out-of-range coordinates. Check exact selected text and UTF-16 boundaries.
- Copy, paste and context-menu paths must use browser clipboard events/providers. Assert bracketed paste, newline normalization, empty/multiline/lone-surrogate cases, async denial/errors and disposal while awaiting clipboard access.
- Test OSC8 and provider/web links, wrapped links, URL normalization, hover decorations, modifier activation, provider priority, stale asynchronous replies, listener disposal and clicked URI/range.
- Verify accessibility DOM roles/live announcements, row updates, focus traversal, screen-reader mode, selection/scroll updates, character limits and disposal through actual browser accessibility/DOM assertions.
- Include negative paths: invalid public numeric arguments, proposed-API access disabled, option mutation errors, unhandled/malformed sequences, callback error propagation, host denial and async work after terminal disposal. Verify state after the error, not just that an error occurred.

## E04. Complete addon integration

- Original applicable addon browser suites execute against the original and equivalent Rust-backed APIs. List all 14 addon names and each activation/disposal, registration order, option mutation, error and live terminal behavior case. Unsupported cases remain named gaps.
- Attach exercises real socket lifecycle/data directions and binary/text semantics. Clipboard exercises OSC52 and host clipboard; fit measures live DOM sizing. Search traverses real buffers/decorations/results and serialize round-trips real cells/modes/cursor/HTML.
- Unicode11/graphemes validate registration/restore/proposed gates plus every original width assertion. Web-links verifies real wrapped cell ranges and URI activation. Program-status/progress verify actual parser registrations, ordered events and reset/disposal.
- Web-fonts/ligatures load fonts and verify shaped output/joiner lifecycle. Image tests decode actual SIXEL/IIP/Kitty fixtures into real buffer tiles and visible layers. WebGL tests use actual GPU resources, atlas events, pixel output and context loss/restoration. Portable-only helper tests do not pass these browser cases.
- Exercise supported addon combinations in both activation orders, renderer replacement, normal/alternate buffers, resizing and terminal-before-addon/addon-before-terminal disposal. Retain a failing trace/screenshot for mismatched behavior.

## E05. Harness failure and receipt integrity

- Server/child processes have explicit readiness conditions and bounded startup/test/shutdown timeouts, not sleeps or unbounded retries. Cleanup executes on test failure, browser crash, timeout and interruption. Do not leave background services running.
- Deliberately fail a page assertion, crash a browser, bind a used port, reject a socket/clipboard request, withhold readiness and return malformed test output. The harness must report nonzero failure and retain logs, not turn missing results into a pass.
- Each run records source/build/fixture hashes, seed and trace counts, browser/OS/tool versions, viewport/DPR/options, exact commands, failed attempts, screenshots/traces and independent review identity. Confirm the build matches the source snapshot; rerun final integration on a stable snapshot.
- Reconcile suite totals with the original inventory. Static definitions are not runtime assertions or parameterized cases. Browser runs against original JavaScript are baseline evidence only.
- E2E success proves tested behavior in the observed environment. Overall equivalence remains NOT PROVEN while any required implementation, source mapping, suite, native OS, browser engine, safety check or comparison is unresolved.
