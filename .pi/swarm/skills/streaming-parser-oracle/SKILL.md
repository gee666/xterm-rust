---
name: streaming-parser-oracle
description: Use when executing unchanged OSC, APC or DCS upstream assertions and regenerating deterministic streaming parser traces for Rust replay.
---

# Streaming parser oracle

Run `node scripts/check-osc.mjs` and `cargo test -p xterm-core --test osc --test osc_edges --test osc_failures --test osc_registry --test osc_differential`. For real outer-parser integration also run `node scripts/escape-osc-oracle.mjs` and `cargo test -p xterm-core --test escape_osc`. Generate updated fixture bytes only with `node scripts/check-osc.mjs --generate`. Node24 native TypeScript transformation is required. `scripts/parser-oracle-common.mjs` loads the original constants, decoders, builder and chosen parser into one VM realm, removing only import/export/type syntax. Original assertions and method bodies remain unchanged.

The common helper exports `parserOracle(name)` for Osc, Apc and Dcs. Dcs also loads Params. It returns context.Parser, baseline case outcomes, source hashes, utf32String conversion and resolvedPromise(callback). Use resolvedPromise for async test handlers; a host-realm promise fails the original parser's instanceof Promise check and would test the wrong path.

OSC uses seed0x05c2019 for512 traces of96 operations. The outer parser oracle adds6726 traces at seed0x05c91, with every sequence split boundary, mixed callback futures and reset. It flushes background cleanup at explicit matching JS/Rust executor turns. Preserve the retained restart cleanup counterexample. Use HandlerResult::resolve for infallible/fallible futures and stop processing on Err instead of resuming false. `results/osc/oracle.json` records all23 original outcomes and source/fixture hashes. Rust replays `tests/fixtures/osc-oracle.tsv`; failures retain exact case and operations in assertion output. Do not hand-edit expected traces. See docs/osc.md for contracts, assertion mapping and integration limits.
