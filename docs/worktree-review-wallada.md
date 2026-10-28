# Pending create-script review

Task245 reviews creation, target isolation, disk checks and cleanup. Tuah244 reviews merge/rebase and dependent builds. Kassa243 is the only implementation task.

Initial source read found two points to check when the files settle:
- worktree-create imports parseOptions/repoInfo/assertSpace while the common file at that instant exported different helper names. This may be an intermediate write, not a final defect.
- recursive --copy-owned skips .git/target/tmp/node_modules, but must also reject nested .env and private .pi paths, not only a top-level owned path.

No create-script tests have run yet. Use tiny temporary Git repos to test two agents get separate branches/targets, preserve the shared registry/source, reject unsafe copy paths and delete only the merged worktree target. Keep evidence to commands and results.
