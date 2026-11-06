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

## Actual browser results

Under grant `409#5033`, the corrected directed source run passed in Chromium 148.0.7778.96, Firefox 150.0.2 and WebKit 26.4. Each observed the same-cell negative, both real hovers and both real popups. `source.json` records the actual cache/current-link state at every stage.

The first source attempt passed Chromium, then failed Firefox navigation before creating a terminal under this new helper's 10-second setup bound. WebKit was unrun. `source-initial.json` and `source-initial.log` retain that attempt. The helper now uses the existing directed source-browser scripts' 30-second setup default. No upstream hook, demo case deadline or assertion changed. The complete corrected source attempt passed all three engines.

Current-main Rust inputs matched the borrowed integration554 package exactly. The same three custom demo cases ran twice through the actual Rust/WASM demo, with the original two workers, 30-second case timeout, zero retries and unchanged assertions:

| Probe | Discovered | Executed | Passed | Failed | Skipped | Unrun |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| Old pointer path | 3 | 3 | 0 | 3 | 0 | 0 |
| Different-row path | 3 | 3 | 3 | 0 | 0 | 0 |

All old failures reached the intended pointer assertion on the second probe. All three fixed cases completed both hover and popup checks, reload, unload, post-unload output and input. `rust-summary.json`, `rust-old/` and `rust-fixed/` preserve commands, reports, errors, screenshots, trace ZIPs and attachments. The temporary old probe was restored byte-for-byte in `finally`; the server closed and source identity remained unchanged. The exact scoped driver/config are retained as evidence, not portable application code.

The initial discovery-only command used an anchored grep that matched no test titles. Its exit 1 and zero discovered cases remain in `rust-list-initial.*`. The corrected unanchored grep discovered three cases before runtime. This was a discovery error, not an application pass or a skipped runtime case.

The original and owned physical reference copies both passed all 776 manifest hashes after execution. No compiler ran in this lane. Native OS matrices, full demo06, full core/addon suites and full equivalence remain open under task327 and the other family tasks.

## Package staging

Authorization `714#4985` allowed a read-only copy from the integration554 owner's debug package, with no donor hold. `results/link-demo-reload/package-copy.json` records all 84 package files and 97 copied files including report/pins/demo assets. Every copy was hashed and reread against the donor. Its Rust source identity is `19ffb731fe479eb0630bf8b01c74464c7be91ae039ab4641ab11e56d18b7cf95`, 1965 inputs, from frozen `24be32786b32729def781ff130ec0700900b0bcc`.

After main advanced to `14139449bae2792afb019f42e8438b074938cfe7`, this branch rebased cleanly and its actual 1965 Rust inputs matched that digest. All 84 package files and every build-report artifact were verified, then `verifyBuiltPackage` passed in the owned root before runtime. The local build pointer was installed only after preserving its original bytes. After execution, the used pointer was snapshotted durably and the exact original bytes restored. The restored latest pointer is not evidence for this run. See `current-input.json`, `package-installed.json` and `pointer-restoration.json`.

The first install copied the package but stopped at missing generated `browser/xterm.css` before changing the pointer. `package-install-initial.json` records the error, independent verification of all 84 copied files and the untouched pointer. The already hash-verified staged CSS was then copied, all build artifacts rechecked and installation completed. No failed build or browser attempt was hidden.

The copy authorization alone did not authorize runtime. Browser execution used the separate explicit grant5033, released at `409#5049`. The package is not a release or benchmark artifact.
