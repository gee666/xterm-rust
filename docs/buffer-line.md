# BufferLine port and evidence

`crates/xterm-core/src/buffer/line.rs` implements the packed row and sparse cell data. `line_edit.rs` contains range edits and directional copies. `line_text.rs` contains UTF-16 translation and canonical caching. Extended attributes use the shared CellData representation, with `ExtendedAttrs::share()` preserving the mutable object and payload identities.

This is one core component. Complete terminal equivalence is NOT PROVEN.

## API mapping

| Upstream | Rust contract |
| --- | --- |
| constructor | `BufferLine::new(cols, Option<&CellData>, is_wrapped)` |
| length, isWrapped | `len()`, `is_empty()`, `is_wrapped` |
| get, set | `get`, `set` using named `CharData` fields |
| getCell | `get_cell(index) -> Option<CellData>` |
| loadCell | `load_cell(index, &mut cell) -> Option<&mut CellData>` |
| getWidth, hasWidth | `get_width`, `has_width` |
| getFg, getBg | `get_fg`, `get_bg` |
| hasContent, getCodePoint | `has_content`, `get_code_point` |
| isCombined, getString | `is_combined`, `get_string` |
| isProtected, getExtended | `is_protected`, `get_extended` |
| setCell, setCellFromCodepoint | `set_cell`, `set_cell_from_codepoint` |
| addCodepointToCell | `add_codepoint_to_cell` |
| insertCells, deleteCells | `insert_cells`, `delete_cells` |
| replaceCells, fill | `replace_cells`, `fill`, explicit protection Boolean |
| resize | `resize(cols, fill) -> bool` |
| cleanupMemory | `cleanup_memory() -> bool` |
| copyFrom, clone | `copy_from(source, blank)`, `clone_line(blank)`, `Clone` |
| copyCellsFrom with distinct rows | `copy_cells_from` |
| copyCellsFrom with source == target | `copy_cells_within`, direction preserved |
| getTrimmedLength | `get_trimmed_length` |
| getNoBgTrimmedLength | `get_no_bg_trimmed_length` |
| translateToString | `translate_to_string(trim, start, end, out_columns) -> Vec<u16>` |
| DEFAULT_ATTR_DATA | `AttributeData::default()`, passed by borrow |

Strings retain UTF-16 units rather than forcing valid UTF-8. This includes lone surrogates and the source's last-unit combined-cell codepoint behavior. Primitive getters return Option for a missing index. Out-of-row mutations have no effect, and zero-column insertion/deletion terminate without modulo-by-zero. These boundary choices are explicit Rust representations rather than a promise of JavaScript numeric indexing for negative or fractional indices.

A `blank=true` clone/copy is an internal optimization with the same source precondition: the row must actually be blank. Call ordinary `clone()` or `copy_from(..., false)` otherwise. Default no-extended values do not expose the source's mutable module-global scratch object. Flagged extended objects do preserve identity across rows and loaded cells.

The retained backing-cell count models the source's typed-array backing allocation. It is independent of Rust allocator capacity. Cleanup follows the source's strict two-to-one threshold and preserves active contents.

## Original assertion mapping

The original file contains 51 tests: 44 BufferLine, six AttributeData and one CellData. The neighboring seven belong to the cell task and are not counted as 44 new row tests.

Paths below abbreviate `crates/xterm-core/tests/`. The cache tests live inside `src/buffer/line_text.rs` because they inspect protected implementation state, as the original subclass does.

