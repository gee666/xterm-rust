# Serializer browser collection

Task382 observes actual collection, rather than inferring it from weak Rust ownership. Run `node scripts/serialize-browser-check.mjs --gc` after the normal verified Rust/WASM demo build. The command reuses the source/Rust serializer servers,776-file reference integrity checks, source/build/artifact hashes and finally cleanup. No engine or binding changes are introduced by this task.

The same four assertions run against the unchanged source and actual Rust browser addon in Chromium148:

- A retained addon keeps its terminal alive and serializes its content.
- Disposing that addon still keeps its terminal alive and usable, matching the source empty dispose behavior.
- Releasing the disposed addon permits both terminal and addon wrappers to collect.
- An unreachable loaded terminal/addon-manager cycle permits both wrappers to collect, without requiring explicit disposal.

Each phase uses exactly two CDP `HeapProfiler.collectGarbage` calls. WeakRefs are observed in a separate page job after collection. There are no sleeps or retry-until-success loops. Retained and released phases on the same page provide a reachability control; the cycle assertion uses a fresh loaded pair. Each engine performs8 forced full collections.

The report counts24 discovered assertions:4 phases ×2 engines ×3 browsers. Eight assertions execute and pass in Chromium;16 are explicitly unrun for Firefox/WebKit because this runner has no forced-GC protocol for those engines. A Chromium result is not an all-browser GC pass. Raw results, exact build/runner hashes, runtime routing and cleanup outcomes are in `results/serialize-gc/verification.json`.

This scoped check does not run the ordinary53-case serialization corpus or the original120-case suite; those are separate `--suite` evidence. It does not prove Rust heap deallocation timing, native memory reclamation, every disposed/open DOM lifecycle, or full task346/322 behavior. Overall equivalence remains NOT PROVEN. The reusable procedure is in `.pi/swarm/skills/serialize-gc-check/SKILL.md`.
