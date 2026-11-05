# Cleanup read-limit failure

The first generated-output restoration attempt stopped in its read-only verification loop. Node execFileSync's default output limit caused ENOBUFS while reading `git show HEAD:results/serialize/html-differential.tsv`, before the plan report was written and before any git restore or removal statement could run. Git status still showed the same15 generated-file differences and both owned build directories. No compiler, browser, source edit, generated-file restoration or deletion occurred.

The local scratch script was changed only to set a bounded32MiB Git output limit. Before continuing, the missing restoration report, the15 remaining differences and both existing directories are checked explicitly. This is a continuation after a verified read-only failure, not a replay of an uncertain deletion. Application proof and accepted reviews are unchanged.
