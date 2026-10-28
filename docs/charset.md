# Character maps and designation state

This ports `src/common/data/Charsets.ts` and `src/common/services/CharsetService.ts` from the supplied xterm.js reference. It does not implement terminal printing or escape dispatch.

## Rust API mapping

| Upstream | Rust |
| --- | --- |
| `CHARSETS[id]` | `xterm_core::charset::builtin_charset(id)` |
| `DEFAULT_CHARSET` | `DEFAULT_CHARSET`, `None` |
| `ICharset` | `Charset`, a shared mutable UTF-16 map handle |
| `CharsetService.charset` | `CharsetService.charset` |
| `CharsetService.glevel` | `CharsetService.glevel` |
| `CharsetService.charsets` | `charsets()` and `charsets_mut()` |
| `setgLevel(g)` | `set_g_level(g)` |
| `setgCharset(g, map)` | `set_g_charset(g, map)` |
| `reset()` | `reset()` |

There are 12 distinct replacement maps, 15 defined designators and the undefined US designator `B`. Finnish `C`/`5`, Norwegian-Danish `E`/`6` and Swedish `H`/`7` share map identity. Unknown designators also return `None`. Tables are kept separate from designation logic in `charset_tables.rs`.

Maps retain complete UTF-16 replacements, including the Dutch `[` to `ij` mapping. The original InputHandler consumes only the first UTF-16 unit of a truthy replacement. Future InputHandler integration must preserve that behavior rather than expand `ij` into two printed cells. Custom maps can contain empty replacements and lone surrogate units.

`Charset::clone` shares mutable identity through an encapsulated `Rc<RefCell<...>>`. `get_first` copies one UTF-16 unit without allocating a replacement string; `get` returns owned replacement units. No public method exposes a live borrow guard. `set(key, None)` assigns an explicit undefined property; `remove` deletes a property and returns whether it existed. `snapshot` distinguishes missing, `CharsetValue::Undefined`, and `CharsetValue::Text`, including empty strings. Keys remain single UTF-16 units because terminal lookup uses `String.fromCharCode`, not arbitrary multi-unit strings.

A print run captures the handle, not frozen map contents. A callback that changes a property on that same object affects the next character; replacing the selected handle does not change the captured object. Built-in maps are cached per native thread, corresponding to a JavaScript realm or worker. The core is already `!Send`. This cache retains alias identity without a global mutex, leaked allocation or borrow across callbacks. Native thread isolation has its own platform-specific test; WASM worker execution remains a separate browser gate.

G levels use `usize`, preserving out-of-designation-array invocation and sparse array growth. Negative, fractional and non-finite JS property keys are not Rust indices. An unassigned slot is `None`. Calling `set_g_level` never grows the array. Designating a slot updates the active map only when it is the active level. Direct mutation via `charsets_mut()` does not refresh the active map, matching direct access to the original array. `reset()` clears all slots and invokes level zero.

`set_g_charset` returns `Result<(), CharsetError>` instead of a possible JavaScript array allocation exception. Overflow and failed allocation leave the complete service unchanged.

## Verification

`tests/charset.rs` ports all four original CharsetService cases without dropping assertions. It also compares every ASCII lookup across every designator and an unknown designator, plus a deterministic operation trace, using the independent JS oracle described in [charset-oracle.md](charset-oracle.md). Additional tests cover alias identity, sparse designation, direct array edits, custom UTF-16 maps, retained identity across service clones, reset and allocation boundaries.

Run from the repository root:

```sh
node scripts/charset-oracle.mjs
node scripts/build-slot.mjs --owner charset-check --lane native -- cargo test -p xterm-core --test charset --test charset_edges --test charset_mutation --test charset_terminal
node scripts/build-slot.mjs --owner charset-clippy --lane native -- cargo clippy -p xterm-core --lib --test charset --test charset_edges --test charset_mutation --test charset_terminal -- -D warnings
```

The fixture generator runs original assertion bodies against temporary import-resolved reference copies. It does not change `source_repo`. Source hashes, Node environment, seed and original suite counts are retained in `results/charset/oracle.json`. Rust execution evidence is retained separately under `results/charset/`.

This component alone does not establish overall terminal equivalence or native Linux/macOS and browser/WASM verification.
