# Charset oracle

`source_repo/src/common/data/Charsets.ts` and `services/CharsetService.ts` remain the reference. Run from the repository root with Node.js 24.15.0 or newer:

```sh
node scripts/charset-oracle.mjs
cargo test -p xterm-core --test charset
```

The Node script writes temporary import-resolved modules under `tmp/charset-oracle/`. It removes interface-only import lines from copies of the two source modules. It does not change implementation statements. The original test body keeps every assertion. Only its imports change to use the temporary source copies and Node's equivalent strict assertions instead of Chai. The harness implements this suite's synchronous `describe`, `beforeEach`, and `it` hooks. It does not claim to run unrelated upstream suites.

`results/charset/oracle.json` records the reference SHA-256 hashes, Node version, platform, seed, fixture counts, and each original assertion's result. The observed original baseline on Windows x64 with Node v24.15.0 passed all four tests. No tests were skipped or left unrun within this suite.

## Fixture contracts

The script generates two TSV files under `crates/xterm-core/tests/fixtures/`. Values representing strings use comma-separated lowercase UTF-16 code-unit hex. `-` represents JavaScript `undefined`, not an empty string.

`charset-maps.tsv` has three columns:

1. Charset designator, encoded as UTF-16 hex.
2. Input code unit in hex.
3. Replacement code units, or `-` when that entry is undefined.

All 16 exported designators and unknown designator `X` are tested against all 128 ASCII input values, giving 2,176 rows. The 15 defined designators share 12 distinct maps. Aliases must retain identity. The Dutch `[` replacement remains two units, `ij`. These fixtures check the stored mapping, not InputHandler's later decision to use only the first replacement unit.

`charset-service.tsv` has six columns:

1. Operation, `s` for designation, `l` for G-level invocation, or `r` for reset.
2. G level in decimal, or `-` for reset.
3. Designated charset in UTF-16 hex, or `-` when absent.
4. Active G level in decimal after the operation.
5. Active map's canonical designator in UTF-16 hex, or `-`.
6. Designated slots, comma-separated canonical designators or `-`. An empty column means an empty array.

Canonical identity uses the lexicographically smallest designator pointing to the same original object. Finnish is `5`, Norwegian/Danish is `6`, and Swedish is `7`. JavaScript array holes and explicit undefined entries both serialize as `-`, while array length stays observable through the slot count.

There are 4,107 operations. Eleven directed operations cover inactive designation, invocation, active replacement, aliasing, sparse designation, undefined designation, and reset. Another 4,096 operations use xorshift32 with seed `0x0c4a25e7`, decimal 206185959. G levels range from 0 through 16. This is service-state evidence only, not terminal integration or a claim of full xterm.js equivalence.

Regenerate fixtures from the script. Do not hand-edit expected values to accommodate a Rust mismatch.
