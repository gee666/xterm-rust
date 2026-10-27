# Cells and attributes

The Rust port of `source_repo/src/common/buffer/CellData.ts`, `AttributeData.ts`, and packed constants lives in `crates/xterm-core/src/buffer/`. This is one subsystem, not proof of full terminal equivalence.

## Contracts

`CellData` contains `content: u32`, `attributes: AttributeData`, and `combined_data: Vec<u16>`. Deref and DerefMut expose inherited attribute methods and fields. `CharData` names the four legacy tuple entries as `attr`, `chars`, `width`, and `code`.

JS strings map to UTF-16 unit vectors so lone surrogates survive. A combined string returns its last UTF-16 unit from `get_code`, even when that is a trailing surrogate. `None` represents JS NaN for an empty combined string. A normal empty cell returns `Some(0)`. `set_from_char_data` deliberately ignores the supplied legacy code and preserves stale combined storage and extended attributes, as the source does. Raw widths use signed `i32` shifts rather than validation to the usual 0 to 2 range, matching the legacy source contract.

Foreground, background and extended data use unsigned 32-bit bit patterns. Numeric flag methods return their original mask, including the high strikethrough bit as `0x80000000` rather than JS's signed `-2147483648`. Browser bindings must convert this to JS signed bitwise representation if exposing the internal numeric method. Public truthiness is unchanged. Colors retain upstream's signed `-1` default sentinel. Underline styles are raw packed `u32` values, retaining reserved values 6 and 7.

`ExtendedAttrs` privately owns `Rc<RefCell<Data>>`. `share()` retains the same mutable object, which buffer storage and cell loads need for JS reference identity. `same_identity()` compares those handles. Explicit `clone()` copies packed data and URL identity into a separate object and drops payload, matching upstream `ExtendedAttrs.clone()`. Payload uses `Option<Rc<dyn Any>>`; accessors return a shared opaque value. Setters do not hold a borrow across callbacks. No unsafe code or dependencies were added. These handles are local-thread objects like the rest of the event-driven core.

Hyperlinks force dashed underline style while retaining the raw style for use after the URL is cleared. Color and variant offset alone do not make extended attributes nonempty. `AttributeData::update_extended` sets or clears HAS_EXTENDED from that exact definition. Underline presence and underline style are intentionally different upstream queries. Attribute comparison ignores protection, URL identity, underline offset, and character content, but retains numeric underline-query differences.

## Verification

```sh
node scripts/cell-oracle.mjs
cargo test -p xterm-core --test cell_upstream --test cell_differential
cargo clippy -p xterm-core --test cell_upstream --test cell_differential -- -D warnings
```

Observed on Windows with Node 24.15.0 and Rust/Cargo 1.85.1. The source oracle executes all five `CellData.test.ts` tests and the seven AttributeData/CellData tests at the beginning of `BufferLine.test.ts`. All 12 passed, with the assertions unmodified. Each has a Rust test in `tests/cell_upstream.rs`, all passing.

`cell_differential.rs` compares 2048 packed-cell snapshots and attribute pairs, plus 1024 legacy CharData cases generated from the original JS implementation. Seed is `0xce11da7a`. It covers every attribute query, arbitrary bit patterns, UTF-16 strings, raw widths, URL overrides and equality. Five additional targeted Rust regressions cover payload cloning, shared identity, hyperlink overrides, malformed strings, and RGB masking. All seven differential/regression tests pass.

Source hashes and per-test original outcomes are in `results/cell/oracle.json`. Fixtures are `tests/fixtures/cell-raw.tsv` and `cell-char-data.tsv`, under `crates/xterm-core/`. Reproduction instructions are also in `.pi/swarm/skills/cell-oracle/SKILL.md`. Focused Rust logs are `results/cell/focused-tests.log` and `focused-clippy.log`.

An integration snapshot ran all then-present workspace tests successfully. Its workspace fmt and Clippy checks failed on concurrently edited Color/APC modules, not cell files; retained logs are `results/cell/{fmt,clippy,workspace-tests}.log`. Owners were notified. These historical logs are not a claim about the eventual workspace status.

Remaining BufferLine tests belong to the separate BufferLine task. Full terminal integration, browser/WASM bindings and native Linux/macOS runtime evidence remain outside this subsystem and are not claimed here. Overall equivalence is NOT PROVEN.
