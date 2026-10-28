# Image metadata reference fixtures

These38 files are byte-for-byte copies of the user-supplied xterm.js6.0.0 reference under `source_repo/addons/addon-image/fixture/testimages`. They are inputs to the unchanged `IIPMetrics.test.ts` assertions, not newly generated images. The supplied addon MIT notice is retained as `LICENSE.xterm.txt`. No separate image-specific notices were present in this fixture directory; no public Git revision or additional licensing claim is invented here.

`manifest.json` records each original relative path, copied path, raw byte length and SHA256, plus the source test and notice hashes. Original files remain untouched. Rust tests read these copies so a clean native CI checkout does not require the ignored source_repo directory.

Verify copies with `node scripts/vendor-image-metrics.mjs`. Refresh only after reviewing an intentional reference update using `node scripts/vendor-image-metrics.mjs --write`. Refresh requires the reference tree and its TypeScript dependency. Verification alone uses Node builtins and the tracked manifest. The source oracle independently rehashes each original/copy pair before comparing Rust behavior.
