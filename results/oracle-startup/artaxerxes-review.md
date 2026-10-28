# Independent oracle startup review

I reviewed task 159's source ordering in `scripts/serialize-oracle.mjs` and `scripts/charset-terminal-oracle.mjs`, and the permanent `scripts/check-oracle-startup.mjs`. The unrelated reflow helper algorithm is not part of this acceptance.

The affected oracles create their results directory and invalidate the previous receipt before creating the temporary directory or resolving esbuild. They retain an uncaught setup failure as a failed receipt and still exit nonzero. The permanent test verifies exactly one root assignment before substituting an isolated root, redirects repository helper imports without changing behavior, forces a broken local esbuild package to prevent a global fallback, and requires both a real nonzero exit and a newly invalidated receipt. Timeouts and crashes are not counted as success. Temporary roots remain under repository-root tmp/; source and shared oracle results are not moved or deleted.

Under the resource pause's allowance for tiny non-build probes, I independently ran `node scripts/check-oracle-startup.mjs`. All six probes passed: missing reference/dependency and blocked temporary directory for reflow, serialize and charset. Each stopped before any bundler/compiler service could start. Exact source hashes, error outputs, receipts and Node/platform metadata are retained in `results/oracle-startup/artaxerxes-all.json`. Node v24.15.0 ran on Windows x64. Disk remained about 878 MiB free; no Cargo command or normal oracle generation ran for this review.

No source assertion was changed, no error was suppressed into a pass, and the tests correctly retain incomplete/failed status rather than an earlier passing sentinel. Fresh successful serializer/charset generation and their Rust/native/browser validation remain separate checks, currently paused for disk capacity. These six negative startup probes do not establish terminal equivalence or complete coverage of every possible filesystem or module-load failure.

Accept task 159's scoped startup-ordering fix and reproducible regression checks. Overall project equivalence remains NOT PROVEN.
