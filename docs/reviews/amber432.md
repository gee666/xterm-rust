# Independent review of raw Scrollable getters

Task432 accepted after resumed isolated review of sourcecef749ae plus7d302606. The first source/light review found no in-scope implementation defect but requested actual Rust browser differential evidence. The final review accepted the observed matrix and the final-tick repair. No reviewer source modifications were made.

The reviewer independently reran existing native executables,76 tests passed. Changed-source formatting and live source-oracle checks passed. All13 current implementation/fixture/lockfile hashes matched environment.json. Compiler WASM, generated WASM, generated bindings, raw browser logs and final generated oracle snippet matched their reports. No new compiler/browser jobs were started during either review stage.

Reviewed phases include selected-state capture before getters, repeated reads and inherited/proxy receivers, per-axis smooth fallback after reentry, direct versus expanded undefined handling, and tick plain-object fallback. The initial three-engine failure remains retained. Cached-first-read negative control failed exactly as expected and the restored final implementation passed5804 getter plus605 retained live programs per engine in Chromium, Firefox and WebKit, two Rust functions/engine with no ignored or filtered cases.

Final focused native/WASM Clippy receipts passed. Changed and five-member format receipts passed. Exact aggregate cargo fmt still reports WindowsOS206. These are scoped Windows/component receipts, not a complete workspace or three-native-OS pass. Full301 Element, state-object, scheduler, clock and event-exception integration remains open. Full equivalence is NOT PROVEN.

Evidence: results/raw-scrollable-getters/{environment.json,browser-initial.json,browser-fixed.json,browser-negative.json,browser-final.json,negative-command.json,final-commands.json,source-format.json}. Review acceptance is also recorded in task432 timeline.
