# Full core browser checkpoint on main1413

Task574 ran the complete original core Playwright configuration against actual Rust/WASM after accepted integration554. This is a full core configuration checkpoint, not full terminal/all-addon/native equivalence. Full327 remains OPEN.

## Exact run

- Canonical main `14139449bae2792afb019f42e8438b074938cfe7`.
- Owned branch `36113a8a` adds only accepted566 custom demo/proof files; its Rust build inputs equal main1413.
- Rust input SHA256 `19ffb731fe479eb0630bf8b01c74464c7be91ae039ab4641ab11e56d18b7cf95`,1965 files.
- Borrowed actual integration554 debug package: all84 package file hashes and every build-report artifact verified before execution. `verifyBuiltPackage` passed in the owned root. No compiler ran.
- Actual build report `results/demos/build/2026-11-06T02-38-23-405Z/report.json`.
- Grant409#5100 and714#5102. Fresh29.72GiB available RAM and O: headroom recorded before launch. ALLreleased409#5118 after the runner returned and owned server closed.
- Native Windows x64, Node24.15.0. The preceding566 source-browser check on the same Playwright installation observed Chromium148.0.7778.96,Firefox150.0.2,WebKit26.4. The fullcore run used those original named projects; it did not launch another version-only probe.

Command from the prepared owned O: root:

```sh
node scripts/e2e-upstream.mjs --suite core --no-build
```

No project or grep filter was supplied. The unchanged original configuration retained all three engines, two workers, its default10-second tests/hooks and zero retries. Any case-specific original timeout remains intact. The runner's existing default300-second bound produced global timeout299500ms. It was not extended. The raw suite completed in64.72 seconds; the managed command returned exit0 in65.44 seconds.

## Counts

| Project | Discovered | Executed/passed | Failed | Original skipped | Unrun |
| --- | ---: | ---: | ---: | ---: | ---: |
| Chromium |478|419|0|59|0|
| FirefoxStable |478|418|0|60|0|
| WebKit |478|357|0|121|0|
| Total |1434|1194|0|240|0|

Raw evidence is retained under `results/e2e/upstream/2026-11-06T03-34-52-313Z-b2a537a0-12f7-45bc-993f-dd7631d760df/`. There was exactly one fullcore runtime attempt in this lane. No timeout/setup failure or case was removed to obtain this result.

The report has zero suite errors, zero fixture-server exceptions and18 ready Rust fixture runtimes. Six `/favicon.ico` requests returned404 and remain in the raw request log. There are no pending requests. Those404 responses were not normalized or described as zero HTTP errors.

All18 readiness records still report missing `WebglAddon`. Core passing does not establish WebGL availability or GPU execution. Original WebGL tests can catch a constructor failure and fall back to DOM, so the addon306/558 obligations remain separate.

## Source pairing and prior failures

The original JS core baseline is retained in archive `43ffa9b1d88c6d96b794aea3c5883079b22aa238` under `results/upstream-browser/runs/2026-11-06T00-57-19-880Z-085e4abf-05e5-4305-8f60-de0b5460177c/core-all-runtime.json`. It also has1194 passed,240 original skips and zero failures/unrun. Its server stopped and both reference hash checks passed. The complete original14-config baseline was not all green; this report makes no broader source-pass claim.

`results/core574/compare.mjs` reuses Reese's named-case identity algorithm and verifies the archived baseline raw/summary hashes against its immutable prior pins. The only identity adjustment is the generated top-level `.test.js` filename to `.test.ts`. No status, assertion, error text or deadline is normalized.

`source-pairing.json` maps every1434 case/project identity one-to-one:1194 passed in both and240 declared skips in both. It checks for duplicate/missing identities and compares actual raw counts. Every one of the27 prior Rust core failures in archive43's later complete core run now passes on current Rust. Those cases cover actual public decoration color/inverse, registration/errors and overview-ruler paths from537/554. Historical failed reports remain intact.

This named-case mapping is not assertion-level mapping or full-state equivalence. The thirteen historical addon configurations used different Rust inputs. They are not added to this current core result to fabricate a fresh4116-case matrix or a cross-source derived total.

## Package and cleanup custody

`results/link-demo-reload/package-554.tar.gz` retains the exact debug package and actual build report,85 regular files and zero links. It was re-extracted and independently hashed. Its size4645035 bytes and SHA256 `bd271e4e140cac5961b63d82e374951baa37396ff81132d9a911d8a1ad90b28d` are recorded with all file pins in `package-archive.json`. Reuse only with the exact19ff input digest and complete artifact verification, never as a release/performance distribution.

The fullcore used pointer is snapshotted and the exact pre-run bytes restored after evidence retention. A restored latest pointer is not proof of this runtime. Both original and owned physical reference copies are checked again after execution. Final owned O: cleanup follows only after proof commits, reviews and task release; the private cleanup receipt is authoritative for actual deletion.

Full native original assertion mapping, full E01-E05/demo/all14 addon current-source matrices, actual native Linux/macOS and interactive examples, quality304, safety checks and comparisons remain open. This scoped checkpoint must not accept327 or claim equivalence.
