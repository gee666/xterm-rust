---
name: worktree
description: Use when creating an agent branch with an independent Cargo target, migrating owned changes, testing in parallel, and merging tested code back.
---

# Parallel worktrees

The old global build queue is retired. Each agent works on `swarm/NAME` in its own worktree and builds only the crates it needs. Cargo's downloaded registry stays shared automatically. Development/test profiles remain small.

From the original repository, with Git and Cargo in PATH:

```sh
node scripts/worktree-create.mjs --agent NAME --main master
```

The command prints the C-volume worktree path, branch and private target. Use that directory for editing and commands. You can invoke the scripts by their absolute path in the original repository until the tooling commit reaches your branch.

For existing shared-tree work, copy only the paths your task owns into the new worktree and commit them there. Do not stash/reset other agents' files. Shared-file hunks need a coherent owner commit first. The create helper accepts repeated `--copy-owned PATH` flags and preflights all matched files. For example, `node scripts/worktree-create.mjs --agent NAME --copy-owned crates/YOUR_CRATE/src/YOUR_FILE.rs`. It preserves exact file bytes, rejects private/generated/rules paths, and never removes the primary copy. Keep the original copy until the branch commit is verified, then restore/remove only your own identical shared-tree paths.

Run real tests in parallel, for example:

```sh
cargo test -p xterm-core --test YOUR_TEST
```

Before integration, commit all source files in the agent worktree, then run from that worktree:

```sh
node /absolute/path/to/original/repository/scripts/worktree-merge.mjs
```

The merge script rebases on main, builds/tests changed crates, checks affected dependents and applicable WASM code, then fast-forwards main only if the tested branch/base did not change. Conflicts stay with the owner. Dirty tracked changes in main are refused rather than stashed. The private target is deleted after a successful merge.

The current committed main baseline has known missing producer/helper commits. A clean worktree exposes them correctly; do not copy another agent's WIP to claim a green build. Owners are committing a coherent bootstrap. Rebase when it lands.

The supplied `source_repo` is for upstream tests only. Never modify its original inputs. The creator makes a physical reference copy using the existing baseline file list, without node_modules or generated outputs. Install source-only Node dependencies in that copy when the original suites need them; Rust-only work does not require this setup. Existing Windows reference junctions must be unlinked before any whole-worktree removal: raw `git worktree remove` can traverse a junction and delete the reference. Do not remove a whole worktree with such a link. The merge script deletes only its regular private target, never the worktree or reference.

Evidence is a short command and result. Review the change and run its tests. Full native OS CI uses GitHub Actions; missing Linux/macOS access is not a blocker.
