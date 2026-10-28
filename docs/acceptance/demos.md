# User-run demo acceptance

Task 45 owns the native example, Rust-backed browser demo and root README. These checks do not replace task 46's full native and three-browser E2E matrix. No demo is accepted while it uses a placeholder host or JavaScript terminal engine.

## Native interactive path

- A clean checkout follows copyable documented Rust/MSRV and oracle prerequisites. The example builds with a locked dependency graph and uses `xterm_core` live terminal state. It must not reimplement parsing or only print canned escape strings.
- Starting the example shows help, dimensions and a readable viewport. An initial sample passes real ANSI color, cursor movement and mixed ASCII/astral/wide/combining Unicode through the Rust parser and displays the resulting cells and attributes.
- A user can send text and input, observe data-event output separately from incoming display writes, move the cursor and change terminal dimensions. A resize visibly uses reflow/live buffer state rather than truncating an independent string copy.
- Commands have explicit syntax and validation. Unknown commands, malformed escapes, invalid dimensions and failed writes return useful errors without corrupting state or silently succeeding. Empty input, EOF and explicit quit exit deterministically; no unbounded background service survives.
- A deterministic script mode drives the exact same command/engine path. Assertions inspect real cells, attributes, cursor, dimensions and emitted data, not merely startup or expected text echoed by the driver. Native Windows execution is observed and retained; Linux/macOS results remain unrun until actually executed.

## Browser path

- Build Rust WASM and binding JS with documented commands and serve a static, local demo with a bounded server and explicit shutdown. Do not import the source_repo JS engine. List every mandatory artifact including WASM, glue and CSS; failures to fetch or initialize show a visible error.
- The public Rust-backed Terminal opens into the documented DOM container and renders real parsed cells. ANSI color, cursor movement and Unicode are visible after writes, and a user can type input with separately observable outbound data.
- Resize controls use the real terminal API. They reject invalid dimensions, show the new dimensions and visibly update the viewport. Reset and disposal release the old host/listeners; reopening does not duplicate input or leaked nodes.
- A deterministic browser test asserts DOM/render state plus Rust-backed buffer/cursor results. It exercises an input action, outbound data, resize and disposal/reopen, records screenshots/results and checks console/network failures. Chromium/Firefox/WebKit runtime results are required for task 46; a WASM compile or host DOM smoke test is not a substitute.

## Demo-05 live option/style/search/serialize controls

Source references are `source_repo/demo/client/components/window/optionsWindow.ts`, `styleWindow.ts`, `addonSearchWindow.ts` and `addonSerializeWindow.ts`. Controls must operate on the same visible Rust-backed Terminal instance.

- Enumerate live scalar options instead of a disconnected defaults copy. Boolean/number/string controls apply actual option setters and show validation errors. Include cursor style/inactive style, blink, font family/size/weights, line height/letter spacing, transparency and scrollbar controls, plus nested Kitty and Win32 extension toggles. Preserve siblings when replacing nested bags.
- Theme controls select default/xtermjs/sapphire/light values on the live terminal and update container background. Transparency makes the actual background transparent. Font changes trigger real measurement/fit and rerender, not only a label. Renderer controls switch actual DOM/WebGL ownership and preserve buffer/cursor/selection.
- Size accepts explicit `COLSxROWS` and auto-fit. Explicit size visibly outlines the screen; invalid input follows the documented source auto-size behavior. Padding changes the terminal container and therefore fit dimensions.
- Search next/previous controls include regex, case-sensitive, whole-word and highlight-all toggles. Assert actual selection/ranges and result counts against the displayed wrapped Unicode/ANSI text, including failed search, wrap direction and reload.
- Serialize displays actual ANSI output. Optional write-back resets and writes that same output into the same terminal; assert complete cells, attributes, wraps and cursor survive a matched-size round trip. HTML serialization displays actual markup and writes HTML to the clipboard with visible success/failure and removed temporary listeners.
- `node scripts/demo-e2e.mjs --case demo-05` must exercise the controls with nonzero state/render/selection/roundtrip assertions and retain browser results. A control updating only JS state, synthetic preview text or another hidden terminal fails this gate.

## Demo-06 addon reload and teardown controls

Source references are `source_repo/demo/client/components/window/addonsWindow.ts` and `source_repo/src/common/public/AddonManager.test.ts`, with the demo's actual addon collection/activation wiring. Every loaded addon is attached to the same visible Rust terminal, with stable explicit ownership.

- Provide load, unload and reload controls for image, WebGL, Unicode providers, fit, web links, clipboard and attach. Also expose the supplied search/serialize, fonts/ligatures and program-status/progress addons when exercising their demo paths; unavailable implementations remain visible failures, never successful inert controls.
- After every transition send real output/input and observe the affected renderer, Unicode widths, fits, link discovery, clipboard OSC52 or socket traffic. Reloading restores one active registration, not duplicate parser hooks, socket sends, click listeners, render jobs or event subscriptions.
- WebGL unload removes its canvases/context resources and returns to the actual remaining renderer. Image unload releases image state/listeners, and Unicode unload restores the documented prior active provider. Fit/link/clipboard/attach disposals stop their actions without disposing an external socket or the terminal unless upstream owns it.
- Dispose then reopen the terminal, reload addons and rerun the sample. Assert DOM/canvas counts, input/output callback counts and active state are fresh, and old handles cannot mutate the new terminal.
- `node scripts/demo-e2e.mjs --case demo-06` must execute each required transition, not just toggle checkboxes. Retain real output/event/resource assertions and teardown evidence.

## README and failure checks

- Provide clean-checkout commands for prerequisites, cargo tests, differential checks, browser build/tests, release benchmark/size/LoC scripts and both demos. Commands reference actual files and flags. State how the exact supplied source reference is obtained and manifest-checked; do not invent a public source commit.
- Every documented command is executed or explicitly labeled unrun with a concrete reason. Missing engine handlers, browser bindings or unimplemented examples are work gaps, not external blockers.
- Deliberately test a missing WASM asset, wrong dimensions and a malformed native command. Neither demo may show a successful supported API result while doing nothing.
- Keep raw logs, exact environment/commands and screenshots under `results/demos/`. Independent review runs the example and verifies it invokes the Rust engine. Overall equivalence remains NOT PROVEN while any full-project requirement is missing.
