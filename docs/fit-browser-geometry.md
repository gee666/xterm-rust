# Fit browser geometry

Overall terminal and utility-addon equivalence remains NOT PROVEN. This check covers default-font Fit geometry, not clipboard, attach, links, raw decorated options or every Fit lifecycle path. Full utility/status closure remains task323.

## Findings

The unchanged original Fit browser suite and the actual Rust WASM terminal both execute27 cases on Windows. Each has17 passes,10 failures, zero skips and zero unrun cases. Chromium and Firefox each pass4 of9; WebKit passes9 of9. Source assertions and terminal font defaults remain unchanged.

Eight failures are the original default/width column bounds in Chromium and Firefox. Their default `monospace` font measures differently from the font expected by the original numeric bounds. The two remaining failures are the original repeated-fit assertion after the preceding small-container case. These fail against JavaScript too. They are not evidence of a Rust-only resize bug.

The direct trace compares12 complete measurement snapshots per browser, including proposals, terminal columns/rows, CSS/device dimensions, default font options and screen inline styles. All36 source/Rust snapshot pairs agree exactly in Chromium, Firefox and WebKit. The sequence is default800x450, wide1008x450, small1x1 and default800x450 again. Each size records before fit, after first fit and after second fit. Reset and Unicode-graphemes activation use the real terminal in both engines.

For example, Windows Chromium's measured character width is8.2470703125px. At two columns the rounded CSS canvas width is16px, so the CSS cell width is8px. Returning to an800px parent first proposes98 columns. After that resize, the CSS canvas is808px and cell width is808/98px, so the second proposal is95 columns. Original JavaScript and Rust both perform this98-to95 change. Fixing only Rust to force stability would change upstream behavior. Firefox's corresponding original assertion is95 versus92. WebKit's default font passes the original bounds and repeated-fit assertions.

No production code changed for this diagnosis. Prior Lane utility patches already match the integrated implementation, so they were not replayed.

## Reproduce

Use the same prerequisites as `scripts/e2e-upstream.mjs`: built original demo bundle, installed original Playwright engines/dependencies, Cargo's WASM target and wasm-bindgen0.2.100. Build the Rust distribution first:

```sh
node scripts/build-demo.mjs --debug
node scripts/fit-browser-oracle.mjs
node scripts/fit-browser-oracle.mjs --suite
node scripts/e2e-upstream.mjs --suite addon-fit --no-build
```

`SOURCE_ROOT` may name an existing original checkout with its generated `demo/dist/client-bundle.js` and installed dependencies. `PLAYWRIGHT_MODULE` can name its existing Playwright module. During this run both pointed to the supplied original checkout, without modifying its sources. The oracle checks all776 manifest entries before and after execution and records the exact original generated-bundle hash. It uses that existing bundle and does not claim to have rebuilt it.

The plain oracle command succeeds only when every source/Rust geometry pair agrees. `--suite` additionally runs the unchanged original Fit test file against JavaScript, with only the server address adapted. It preserves the original projects, fonts, test bodies, assertions, retries and timeout. It returns a failure exit code for the10 observed upstream failures; they are never converted into passes. `e2e-upstream` does the same against Rust and also fails here.

Both servers and browser processes have bounded lifetimes and close in `finally`. The focused oracle removes its own `tmp/fit-browser` files. Reports are under `results/fit-browser/`. Do not run two instances of this focused oracle in the same checkout because they share those named report/scratch paths.

## Evidence

- `results/fit-browser/oracle.json` records Node/browser versions, Rust build identity, source hashes, integrity checks, all source/Rust snapshots and original-suite results.
- `results/fit-browser/source-suite/` retains the original Playwright JSON/log and exact counts.
- `results/fit-browser/rust-suite/` retains the unchanged-suite Rust Playwright JSON/log, summary and fresh build receipt.
- `results/fit-browser/suite-comparison.json` compares each project/test outcome and failure message, rather than only aggregate counts.

The Rust distribution in these receipts was built from main3ee576a. These observations are not a final integrated browser matrix, a native Linux/macOS result or proof of complete Fit/addon equivalence. Keep the failing original assertions visible in the final verification report. No font override or relaxed assertion is proposed.
