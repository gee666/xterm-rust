# Native search highlight column validation

Task560 repairs the native addon path only. Full Search parity, public raw matching347, disposed-cache ownership527 and family322 remain open. Full equivalence is NOT PROVEN.

## Repair

Source `SearchEngine.find` evaluates its raw term guard, then checks `startCol >= terminal.cols`, then initializes the cache. The Rust addon previously initialized its cache and read matching options and text before calling the core engine's column check. A term guard can resize the captured terminal before the second highlight call. With start column3 and new width2, Rust called the matching getter instead of returning the source invalid-column error. It also installed a new cache timeout after resize had invalidated the previous cache.

`crates/xterm-addons/src/search/find.rs` now checks the captured engine host immediately after that guard. A replaced addon host cannot change which terminal supplies the width. The check precedes cache setup and all matching callbacks.

The core keeps the checked `SearchEngine::find` API for direct callers. Its additional `find_after_column_check` method continues a host-phased operation without checking the column again. This distinction is deliberate. A later callback can resize the terminal after the source already accepted the column. The addon must not reject it at a second, later check. Both entry points share matching and traversal; neither adds JavaScript engine delegation.

## Observed results

Windows Rust/Cargo1.85.1 and Node24.15.0, base224d1b33, source frozen6e411202. Raw commands and outputs are under `results/search-highlight-bounds/`.

- Native106 functions PASS, zero failed, ignored or filtered. Addon71, cache18, differential3, integration6 and ring8. Two new functions cover eight addon resize/reactivation/guard-error scenarios and the core's checked versus previously-checked entry points. Existing seeded corpus assertions were not regenerated or changed.
- Old native control replaced only the addon `find.rs` with exact base224d bytes. New test0PASS/1FAIL/70filtered at `MissingOptions` instead of `InvalidColumn`. A finally restored the exact fixed bytes. The complete native106 then passed.
- Shared source fixture8 cases EACH Chromium148.0.7778.96, Firefox150.0.2 and WebKit26.4 PASS. Both directions, same-host/replacement-host and raw-guard throws are retained. The supplied original source and its physical worktree copy each retain all776 hashes.
- Scoped strict native Clippy and actual WASM library check PASS. All five whole-member formatting checks PASS. Exact aggregate `cargo fmt --all -- --check` FAILS with WindowsOS206, retained in `fmt-all.log`. No aggregate formatting pass is claimed.
- Full workspace/all-feature tests, release, full unchanged upstream suites, Linux and macOS runtime checks were not run for560. Those obligations remain in the existing full-scope tasks.

## Public browser failure retained

This native repair is not a passing public-WASM bounds regression. The same fixture first needs a valid raw object with `length` and `toLowerCase` methods to reach the second engine call. Public `JsTerm::text` still rejects that object, an existing347 gap. Actual Rust/WASM Chromium ran the complete source assertions unchanged and produced0PASS/1FAIL, no ignored or filtered functions. It stopped after two length reads rather than four, with `TypeError: Search term must be a string`. No second-guard bounds outcome was reached.

The initial report lacked the thrown-value diagnostic. The final run changed only the fixture's failure diagnostic, not its expected values. Both reports, compiler identities and logs are retained under `public-initial/` and `public-final/`. The final source fixture was rerun in all three engines. Firefox/WebKit public-WASM bounds execution is UNRUN. No public-WASM passing bounds proof is claimed.

The failing public test is retained as `results/search-highlight-bounds/public-counterexample.rs`, alongside the shared fixture and raw failing results. It is a recorded counterexample, not an ignored test or a successful compatibility assertion. It is outside the passing native test target because public raw matching has not been repaired. Do not treat a green native run as closure of this browser failure. Task347 must activate this regression when implementing valid raw matching and make its unchanged source assertions pass.

## Reproduction

Run from an owned checkout with the normal Rust prerequisites. During swarm work, the checkout, target and TEMP/TMP/TMPDIR must be on O:, and a heavy allocation is required. Native commands use Cargo `-j2`.

```text
node results/search-highlight-bounds/native-reproduce.mjs
SEARCH_CASES=search-highlight-bounds node scripts/search-call-args-source.mjs
```

The source runner needs SOURCE_DEPENDENCIES and PLAYWRIGHT_MODULE pointing to the existing original dependency installation when the physical reference copy lacks dependencies. No reference inputs are changed.

To replay the known failing public counterexample, use this Node program from the checkout root. Set WASM_BINDGEN to the installed0.2.100 CLI and PLAYWRIGHT_MODULE to Playwright's index.mjs. It refuses to overwrite an existing test and removes only the exact file it created. Exit1 and the raw TypeError remain a failure, not an expected-success substitute.

```js
import { readFile, writeFile, unlink } from 'node:fs/promises';
import { spawnSync } from 'node:child_process';
const path = 'crates/xterm-web/tests/wasm_search_highlight_bounds.rs';
const bytes = await readFile('results/search-highlight-bounds/public-counterexample.rs');
await writeFile(path, bytes, { flag: 'wx' });
try {
  const result = spawnSync(process.execPath, ['scripts/browser-host-check.mjs'], {
    env: { ...process.env, WASM_TEST: 'wasm_search_highlight_bounds', BROWSERS: 'chromium', CARGO_BUILD_JOBS: '2' },
    stdio: 'inherit', timeout: 180000
  });
  if (result.error) throw result.error;
  process.exitCode = result.status ?? 1;
} finally {
  if (!(await readFile(path)).equals(bytes)) throw Error('Counterexample changed; preserve it');
  await unlink(path);
}
```

`inputs.json` pins1909 tracked crate/build inputs. `runtime-pins.json` describes twelve retained runtime files for independent inspection before owned cleanup. Historical raw reports retain paths that will no longer exist after cleanup. Git source and the durable logs remain the evidence, not a promise that removed binaries can still run.

## Independent review and cleanup preflight

Task560 was accepted at e6b5d7c6 with no findings. The isolated reviewer reran all five pinned native executables,106PASS, and verified43 then-current durable pins,1909 input pins,12 runtime pins and both776 reference inventories. No compilers, browsers, servers or file edits were performed by the reviewer. The public-WASM failure and open347/527/322 scope were checked explicitly.

The read-only cleanup preflight found17489 regular files,3014 directories,1772527880 bytes, no links, redirects or owned workers. This is not a cleanup-complete claim. Two final metadata files and this note may change the apply scan counts. All source and evidence are preserved before normal owned worktree removal. The actual apply receipt, after removal, belongs at `.pi/swarm/work/2/john560-cleanup.json`; verify its complete/rootAbsent/unregistered flags and original776/archive-pin checks before treating cleanup as finished.
