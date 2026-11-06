# Demo web-links reload probe

Task 566 narrows one failure in task 327. Full browser/addon/application equivalence remains NOT PROVEN.

## Retained failure

Archive `archive/Reese541548-complete` at `43ffa9b1d88c6d96b794aea3c5883079b22aa238` retains the complete failed demo06 report and traces under `results/demos/browser/2026-11-06T00-48-58-448Z-58f99517-d4e8-4d41-9b31-b258a7c3de9c/`.

The Chromium trace shows the first web-links hover and popup completed. The addon was then reloaded, and the second probe returned to exactly the same buffer cell. It failed waiting for `.xterm-cursor-pointer`. Firefox has the same retained second-probe failure. This is not evidence that URL discovery or every live hover is broken.

The unchanged `source_repo/src/browser/Linkifier.ts` stores `_lastBufferCell` independently of the current link and provider registration. Moving outside the terminal clears the current link, not that cell. Returning to the same cell does not call `_handleHover`. Replacing the provider does not erase this cache. The directed method control in `results/link-demo-reload/method-control.json` observes zero hover calls for that same-cell return and two calls after visiting a different row and returning.

## Probe change

Only the custom demo probe changes. Before entering the URL's first row, it visits the next row with actual Playwright mouse input. This makes the cell transition explicit and requests fresh provider results after reload. Both original hover assertions and both real popup URL/close checks remain. The Rust linkifier and all supplied reference files/assertions are unchanged. Changing production cache behavior to satisfy a custom probe would alter source semantics.

The unit regression loads the actual probe module with a controlled pointer/cell dependency. It does not execute the terminal engine and is not browser equivalence evidence. It checks cold and already-cached first-row states, two probe cycles, both pointer assertions, and popup listener/click/URL/close ordering. Before the change, both controls failed. After the change, both pass. See `unit-old.log` and `unit-fixed.log`.

## Commands

From an owned worktree with the exact reference available:

```sh
node --test scripts/demo-link-probe.test.mjs
SOURCE_ROOT=/path/to/exact/source_repo node scripts/link-demo-reload-source.mjs
```

The source browser script uses real Chromium, Firefox and WebKit pointer input against unchanged source bundles. It checks the bundle sourcemaps against the source, first hover and actual popup, reload through an outside button, a same-cell negative control, then different-row return with second hover and actual popup. Each browser closes in `finally`; the artifact server closes its own connections. Use an owned O: `TEMP`, `TMP` and `TMPDIR` and a granted browser allocation.

The actual source browser run and matching current Rust demo rerun are pending. Do not infer their results from the unit control or historical trace.

## Package staging

Authorization `714#4985` allowed a read-only copy from the integration554 owner's debug package, with no donor hold. `results/link-demo-reload/package-copy.json` records all 84 package files and 97 copied files including report/pins/demo assets. Every copy was hashed and reread against the donor. Its Rust source identity is `19ffb731fe479eb0630bf8b01c74464c7be91ae039ab4641ab11e56d18b7cf95`, 1965 inputs, from frozen `24be32786b32729def781ff130ec0700900b0bcc`.

That package is staged only under the owned temporary root. It must not run as a current build before the actual Rust inputs match accepted554/current main and `verifyBuiltPackage` passes. No compiler or browser operation was authorized by the copy. The package is not a release or benchmark artifact.
