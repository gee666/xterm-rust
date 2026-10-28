# Original erase fixtures

These `.in` and `.text` files are copied byte-for-byte from `source_repo/test/fixtures/escape_sequence_files/`. The xterm.js authors distribute them under the MIT license included in the repository's `LICENSE`. Keep their original data and expected assertions intact. The local `.gitattributes` disables newline conversion because terminal input bytes are observable behavior.

`source_repo/src/browser/Terminal2.test.ts` uses an 80-column, 25-row PTY. That original suite skips Windows and explicitly skips `t0055-EL` because xterm's pending-wrap behavior differs from xterm.js. The Rust native replay models PTY ONLCR explicitly and compares the four applicable legacy text expectations. `scripts/csi-erase-fixtures.mjs` separately compares the real unchanged JS engine for all five, including EL, and records source hashes and any legacy mismatch.

An explicit ONLCR replay is not an observed native PTY or browser test. Actual platform/browser checks remain separate acceptance gates.
