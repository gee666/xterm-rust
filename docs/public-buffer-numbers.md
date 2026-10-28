# Public buffer Number behavior

Task 136 owns `public_api/line_numbers.rs`, `public_api/line_translation.rs`, the crate-private `buffer/line_numeric.rs` packed readers, and the numeric cell/line methods in the WASM buffer bindings. Task 66 retains namespace/view/parser ownership. The usize native BufferLine API does not change. Epictetus approved the private child module and its declaration.

## NUM-CELL required cases

Reference methods are `BufferLine.getCell/loadCell/getExtended`, all exposed CellData getters, and AttributeData's flags/colors/underline/equality methods.

- Check the original Number against bounds before packing. Negative values, negative subnormals, both infinities, length and larger indices return undefined. Negative zero is zero. NaN passes the original comparisons and returns a cell with undefined content/fg/bg, not undefined cell and not a normal default cell.
- Compute `x * 3`, then each word offset independently in f64. A third-cell fraction can read foreground as content and background as foreground. Other fractions address non-integer TypedArray keys and yield undefined. Do not truncate, round or cast x to a machine integer first.
- Retain each missing word separately. Bitwise getters see zero, while `isAttributeDefault` remains false for undefined foreground/background because its source comparison is strict. Missing combined data returns undefined from getChars and throws TypeError from getCode. Missing extended data throws only when the invoked method actually dereferences it.
- Compare every exposed width/color/flag/underline getter and both equality paths. Preserve the source's early color/flag exits before extended getters. Keep shared extended data live; do not clone it while loading a destination. Host getters must remain lazy so callback failures and attribute mutations do not silently become cached values.
- Load a provided destination by assigning content, fg, bg, combinedData and extended in source order. Support managed getNullCell destinations and arbitrary objects. Falsey destinations request a new cell. Keep object identity, partial assignments and the original thrown JS value when a setter rejects. JS accessor mutation during the load still requires explicit source/Rust operation traces, not a claim based on a plain object.
- Preserve UTF-16 strings, including malformed units. Non-numeric JavaScript coercion, mutable raw cell fields outside declared types, cross-terminal default-extended object identity, payload cycles and arbitrary custom accessor behavior need separate browser evidence. They are not established by a native f64 test.

`NumericCell` uses Option for raw undefined fields, separate from bitwise-normalized CellData. Native methods that dereference missing combined/extended data return typed errors. The browser converts those failures into TypeError. The ordinary usize interface still returns None outside the line.

## NUM-TEXT required cases

Reference is `BufferLine.translateToString`, including its canonical cache, range loop, UTF-16 output and optional column map.

- Preserve omitted/nullish defaults separately from Number NaN. Canonical full-line requests use the existing source-compatible cache; range and column-map requests do not. Cover repeated full/trimmed observations after mutations and range requests.
- Negative starts pad with spaces. End beyond length can pad. Reversed, equal, NaN and infinite-start ranges follow the actual loop comparison. Fractional positions read packed words at `start * 3`, not rounded cells. Advance by the signed packed width, with zero replaced by one.
- Combined strings, wide cells, trailing width-zero cells and invalid TypedArray addresses must produce identical UTF-16. Undefined combined strings join as empty entries without a column map, but accessing their length for a map throws TypeError. Every UTF-16 unit gets the original floating point column; append the final position even for an empty loop.
- Match trimRight's Math.min NaN behavior and canonical-cache trimEnd behavior. Do not apply a blanket start/end clamp or silently convert to usize.
- The safe Rust helper explicitly rejects infinite/non-progressing/cyclic ranges and propagates allocation failures. Source can loop until process exhaustion on these inputs. This is a deliberate safety difference, not demonstrated source equivalence. Keep this limitation visible until the project adopts and documents an accepted compatibility decision. There is no successful empty-string fallback for a non-terminating source loop.

## Commands and current evidence

`node scripts/public-cell-numbers.mjs` runs original classes bundled without edits, verifies fixture drift and executes the production Rust helpers. `--generate` is only for reviewed fixture generation. It records exact source/input hashes before/after execution and retains per-attempt logs under `results/public-cell-numbers/`.

The retained Windows run has 2,071 Number cases, including 2,048 seeded IEEE-754 patterns with seed 1314213187, and 660 translation/column-map cases. Five focused tests and three differential/safety tests passed, including scalar metadata that does not clone long combined text. Generated fixture length is intentional comparison data, not an oversized implementation module. The result lists browser and coercion omissions explicitly.

For the actual JS/WASM boundary run `WASM_TEST=public_buffer_numbers_wasm node scripts/browser-host-check.mjs`, with Cargo on PATH. This uses the existing bounded browser runner and shared target directory. The six browser tests invoke exported Rust buffer methods in JavaScript, checking all numeric getter fixtures and both equality paths, 330 public three-argument translations, falsey/managed/arbitrary destinations, shared attribute mutation, setter failures and setter-triggered row changes. Required browsers are Chromium, Firefox and WebKit. Attempts so far failed during unrelated library integration compilation; no browser pass is claimed. Local builds now run through the controlled `scripts/build-slot.mjs` queues with the shared target/profile and 1 GiB guard. `scripts/public-buffer-numbers-browser.mjs` wraps the existing browser runner with full core/addon/web input snapshots and requires one passing result for each of Chromium, Firefox and WebKit.

Native and WASM changes remain in progress. No independent acceptance, native Linux/macOS result or complete public buffer equivalence is claimed here.
