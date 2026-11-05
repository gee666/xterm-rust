# Independent omitted-column review

Task 437 was accepted at frozen `4576a356`. The reviewer found no problems.

The reviewer checked that the browser adapter's existing empty-loop sentinel now covers omitted and explicit undefined start columns. It preserves captured end columns, later rows and the shared serializer.

Independent checks:

- Recompared source/Rust traces in Chromium, Firefox and WebKit. Both new resize probes fail in each baseline and match in each repaired run, including exact HTML, getter order, controls, thrown-value identity and cell state.
- Checked unchanged-suite reports. Source 120/120 and Rust 120/120 pass, with no skipped or unrun cases.
- Verified current source, package and runner hashes, reconstructed the baseline source hash, and checked all 776 reference files.
- Directly reran 14 existing native binaries, with 62 tests passing. The reviewer did not rerun the two file-writing differential tests; their retained producer results pass.
- Performed no compilation or new browser runs and modified no files.

The producer ran all 15 native binaries, with 64 functions passing. These counts describe different executions and are not interchangeable. The known ANSI raw-numeric mismatch, workspace formatting OS206 and full family 346/322 work remain open. Full equivalence is NOT PROVEN.

Canonical pickup after accepted 427 uses only `532551a2`, `e7a7be6a`, `4576a356` and the subsequent review/cleanup receipt. The private prerequisite merge `287cc0be` must not be replayed into canonical main.
