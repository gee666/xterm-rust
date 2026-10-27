---
name: compatibility-inventory
description: Use when checking complete xterm source/API/addon/suite inventories, explicit Rust mappings, focused acceptance gaps or stale evidence snapshots.
---

# Compatibility inventory

Run from the repository root with Node22+ and npm:

```sh
npm ci --prefix tools/compatibility --ignore-scripts --no-audit --no-fund
node scripts/compatibility-inventory.mjs
node scripts/compatibility-inventory.mjs --check
node --test scripts/compatibility-inventory.test.mjs
```

The exact TypeScript5.9.3 parser is tooling-only. The generator does not execute terminal logic or mutate source_repo. Its reference exclusions and schema are documented in docs/compatibility.md; acceptance is docs/acceptance/compatibility.md.

Edit tools/compatibility/mappings.json for reviewed implementation/test/evidence/binding/gap references. Add exact focused criterion/checklist rules to tools/compatibility/behavior-criteria.json. Do not guess coverage from filenames or task titles. Unmapped rows must remain explicit gaps; no implemented helper establishes a public/browser binding.

Read results/compatibility/summary.json for counts, modules.json/apis.json/suites.json for exact source contracts and gaps, manifest.json for source hashes and evidence-manifest.json for mapped snapshot hashes. Original counters are copied only from explicit upstream-baseline per-source evidence. Rust unassessed counters stay null.

All shell commands require a timeout. Temporary negative-test output stays under tmp/compatibility-validation and is removed after success. Concurrent owner writes correctly stale evidence snapshots. Regenerate only after reviewing the changed content, never suppress the stale check. Retain command output under a separate results/compatibility-check/ folder so the generator's own output schema stays stable. Independent review must run commands and inspect real missing rows, not merely read summary counts.
