# Cells and attributes

The Rust port of `source_repo/src/common/buffer/CellData.ts`, `AttributeData.ts`, and packed constants lives in `crates/xterm-core/src/buffer/`. This is one subsystem, not proof of full terminal equivalence.

## Contracts

`CellData` contains `content: u32`, `attributes: AttributeData`, and `combined_data: Vec<u16>`. Deref and DerefMut expose inherited attribute methods and fields. `CharData` names the four legacy tuple entries as `attr`, `chars`, `width`, and `code`.

JS strings map to UTF-16 unit vectors so lone surrogates survive. A combined string returns its last UTF-16 unit from `get_code`, even when that is a trailing surrogate. `None` represents JS NaN for an empty combined string. A normal empty cell returns `Some(0)`. `set_from_char_data` deliberately ignores the supplied legacy code and preserves stale combined storage and extended attributes, as the source does. Raw widths use signed `i32` shifts rather than validation to the usual 0 to 2 range, matching the legacy source contract.

Foreground, background and extended data use unsigned 32-bit bit patterns. Numeric flag methods return their original mask, including the high strikethrough bit as `0x80000000` rather than JS's signed `-2147483648`. Browser bindings must convert this to JS signed bitwise representation if exposing the internal numeric method. Public truthiness is unchanged. Colors retain upstream's signed `-1` default sentinel. Underline styles are raw packed `u32` values, retaining reserved values 6 and 7.

`ExtendedAttrs` stores valid internal OSC link IDs as `u64`, separate from the packed `u32` attribute word. This preserves all exactly representable generated IDs through JavaScript's `2^53` boundary without truncating at `2^32`. Direct assignment of arbitrary JS numeric values such as fractional, negative or NaN URL IDs is not a typed Rust operation; any binding exposing that internal setter must define those conversions separately. The OSC service generates nonnegative integer IDs.

A native payload destructor may execute user code. `set_payload` now publishes the new Rc, releases the private borrow, and then drops the previous payload. The native Drop regression observes the replacement and writes another payload through a shared handle; the outer setter must not overwrite that reentrant change. Task 73 was reopened when this missing ownership checkpoint was found. The full 33-cell debug suite and focused Clippy now pass, including this regression. Current release verification remains for the branch workflow; earlier release counts below are historical.

`ExtendedAttrs` privately owns `Rc<RefCell<Data>>`. `share()` retains the same mutable object, which buffer storage and cell loads need for JS reference identity. `same_identity()` compares those handles. Explicit `clone()` copies packed data and URL identity into a separate object and drops payload, matching upstream `ExtendedAttrs.clone()`. Rust's full `CellData::clone()` is a language addition, using that same attribute-clone rule. It is not JS reference assignment; buffer storage must use `extended.share()` instead. Payload uses `Option<Rc<dyn Any>>`; accessors return a shared opaque value. Setters do not hold a borrow across callbacks. No unsafe code or dependencies were added. These handles are local-thread objects like the rest of the event-driven core.

Hyperlinks force dashed underline style while retaining the raw style for use after the URL is cleared. Color and variant offset alone do not make extended attributes nonempty. `AttributeData::update_extended` sets or clears HAS_EXTENDED from that exact definition. Underline presence and underline style are intentionally different upstream queries. Attribute comparison ignores protection, URL identity, underline offset, and character content, but retains numeric underline-query differences.

## Verification

```sh
node scripts/cell-oracle.mjs
cargo test -p xterm-core --test cell_upstream --test cell_differential --test cell_branches --test cell_extended --test cell_url
cargo test -p xterm-core --release --test cell_upstream --test cell_differential --test cell_branches --test cell_extended --test cell_url
cargo clippy -p xterm-core --test cell_upstream --test cell_differential --test cell_branches --test cell_extended --test cell_url -- -D warnings
```

Observed on Windows with Node 24.15.0 and Rust/Cargo 1.85.1. The source oracle executes all five `CellData.test.ts` tests and the seven AttributeData/CellData tests at the beginning of `BufferLine.test.ts`. All 12 passed, with the assertions unmodified. Each has a Rust test in `tests/cell_upstream.rs`, all passing.

`cell_differential.rs` compares 2048 packed-cell snapshots and attribute pairs, plus 1024 legacy CharData cases generated from the original JS implementation. Seed is `0xce11da7a`. Every pair also checks symmetry and reflexivity. Five additional targeted regressions cover payload cloning, shared identity, hyperlink overrides, malformed strings, and RGB masking.

The resumed acceptance pass adds `cell_branches.rs` with eight deterministic branch matrices and `cell_extended.rs` with four tests, including 4096 JS-generated mutation operations across three objects. Mutations exercise raw words, style/color/offset setters, URL replacement, payload changes, reference assignment and explicit cloning. The snapshot after each step includes all object getters and pairwise identity.

C1 maps to `legacy_utf16_conversion_branches_and_signed_width_boundaries`, `raw_content_masks_and_out_of_range_codepoints_keep_js_truncation`, and the legacy fixture test. C2 maps to the mode/payload/RGB and individual flag tests, underline query distinctions and all raw snapshots. C3 maps to the four extended tests plus payload/identity/hyperlink regressions. C4 maps to three deterministic equality tests, upstream style/color/URL/offset cases and the 2048 raw pairs. C5 maps to the unchanged 12-case source baseline, exact fixture hashes and reproduction logs. These cover `docs/cell-acceptance.md`; no browser or terminal integration claim follows from them.

On the resumed Windows run, all 31 scoped Rust tests pass in debug and release. All 12 scoped upstream tests pass, with zero skipped or unrun within that stated scope. Regenerating the oracle twice yields identical hashes for all three fixtures. Focused rustfmt passes. Exact logs are `results/cell/run2-{debug,release,oracle,oracle-repeat,reproduction,fmt,clippy,toolchains}.log`. The first run2 Clippy snapshot failed in concurrent `log_service.rs`, `serialize/ansi.rs` and `buffer/reflow.rs`, not cell code. Owners were notified; that failure is retained rather than suppressed.

A subsequent integration fix widens valid OSC URL IDs beyond the packed-word width. `cell_url.rs` checks six original-JS fixtures at 0, 1, `2^32-1`, `2^32`, `2^53-1` and `2^53`. It verifies raw and visible style, URL clearing after clone, shared payload identity, and BufferLine fill/copy identity. All 32 cell tests plus the BufferLine/reflow/SGR dependent fixtures pass in `results/cell/url-tests-final.log`. This change does not expand the finite test set into a full-equivalence claim.

Source hashes and per-test original outcomes are in `results/cell/oracle.json`. Fixtures are `tests/fixtures/cell-raw.tsv`, `cell-char-data.tsv`, `cell-extended.tsv` and `cell-url.tsv`, under `crates/xterm-core/`. Reproduction instructions are also in `.pi/swarm/skills/cell-oracle/SKILL.md`. Focused Rust logs are `results/cell/focused-tests.log` and `focused-clippy.log`.

An integration snapshot ran all then-present workspace tests successfully. Its workspace fmt and Clippy checks failed on concurrently edited Color/APC modules, not cell files; retained logs are `results/cell/{fmt,clippy,workspace-tests}.log`. Owners were notified. These historical logs are not a claim about the eventual workspace status.

Remaining BufferLine tests belong to the separate BufferLine task. Full terminal integration, browser/WASM bindings and native Linux/macOS runtime evidence remain outside this subsystem and are not claimed here. Overall equivalence is NOT PROVEN.
