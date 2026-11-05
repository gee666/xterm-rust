---
name: control-margins-check
description: Use when checking BS and reverse-index origin-margin restrictions against the unchanged source terminal.
---

Run `node scripts/control-margins-oracle.mjs` then `cargo test -p xterm-core --test control_margins -j2`. Use `--generate` only to record deliberate source fixture updates. `XTERM_SOURCE_DEPENDENCIES` may point at an existing source tree with esbuild installed; original source inputs still come from this checkout's `source_repo`. The runner verifies all 776 source-manifest files before and after execution and removes its owned bundle in `finally`.

The 288 cases cover both buffers, origin and reverse-wrap modes, wrapped rows and cursor boundary inputs before BS or ESC M. They reuse complete terminal state and ordered event projections. Direct cursor mutations model InputHandler boundary contracts, not proof that every state is publicly reachable. Full family, browser and native OS requirements remain open. Use an owned O: worktree and resource coordination, not retired build-slot automation.
