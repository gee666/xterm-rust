# QOI source fixtures

`palette.rgba` is copied unchanged from the supplied addon-image `fixture/palette.blob`. `palette.qoi` is encoded with the installed xterm-wasm-parts0.4.1 QoiEncoder using the original ImageAddon.test.ts dimensions640x80. The native decoder must reproduce every original RGBA byte.

The package MIT notice is retained in `licenses/xterm-wasm-parts-MIT.txt`. `qoi-source.json` in the parent directory is generated source observations, not a native implementation table. Regenerate only with `node scripts/qoi-oracle.mjs --generate`; normal verification rejects source fixture drift.
