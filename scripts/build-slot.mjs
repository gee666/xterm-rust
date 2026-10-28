#!/usr/bin/env node
console.error('The global build queue is retired. Use scripts/worktree-create.mjs and .pi/swarm/skills/worktree/SKILL.md; build and test in your own worktree.');
process.exitCode = 1;
