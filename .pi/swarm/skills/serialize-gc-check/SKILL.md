---
name: serialize-gc-check
description: Use when checking real SerializeAddon host retention and unreachable Terminal/addon cycles with forced Chromium collection.
---

# Serializer GC check

Build the actual Rust browser package with `node scripts/build-demo.mjs --debug`, using the locked wasm-bindgen CLI0.2.100. Set SOURCE_ROOT to the unchanged supplied source with Playwright and its original demo bundle. From an owned O: worktree and an allocated browser slot, run:

```sh
node scripts/serialize-browser-check.mjs --gc
```

The existing serializer servers and source/build/artifact checks are reused. The GC helper performs exactly two CDP full collections between separate page jobs at each of four phases per engine. It asserts retained host usability, disposal retaining the host, collection after the addon is released and collection of an unreachable loaded manager cycle. No sleeps or retries until success.

Results are under `results/serialize-gc/`. Firefox and WebKit are explicitly unrun because this runner has no forced-GC protocol for them. This command does not execute the ordinary serializer matrix or original suites and does not establish full equivalence. `--suite` remains the separate full serializer command. All generated servers/pages/sessions and root tmp/serialize-gc are cleaned in finally paths. Use a bounded outer timeout and preserve the report before cleaning the owned worktree.
