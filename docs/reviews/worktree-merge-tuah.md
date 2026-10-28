# Worktree merge review

Reviewed the canonical `worktree-lib.mjs` and actual create/merge CLIs.

Ran `node tmp/tuah-worktree-merge-review.mjs`: seven real Git/Cargo scenarios pass. Successful create→commit→merge fast-forwards and deletes only the private target. Dirty main, untracked required source, failing tests, source generated during a gate, a broken unchanged consumer and a rebase conflict all refuse promotion without changing main. The source reference survives.

Confirmed `--no-renames` selects both old/new crate paths; `changedPackages` returns both crates. Earlier generated-source omission was demonstrated and fixed.

Creator/carry review is task245. Its byte-fidelity findings must also be fixed before the whole automation243 is accepted. This is merge correctness evidence, not Rust terminal parity.
