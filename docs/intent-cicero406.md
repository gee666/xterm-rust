# Font callback buffer mutation

Continue the existing full xterm.js 6 Rust port on actual main, preserving all source behavior and assertions. Task406 fixes retained ArrayBuffer writes during Mac kern warning callbacks. Use O: owned scratch, no JS font parser delegation, differential/native/browser checks and canonical reviewed integration. Task342 broader metadata and overall equivalence remain open. Owned temporary root is O:/xterm-rust-swarm-16/tmp/C406; no owned processes yet.
