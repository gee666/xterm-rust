# Serialize acceptance contract

This is the task 37 checklist, not a claim that the addon passes. Reference files are `source_repo/addons/addon-serialize/src/SerializeAddon.ts`, its adjacent test file, and `typings/addon-serialize.d.ts`. Source assertions remain unchanged. Every item requires an exact source/Rust comparison or an identified upstream assertion, with command and result retained under `results/serialize/`.

## ANSI cells and attributes, criterion 18

- Empty default buffer serializes to no text, with cursor moves only when needed. Empty trailing default rows are omitted if no scrollback is retained.
- Normal text, actual spaces, empty cells, combined UTF-16 strings, non-BMP text, lone UTF-16 surrogates and wide-character placeholder cells retain their different meanings. Width-zero placeholders never emit text or style.
- Internal empty cells emit cursor-forward, not spaces. Background-colored gaps emit erase-character before cursor-forward. Trailing colored gaps emit erase-character without an unnecessary cursor move.
- A style change after a pending gap flushes the old background before changing SGR. A foreground-only change on an empty cell does not cause a visible style update.
- Equal rendering attributes emit no SGR. Protection, hyperlink identity and underline variant metadata are not serialized as rendering changes.
- Default attributes reset with SGR 0 only when the old attributes are not default.
- Foreground and background transitions cover default, ANSI 0-7, bright ANSI 8-15, palette 16-255, RGB including zero and 255 components. Test each direction, including palette-to-RGB and RGB-to-default.
- Flag transitions cover inverse, bold, underline, overline, blink, invisible, italic, dim and strikethrough. Preserve upstream ordering and separate bold/dim SGR 22 behavior rather than normalizing sequences.
- Underline none/single/double/curly/dotted/dashed retain SGR 24, 4 or 4:N. Default/palette/RGB underline color follows upstream 58:5:N and 58:2::R:G:B spelling. Both-nonunderlined attributes ignore stale underline colors.
- Unwrapped rows use CR LF. Naturally wrapped rows omit a separator only when upstream last-cell, second-last-cell, next-first-cell width/background and pending-null tests permit it.
- Invalid wrap boundaries force wrapping using temporary hyphens, cursor movement and erase operations. Verify 0, 1 and multiple pending null cells, wide next-row first characters, missing last-row content and background mismatch.
- Background-color erase behavior switches after a full visible screen while serializing scrollback. Verify the remembered style source cell, not merely the most recently visited cell.
- The final cursor is restored with vertical move before horizontal move. Cover each direction, no movement, pending wrap position at cols, nonzero baseY and selected ranges that exclude final cursor restoration.
- Restore current input attributes even when they differ from every cell, including empty buffers and explicit ranges. Current attributes use the same SGR diff logic as content.
- Default serializes all available normal-buffer lines. Explicit scrollback clamps scrollback + rows to [0, buffer length]. Verify zero, negative, oversized and values yielding zero rows. JS number validation/conversion must be documented at the browser boundary.
- Explicit inclusive start/end row ranges override scrollback. Marker line values behave like numeric row values. Verify first/middle/last ranges, reversed and out-of-buffer ranges against upstream, with no invented clipping behavior.
- Differential corpus must retain seed, iteration count, full UTF-16 output and input buffer cells/flags/widths. Failures remain saved. Round-trip tests write serialized text into a same-size Rust terminal and compare cells, widths, attrs, wrapping, cursor and buffers.

## HTML, modes and lifecycle, criterion 19

- Before activation, both serialize methods return the upstream activation error. Activation replaces the terminal reference. Dispose follows upstream behavior and must not silently make the addon unusable if upstream does not.
- Normal buffer always serializes first. Include alternate content only when active and not excluded, using exact 1049h and home prefix. Verify inactive alternate data is omitted and excludeAltBuffer does not exclude normal data.
- Serialize modes in source order: application cursor, keypad, bracketed paste, insert, origin, reverse-wrap, focus, wraparound disable, mouse tracking, cursor visibility. Cover default/nondefault of every field. Synchronized-output mode is deliberately not persisted.
- Mouse tracking covers none, x10, vt200, drag and any. Preserve the exact DEC private mode number.
- A nondefault scroll region emits 1-based top/bottom DECSTBM after modes. Default region emits nothing. excludeModes suppresses modes and margins together, including when alternate buffer is active.
- HTML uses the active buffer, independent of ANSI's normal-first policy. Default HTML includes all rows and pads empty cells as spaces. Wide placeholders contribute nothing. Wrapped rows still have separate divs.
- HTML output must match the exact html/body/fragment/pre/div/span wrapper and attribute spelling, including row boundaries and empty rows.
- Default global foreground/background are black on white. includeGlobalBackground switches to configured theme or white on black fallbacks. Font family and font size retain source formatting.
- HTML palette resolution uses active theme palette when present and exact default ANSI colors otherwise. Verify low ANSI, bright ANSI, 6x6x6 cube, grayscale endpoints and RGB components with leading zeros.
- HTML emits complete current span styles on attribute changes, not only changed declarations. Default reset closes the styled span and opens an unstyled span. Equal attributes emit no span boundary.
- Flags map to inverse's source hardcoded colors, font weight, visibility, italic, opacity, and text-decoration in source order. Cover combinations of underline/overline/strike/blink and all underline variants/colors.
- HTML escaping preserves source's whole-cell behavior: a cell equal to ampersand or less-than is escaped; greater-than, quotes and combined strings follow the source exactly. Do not silently replace this with a different escaping policy.
- HTML range overrides onlySelection and scrollback; startCol only applies to its first row and end is terminal cols. Explicit selection uses inclusive start coordinates and exclusive end column on final row. Cover multiline, empty and absent selection.
- HTML scrollback uses the same clamp as ANSI. Verify active alternate buffer, zero rows and out-of-buffer ranges.
- Rust public API and browser/WASM addon binding must call the Rust serializer with live terminal data. A detached fixture-only serializer is partial coverage, not a completed addon. Proposed API gating follows the terminal's buffer access behavior.

## Required evidence

Run every discovered original serialize test and preserve its assertion counts. Port each assertion with a source location mapping; identify pending round-trip tests explicitly. Run deterministic and seeded exact-output differential checks independently of the terminal parser, then real terminal integration and browser/WASM API tests. Record Windows, Linux and macOS runtime evidence separately. Passing serializer fixtures alone does not prove terminal or browser equivalence.
