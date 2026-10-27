---
name: verification-index
description: Use when auditing retained verification evidence, source reference integrity or final equivalence claims.
---

Run from the repository root with Node24 or later:

```sh
node --test scripts/verification-index.test.mjs
node scripts/verification-index.mjs
```

The first command tests exact byte hashing, deterministic ordering, changed/missing reference files, invalid/duplicate paths, CLI failure reporting and index self-exclusion. The second checks the776-file starting manifest in docs/baseline/source-manifest.json against source_repo and writes results/verification/index.json for all retained result files. Nonzero status means changed/missing reference or an invalid manifest/I/O failure.

Never edit original source/assertions to clear integrity failures. The reference is user-supplied; do not claim a public revision. The starting manifest does not authenticate added files. Use the compatibility inventory for current-tree discovery.

Do not redirect index stdout into results during generation. Regenerate after final edits, on a stable committed snapshot. An indexed log is not a passing test. Read docs/verification-acceptance.md V01-V06 and preserve failed attempts, unknown/unrun counts, precise seeds/commands/platforms and unresolved reviews. Overall equivalence stays NOT PROVEN while any required contract or runtime evidence remains absent. Shell callers must use explicit timeouts and temporary files under tmp/.
