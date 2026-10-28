# Independent canonical-asset and build-boundary review

Task 204 changes line-inventory classification, not terminal behavior. I read `scripts/count-loc.mjs`, the complete rules file, all 18 filesystem tests and the relevant explanation in `docs/loc.md`.

A generated CSS distribution copy is still inventoried with its path, bytes and SHA-256. It is excluded from duplicate source-line totals only if its declared canonical source is counted and their complete byte hashes match. Missing or divergent canonical data fails the scan. Mandatory generated JavaScript bindings remain included separately, so this exception cannot silently omit ABI glue. Classification and purpose/category/extension totals remain reconcilable and the report is explicitly a partial working-tree source snapshot, not an equivalent-functionality reduction claim.

The new port-root target exclusion is checked before symlink refusal, preventing traversal into the approved external Cargo cache. It does not weaken selected-root, ordinary source-entry or reference-tree symlink guards. The tests exercise actual Windows junctions, exact reserved root handling, missing/divergent copies and negative controls. This scanner explicitly assumes stable inputs and does not claim to defeat a hostile path-swap race.

I independently ran `node --test scripts/count-loc.test.mjs`: 18 tests passed, zero skipped or ignored. The output is `artaxerxes-204-tests.tap`. I then ran `node scripts/count-loc.mjs --check-baseline --output results/loc/reviews/artaxerxes-204-snapshot`. It succeeded on the actual current target junction without traversing that build tree. The reference legacy subtotal exactly matched 396 files, 96,130 physical lines, 87,026 nonblank lines and 3,962,777 bytes. The complete raw inventories, hashes and partial source report are retained under that unique output directory.

No compiler, bundler, Cargo build, source mutation or cache relocation ran for this review. Accept task 204's scoped inventory changes. Final integration recount task 98 and actual speed/distribution-size comparisons remain separate; no full equivalence or code-reduction claim is inferred.
