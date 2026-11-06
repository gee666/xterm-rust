# E2E WASM fetch transport

Task548 changes the existing Rust browser test transport, not the terminal engine or original assertions. Full327/application equivalence remains NOT PROVEN.

## Retained failure and unchanged input

The first full0442 Rust matrix used the accepted531 debug WASM. Its core attempt hit six unchanged Firefox10s beforeAll deadlines and left413 cases unrun. Image and Fit added34 and8 unrun cases. The fresh original JS core completed1194 passes and240 original skips with no failure or unrun case under the same two-worker configuration. These are retained initialization failures, not an unavailable-platform blocker.

The old generated bootstrap was21701379 bytes. It embedded16062126 WASM bytes as base64, then invoked a JavaScript callback for every decoded byte through Uint8Array.from. The embedded bytes were independently decoded and hashed. They match the verified package hash `0493b554f5782083fac68f383235eb40df84d6f69252666ce699afdb3790a6f6`. `results/reese327/transport-old-embedded.json` records that check.

This identifies removable transport work. It is not a release parser benchmark or a claimed application speedup.

## Implementation

The bootstrap requests `/pkg/xterm_web_bg.wasm` from the existing owned fixture server. It rejects an unsuccessful HTTP response, then passes the actual Response to the existing asynchronous wasm-bindgen initializer. The server already serves that verified package file as application/wasm. No Rust code, package bytes, original assertion, hook timeout, worker count or retry policy changes.

The readiness image and token remain unchanged. Constructors are published only after real WASM initialization. Network, HTTP and compilation failures record the initialization error and notify the failed navigation. The original tests cannot receive an early successful readiness signal. The missing-file guard still rejects a missing WASM package before emitting a bootstrap, without reading the whole binary into Node just to encode it.

## Controls

The new ordering and failure tests first ran against the unchanged old production bootstrap and failed0/2, with the missing initial GET and wrong initializer input recorded. After the production change, all36 E2E/acceptance controls passed, including missing-WASM rejection, held compilation, constructor publication ordering, missing-addon reporting, network rejection, HTTP404 and invalid-WASM failure notification. Existing fixture server/readiness/cancellation/counter/provenance checks remain in the run. Test-only export stubs check transport order, not terminal behavior.

Logs are `results/reese327/transport-unit-old.log`, `transport-unit-fixed.log` and `transport-unit-final2.log`. The old actual browser matrix and every original-source result remain intact.

## Actual browser verification

A separately granted browser-only phase executed complete, unfiltered core, image and Fit configs against the same package. All three original projects ran with two workers, unchanged10s hooks and zero retries. No Rust rebuild was needed.

| Config | Discovered | Passed | Failed | Original skips | Unrun |
| --- | ---: | ---: | ---: | ---: | ---: |
| core |1434|1167|27|240|0|
| image |639|628|7|4|0|
| Fit |27|17|10|0|0|

There were no beforeAll startup failures or fixture server errors. Each report records successful GETs for the actual WASM path and ready constructor publication. The27 core failures are the absent registerDecoration API on this snapshot. Image private-accessor gaps and source-shared geometry failures remain; Fit's ten failures also occur in the fresh original baseline. Every command still returned nonzero for its application failures. Successful transport does not turn those failures into a pass.

`results/reese327/transport-browser-progress.json` records finite commands and cleanup. `verify-transport.mjs` independently checks raw/stored counts, zero retries, zero setup fallout, successful WASM requests, current1866-input identity and all75 unchanged package files. It writes `transport-proof.json`. Both core/image/Fit first-attempt failures and complete reruns are retained. The first full14-config checkpoint is never overwritten.

The derived latest complete-config accounting combines those three full reruns with the eleven other first-run configs on identical Rust inputs:4116 discovered,3825 attempted,3701 passed,124 failed,291 original skips and zero unrun. This is not one fresh14-config run under the new transport. `compare-baseline.mjs --updated` records3697 source/Rust passes,56 source-passed/Rust-failed,68 failures in both,291 skips in both and four source-failed/Rust-passed. These are named case outcomes, not full output equivalence.

All resources from this phase were released after the owned supervisor and servers exited. Task327/330, application repair and final stable integration proof remain open.