| Original BufferLine test | Rust file and test |
| --- | --- |
| ctor | buffer_line_upstream.rs, ctor |
| insertCells | buffer_line_upstream.rs, insert_cells |
| deleteCells | buffer_line_upstream.rs, delete_cells |
| replaceCells | buffer_line_upstream.rs, replace_cells |
| fill | buffer_line_upstream.rs, fill |
| clone | buffer_line_upstream.rs, clone |
| copyFrom | buffer_line_upstream.rs, copy_from |
| should support combining chars | buffer_line_upstream.rs, should_support_combining_chars |
| resize / enlarge false | buffer_line_upstream.rs, resize_enlarge_false |
| resize / enlarge true | buffer_line_upstream.rs, resize_enlarge_true |
| resize / shrink true | buffer_line_upstream.rs, resize_shrink_true |
| resize / shrink to zero | buffer_line_upstream.rs, resize_shrink_to_zero |
| resize / remove combining data | buffer_line_upstream.rs, resize_removes_cut_combining_data; line_storage_tests.rs, upstream_shrink_regrow_sparse_map_cardinality |
| getTrimLength / empty | buffer_line_upstream.rs, trim_length_empty |
| getTrimLength / ASCII | buffer_line_upstream.rs, trim_length_ascii |
| getTrimLength / surrogate | buffer_line_upstream.rs, trim_length_surrogate |
| getTrimLength / combining | buffer_line_upstream.rs, trim_length_combining |
| getTrimLength / fullwidth | buffer_line_upstream.rs, trim_length_fullwidth |
| translate / empty | buffer_line_translation.rs, empty_line |
| translate / ASCII | buffer_line_translation.rs, ascii |
| translate / surrogate | buffer_line_translation.rs, surrogate |
| translate / combining | buffer_line_translation.rs, combining |
| translate / fullwidth | buffer_line_translation.rs, fullwidth |
| translate / space at end | buffer_line_translation.rs, space_at_end |
| translate / sane value | buffer_line_translation.rs, should_always_return_sane_value |
| translate / endCol zero | buffer_line_translation.rs, end_column_zero |
| addChar / empty width one | buffer_line_upstream.rs, combining_sets_width_on_empty_cell |
| addChar / append combined | buffer_line_upstream.rs, combining_add_to_combined_cell |
| addChar / create combined | buffer_line_upstream.rs, combining_create_on_taken_cell |
| fullwidth / insert at pos | buffer_line_translation.rs, insert_wide_char_at_pos |
| fullwidth / insert at end | buffer_line_translation.rs, insert_wide_char_at_end |
| fullwidth / delete | buffer_line_translation.rs, delete_fullwidth |
| fullwidth / replace start zero | buffer_line_translation.rs, replace_fullwidth_start_zero |
| fullwidth / replace start one | buffer_line_translation.rs, replace_fullwidth_start_one |
| extended / setCells | buffer_line_extended.rs, set_cells |
| extended / loadCell | buffer_line_extended.rs, load_cell |
| extended / fill | buffer_line_extended.rs, fill |
| extended / insertCells | buffer_line_extended.rs, insert_cells |
| extended / deleteCells | buffer_line_extended.rs, delete_cells |
| extended / replaceCells | buffer_line_extended.rs, replace_cells |
| extended / clone | buffer_line_extended.rs, clone |
| extended / copyFrom | buffer_line_extended.rs, copy_from |
| canonical cache | line_text.rs, upstream_canonical_cache_contract |
| invalidate cache | line_text.rs, upstream_all_mutations_invalidate_cache |

The two original enlarge cases have identical setup despite their names. Both remain separate Rust tests. Extra regression tests cover extended reuse and payload sharing, protected boundary edits, cleanup thresholds, blank copying, zero-row boundaries, sparse map pruning, empty combined strings and JavaScript trimEnd's whitespace set.

## Reproducible commands

Run from the repository root with Rust 1.85 or newer and Node 24.15 or newer. The JavaScript oracle needs the supplied read-only reference under `source_repo`.

```text
node scripts/buffer-line-oracle.mjs
cargo test -p xterm-core --test buffer_line_upstream --test buffer_line_translation --test buffer_line_extended --test buffer_line_differential
cargo test -p xterm-core --lib buffer::line
cargo clippy -p xterm-core --lib --test buffer_line_upstream --test buffer_line_translation --test buffer_line_extended --test buffer_line_differential -- -D warnings
cargo fmt --all -- --check
```

Observed native Windows results are retained under `results/buffer-line/`:

- `oracle.json` records all 51 original cases passed, the source hashes, Node version, seed `0xb0ff113e`, 128 traces, 8192 steps and the fixture SHA256.
- `implementation-tests.log` records 46 external tests passed. These comprise 42 original row cases, two extra edit/reuse regressions, one differential replay and one extended identity/payload regression.
- `implementation-unit.log` records nine internal tests passed. Two complete the 44-case original row mapping. One preserves the source's exact sparse-map count assertions; six cover additional storage/cache boundaries.
- `implementation-clippy.log` records targeted warning-free Clippy. `implementation-fmt.log` records formatting of owned source and tests.
- `rust-toolchain.txt` records actual compiler, Cargo and Node versions.

Independent oracle and review work is task 52, owned by Isaac. Review findings and independent command logs belong in `review-*.log` and the review notes. The full workspace was changing during these checks, so these targeted results must not be reported as a workspace-wide green gate. Native Linux/macOS execution, terminal integration, browser/WASM and addons are outside this evidence.
