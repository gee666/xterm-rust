# Compatibility inventory acceptance

These checks govern the index, not implementation equivalence. A missing implementation must remain visible even when the index passes. `docs/compatibility.md` must state NOT PROVEN while required behavior or evidence is absent.

## Reference completeness

1. Walk the supplied reference without following symlinks. Reject symlinks rather than silently omitting their targets. Sort paths and use SHA-256 of exact file bytes.
2. Include every production TypeScript, declaration, and CSS file under common, headless, browser, addon src/typings, root typings, and CSS. A new file must make `--check` fail until reviewed and regenerated.
3. Include all fourteen addon directories, including addons with no Rust implementation. Discover the list from disk, not a hardcoded successful subset.
4. Include every source test, benchmark, shared test-registration helper, runner configuration, and escape-sequence fixture. Test support cannot disappear merely because it lacks a `.test.ts` suffix.
5. Exclude only documented dependency, VCS, and generated-output directories. Building JavaScript must not increase the source inventory. A file hash does not establish public upstream provenance.
6. Give every production row an implementation list, Rust-test list, executable command, evidence list, binding description, status, and concrete remaining gap. Empty lists must mean explicitly unmapped, never success.

## Public contracts

1. Parse every `.d.ts` file with the pinned TypeScript AST parser. Reject syntax diagnostics instead of falling back to regular expressions.
2. Retain each exported type/interface/class/variable and every member, overload, constructor, callable signature, optional marker, readonly/static modifier, and exact signature. Signatures include callback/object/union contracts even when they are nested.
3. Identify option members and events separately. Preserve source documentation for defaults, callback ordering, numeric ranges, constructor-only restrictions, disposal, and return/error behavior. Distinguish a documented contract from a tested implementation.
4. Inventory headless and browser declarations independently, even where names coincide. Do not merge two contracts based on equal member names.
5. Record experimental documentation and source runtime proposed-API guards separately. The `allowProposedApi` option's description must not imply that the option itself requires gating.
6. Record source constructor-only `cols`/`rows` behavior. Mutating `Terminal.options` must remain a missing contract until Rust binding tests prove the source's rejection behavior.
7. Explain Rust representations for UTF-16/lone surrogates, undefined/None, exceptions/Result, signed bitwise values, callbacks/futures, identity/sharing, and drop versus explicit disposal wherever relevant.
8. Internal CellData, events, Unicode or lifecycle helpers must not mark public Terminal/browser bindings implemented. Type-name resemblance is not executable evidence.

## Suite and behavior accounting

1. Index each suite and case by source path, line, exact title expression and nested suite names. Retain dynamic title expressions and conditional test registrations rather than inventing expanded counts.
2. Recognize Mocha and Playwright suite/case registrations, skipped definitions, shared injected renderer cases, and benchmark contexts/cases. Runtime `test.skip(condition, reason)` annotations must not count as test registrations.
3. Static definitions, expanded runtime cases, assertions, and Rust test functions are different units. Report them separately. Never derive passed/executed counts from a present fixture or filename.
4. Every suite must link exact implementation/test/evidence references or state a migration/execution gap. Partial BufferLine cell-only migration must not count the remaining BufferLine suite as covered.
5. Each behavior needs focused acceptance IDs and assertion references. An absent focused behavior criterion must be visible as an acceptance-mapping gap, not filled with an umbrella implementation criterion.
6. Track original and Rust discovered/executed/passed/failed/skipped/unrun evidence separately. Unknown values remain null with a reason. Shared helper execution multiplicity is a runtime runner responsibility.
7. Retain fixture bytes and hashes, including currently unused `.in_` inputs and expected text/HTML. No fixture may be described as executed without its Rust runner results.

## Integrity and regeneration

1. `npm ci --prefix tools/compatibility --ignore-scripts` must use the checked-in exact parser lockfile. It must not modify source_repo or depend on globally installed TypeScript.
2. Generation and `--check` must work from a non-root current directory. Paths in output remain repository-relative with `/` separators.
3. Check every supplied source/mapping path and evidence file exists. Reject duplicate module mappings, duplicate API IDs and malformed status rows.
4. `--check` must compare generated content, not only counts. Changing a source signature, test title, fixture byte, mapping gap or evidence reference must invalidate the result.
5. The independent checker must cross-check disk path sets, exact declaration anchors, fixture hashes, addon count, experimental guards, conditional renderer cases and missing-status honesty.
6. Include new-module, removed-module, changed-signature and absent-evidence failures in automated validation or document the exact checker condition that rejects each.
7. Keep source hashes separate from non-reproducible execution environment data. Inventory generation must not claim it ran Rust or original suites.
8. Independent review must inspect generated gaps as well as implemented mappings and run the inventory/checker commands. Index acceptance must not be described as full-port acceptance.
