# Independent named control-string data review

Task 216 ports the complete supplied EscapeSequences data module. I read the full original const-enum file, generated Rust data, verifier, all three exact Rust assertion functions, isolated wrapper and status/lease receipts. Public `data::escape_sequences` export is present in the actual core crate.

The module preserves all 67 names and UTF-16 values: 34 C0 declarations including Space/Delete, 32 C1 declarations and escaped ST. The Rust C1 literals correctly use Unicode escapes rather than invalid raw UTF-8 byte assumptions. Escaped ST is ESC/backslash and remains distinct from the single-unit C1 ST. MIT attribution and the generation/source reference remain intact. It is real public protocol data, not a nominal API or JavaScript runtime dependency.

I independently ran `node scripts/escape-sequences-oracle.mjs` and observed all 67 names/values match. I independently executed the retained isolated native test artifact and observed all three tests pass with zero ignored tests. Its wrapper imports the exact repository production and assertion files by path without copying or rewriting either. Native output and source-verifier output are retained in this review directory. The owner's lease receipts separately retain the actual isolated rustc and warning-free clippy-driver executions for the same modules.

The isolated proof is deliberate and bounded: an integrated Cargo attempt was blocked by an unrelated scheduler trait migration. I do not count the isolated namespace alias as proof that the whole current workspace compiles; the real root export was checked in source. Full workspace/feature/native-platform/browser gates remain separate work. No new compiler or Cargo process ran in my review; I executed the retained artifact and the source-only verifier.

Accept the complete data module and its source/name/value checks. No dedicated original EscapeSequences test suite is invented, and no terminal-equivalence or other-platform runtime claim follows from these three native tests.
