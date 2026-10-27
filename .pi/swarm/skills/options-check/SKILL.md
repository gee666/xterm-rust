---
name: options-check
description: Use when validating OptionsService defaults, constructor diagnostics, JS coercion, event ordering and disposal against the original TypeScript source.
---

Run `cargo test -p xterm-core --test options_service --test options_boundary` for faithful upstream and Rust boundary assertions.

Run `node scripts/check-options.mjs` with Node 24 or later and cargo/rustc on PATH. The script strips types and changes import/test-runner plumbing only. It executes the unchanged original ten cases, compares all 48 defaults, constructor diagnostics, seeded setter traces, custom object coercion hints/failures, long radix inputs, and a reentrant/disposal scenario against the Rust options_oracle executable. Generated build/summary evidence goes to results/options/. Failing operations are retained in counterexample.json. A retained counterexample can describe a fixed historical bug; inspect docs/options-verification.md for resolution status.

The core constructor takes an ordered iterator. A JS binding must enumerate inherited enumerable properties with for-in order. Prototype mutation and inherited Object.prototype keys require separate binding evidence; this check does not prove those browser contracts or full terminal equivalence.

Keep the shared target directory and repository build profiles. Check disk before large builds. No detached services are started.
