---
name: font-cff-fdselect-check
description: Use when verifying eager CFF2 FDSelect formats0/3/4, maxp defaults, constructor error ordering, raw dictionary addresses and actual addon recovery.
---

# FDSelect checks

Read docs/font-cff-fdselect.md and task559 pickup before replay. Reuse scripts/font-cff-fdselect-oracle.mjs, font-cff-fdselect-browser-source.mjs and browser-host-check.mjs with WASM_PACKAGE=xterm-font-addons and WASM_TEST=font_cff_fdselect_browser. The oracle checks immutable fixtures by default. Generate only after inspecting source drift. Source hash737212d8aae53d1996ef10b315e4369a1542ebb9b94c13c5f99793914745d484 is mandatory. Each source child is bounded128MiB/3000ms and any child failure fails the run.

Use a granted heavy slot, private O:target/TEMP/TMP, Cargo-j2, existing Playwright and matching wasm-bindgen0.2.100. Native tests retain the full predecessor chain. Actual browser tests independently reconstruct inputs and exercise addon rejection caching/fallback/recovery/read/refresh/disposal. Keep Bounds/Type wording limitations explicit. Charstrings/CFF1/full342/equivalence remain NOT PROVEN. Retain failures/compiler pins and reference hashes, close all owned processes, inspect complete tree for links before cleaning only owned temporary outputs.
