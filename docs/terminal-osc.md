# Terminal OSC handlers

`crates/xterm-core/src/terminal/osc.rs` and `osc_colors.rs` port the OSC InputHandler registrations 0/1/2/4/8/10/11/12/104/110/111/112. The real escape parser invokes these callbacks over shared TerminalState. No original JavaScript engine is used by the Rust implementation.

| Source behavior | Rust integration |
| --- | --- |
| title/icon mutation | `TerminalState::set_title` and `set_icon_name`, internal helpers shared with CSI window handling |
| `_onTitleChange` | ordered `TerminalEvent::TitleChange(Vec<u16>)` |
| `_onColor` | `TerminalEvent::Color(Vec<ColorRequest>)` |
| color indices 0..255/256/257/258 | `ColorIndex::Indexed(u8)/Foreground/Background/Cursor`; `protocol_index()` exposes source numbers |
| REPORT/SET/RESTORE | `ColorRequest::Report/Set/Restore`; Restore with None means the indexed palette |
| OSC8 link registration | shared `TerminalState.links: Rc<OscLinkService>` and extended u64 cell URL ids |
| printing a current link | live URL id lookup after accessibility callback, before source wrap handling; captured cell attributes are separate |

Titles, icons, ids and URIs retain UTF-16 units. OSC0 dispatches its title event before icon mutation. No State borrow crosses that observer. Indexed color requests remain one ordered batch, while special colors dispatch one event per valid slot. Invalid slots still advance the special index. Listener errors stop later side effects and retain the original error object through error source chains.

OSC8 splits at the first semicolon, trims params with ECMAScript whitespace and uses the first exact `id=` segment. URI semicolons remain intact. Empty URI with nonempty params returns false; an absent separator is handled without changes. Opening/closing clones extended attributes so existing cells keep their URL identity. Explicit clones drop payload as source does; ordinary printing shares existing payload instead. Source print-call capture and reentrant reset details are owned by task 153, with directed cell regression tests here.

Color requests do not invent a headless palette. A native raw observer can choose host policy. The browser adapter must use its actual ThemeService and current palette for Set/Report/Restore. A Report for an indexed color replies `ESC ] 4 ; index ; rgb:rrrr/gggg/bbbb ESC \\`; special indices 256/257/258 reply through OSC 10/11/12. `input::xparse_color::to_rgb_string(color, None)` supplies the same 16-bit representation. Real browser processing, replies, canvas colors and link activation remain tasks 31/32/35, not established by these native checks.

## Original assertions

`tests/terminal_osc_upstream.rs` maps all 12 source OSC case bodies in InputHandler.test.ts lines 2141-2310. All 33 original expected assertions are retained. Two source deep-object checks for link data become separate id and URI field assertions in Rust, so the Rust test has 35 assertion macros. The original Mocha/Chai runner discovers/executes/passes all 12 with no failures or pending cases. The driver compiles original source and assertions directly with esbuild; it does not rewrite them.

## Differential snapshots

`scripts/check-terminal-osc.mjs` creates a real source InputHandler with BufferService, CoreService, CharsetService, UnicodeService configured to V6, and OscLinkService. The Rust adapter creates the actual CoreTerminal with 4 columns, 3 rows and 2 scrollback rows. Both consume the same UTF-16 writes.

The corpus has 30 directed operations, 512 randomized fragments at xorshift32 seed 1330856788, resets every 16 fragments and random chunk splits. The resulting 706 snapshots compare:

- exact title/icon units and title observer-time values;
- ordered color event batch boundaries, indices and RGB channels;
- current packed attributes, extended data and URL id;
- every allocated normal/alt row and cell, packed content/fg/bg/extended, URL id and UTF-16 text;
- active buffer, cursor, scrollback base, viewport display and scroll margins;
- live marker ids, lines and disposed state;
- every live numeric link entry and exact id/URI data, including entries without a printed cell.

`OscLinkService::link_ids()` is a Rust-only diagnostic inventory used to compare the complete numeric store rather than guessing an id range. It does not mutate service state. Snapshot strings are compared directly, not only through a digest. The generated 706-line TSV intentionally exceeds the source-file line cap because it is oracle data. A mismatch retains its full input prefix and both snapshots. Default verification rejects fixture drift. `--write-fixtures` is only for intentional reviewed regeneration.

The fixture is not a full terminal equivalence proof. It does not compare every mode, option, saved-cursor field or addon. Other native platforms and browser behavior remain separate gates.

## Focused tests and current evidence

```sh
node scripts/check-terminal-osc.mjs
cargo test -p xterm-core --test terminal_osc_upstream --test terminal_osc_edges --test terminal_osc_differential --test terminal_osc_payload --test terminal_osc_cells
cargo clippy -p xterm-core --lib --test terminal_osc_upstream --test terminal_osc_edges --test terminal_osc_differential --test terminal_osc_payload --test terminal_osc_cells --example terminal_osc_oracle -- -D warnings
```

Current native Windows checks with Node 24.15.0 and Rust 1.85.1 pass:

- 12 original Mocha cases and 33 assertions passed.
- 29 Rust tests pass, including the 12 migrated cases, event/error/chunk edges, full snapshot replay, real payload limit and six cell/UTF-8/identity cases.
- 706 source/Rust snapshots matched at the recorded seed.
- Focused Clippy passed with warnings denied.

The current 706 snapshots include marker ids and every live numeric link entry. The six cell cases cover UTF-8 byte chunks, lone-surrogate link data, shared payload versus explicit clone, high URL ids, wide combining cells and alternate-buffer close/restore. Original assertions remain unchanged. Use `results/terminal-osc/latest.json` for the current result; older root summaries are historical.

Results are retained in `results/terminal-osc/`, with original source hashes, commands, runtime, seed and counts. Full terminal equivalence remains NOT PROVEN. Acceptance is enumerated in `docs/acceptance/osc-input.md` and criteria 391-395, with service criteria 247-250 and cell-carrier task 117.
