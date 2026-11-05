# Preliminary independent reviews

## Task521 at fb7e8f8c

The reviewer found no scoped production defect in shared addition/clamp and ANSI allocation validation. Syntax and whitespace checks passed. The reviewer executed the four new cases against the actual original headless engine, including63 subcases and seven exact thrown-sentinel identities.

The reviewer found a separate actual failure in the reentry success0 control. Reset/shrink invalidated captured endpoints; source then read absent nextLine.isWrapped, while Rust's browser adapter constructed V8-only TypeError text. This was not omitted. Commit f7c109d6 preserved it as a dedicated full-run case and bounded successful allocation reentry to valid endpoints. Task526 then repaired the browser error construction, added two explicit past-buffer ranges and native TypeError prototype/constructor checks. The original preliminary test remains in ec2b22ac.

The review correctly requested compiler, actual WASM, baseline/fixed three-browser, unchanged-suite and trace/pin evidence. Those had not yet run and were not claimed.

## Task526 at bcfcb766

The reviewer found no source defect in the browser-native error bridge. Rust traversal/failure point/native errors were unchanged. Read-only syntax, whitespace and direct Node inline-helper checks passed. Actual original headless replay matched all six retained source-only traces, including all three missing-line controls.

The reviewer requested actual WASM and Firefox/WebKit proof because Node/V8 and static inspection cannot prove those diagnostics. No compiler or browser jobs were run during either preliminary review. Both tasks remain in progress pending runtime evidence and resumed independent reviews.
