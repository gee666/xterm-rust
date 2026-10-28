# OSC link id and marker service

`crates/xterm-core/src/osc_link_service.rs` ports `source_repo/src/common/services/OscLinkService.ts`. It uses the actual Rust BufferService and Marker implementation, not a marker-provider mock.

| Upstream | Rust |
| --- | --- |
| `IOscLinkData` | `OscLinkData { id: Option<Vec<u16>>, uri: Vec<u16> }` |
| passed/returned data identity | `LinkDataRef = Rc<RefCell<OscLinkData>>` |
| constructor with BufferService | `OscLinkService::new(Rc<BufferService>)` |
| `registerLink(data)` | `register_link(LinkDataRef) -> Result<u64, OscLinkError>` |
| `addLineToLink(id, y)` | `add_line_to_link(u64, i64) -> Result<(), OscLinkError>` |
| `getLinkData(id)` | `get_link_data(u64) -> Option<LinkDataRef>` |
| Rust diagnostic inventory | `link_ids() -> Vec<u64>`, sorted, read-only numeric-store keys |

The active buffer is consulted live. Named keys concatenate id, two semicolons and URI as UTF-16. The source does not escape that separator, so different input objects can intentionally collide. Empty ids remain different from absent ids. Repeated unnamed links always allocate new numeric ids.

Data remains mutable and lookup returns the same object identity. A named key is captured at creation. In particular, changing data.id to absent before its last marker disposal retains the original named key while removing numeric lookup. Re-registering that key then returns a stale id. This odd behavior is tested because it is what the source does, not something the port should silently repair.

Lines deduplicate by each marker's current line only, even across different active buffers. Marker callbacks remove by true object identity, not numeric marker id. One surviving line keeps the link entry alive. The last disposal removes the numeric entry and conditionally the original named key according to current data.id.

Link ids are service-local u64 values for the JavaScript valid integral counter domain. The counter saturates at2^53. Every displaced live entry remains retained until its markers expire, including an entry removed from numeric lookup by another saturated-id marker. This preserves the three-entry source case in which the second marker must still remove a newly registered third entry. Independent review found this gap; the retained counterexample is `results/reviews/willie-osc-link/saturation-counterexample.json`. Tests cover that boundary without creating2^53 real links. Internal cell URL ids must also be wide enough, tracked by task117. Arbitrary fractional or negative JavaScript-number ids and accessor-based data objects are not represented by this typed service API.

No buffer/data borrow crosses a user callback. A borrow conflict produces a typed recoverable error. A marker cleanup callback wraps a borrowed-data error in EmitError::Callback, retaining its identity and source chain. As in Marker, invalidation and partial cleanup remain committed if a listener fails. Later disposal does not replay it.

Callbacks capture weak service/entry/marker references. This creates no service-marker ownership cycle. Dropping the last service handle releases its maps without disposing buffer-owned markers or calling listeners unexpectedly. A later buffer-marker disposal is safe after the service has gone away.

## Reproduction and assertion accounting

Install original test dependencies through the shared original baseline workflow, without creating a duplicate dependency tree:

```sh
npm --prefix source_repo ci --ignore-scripts --no-audit --no-fund
node scripts/check-osc-link-service.mjs
cargo test -p xterm-core --test osc_link_service
cargo test -p xterm-core --lib osc_link_service
cargo clippy -p xterm-core --lib --test osc_link_service --example osc_link_oracle -- -D warnings
```

The checker rebuilds its original BufferService/OscLinkService dependencies and the original test file from source with esbuild. It runs actual Mocha and Chai against the original three test bodies and five assertions. Source assertions are neither rewritten nor replaced by a fake service. Three original cases passed with no failures or pending cases in the retained run.

A deterministic shared line protocol drives actual JavaScript and Rust buffer-backed services. The fixture has31 directed operations and1,200 xorshift32 operations, seed1330856780. All1,231 state snapshots matched in the retained run. Snapshots include returned numeric ids, live link data for every returned id, active-buffer identity and every allocated marker's line/disposed state. Inputs cover current/moved/negative lines, buffer switching and scrolling, unknown ids, delimiter collisions, live-data mutation, UTF-16 surrogates and marker disposal. Separate direct source and Rust unit checks cover the otherwise impractical2^53 overwrite boundary.

The generated1,231-line fixture is oracle data and intentionally exceeds the normal source-file line cap. Default verification compares it with fresh source output before comparing Rust. `--write-fixtures` is only for intentional reviewed fixture generation. A failure retains its complete operation prefix and both outputs in counterexample.json. Temporary modules stay under tmp/osc-link-service/.

The focused Rust suite contains13 tests, with its first three preserving every upstream assertion, plus two private counter-boundary unit tests. The 13 integration tests and both unit tests passed, retained in rust-tests.log and unit-tests.log. After the review finding, both counter unit tests, six direct source saturation assertions and the 1,231-operation differential were rerun successfully. The later read-only diagnostic inventory and its assertions await a fresh run after the coordinated disk pause. The fresh1,231-operation executable comparison passed separately in summary.json. Focused Clippy passed with warnings denied after shared headless/input findings and the trace adapter's format-collection allocation were fixed. No warnings are suppressed. Exact output is retained in clippy.log.

`results/osc-link-service/` retains baseline output, runtime/dependency/source/fixture metadata and build failures. This service does not prove OSC8 parsing, full cells or browser link behavior. Those gates are task113/35 and criterion250. Native Linux/macOS and browser execution remain unrun here. Full terminal equivalence is NOT PROVEN.
