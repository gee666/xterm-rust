# Font addon acceptance

Owner Fergus, task 40. Source paths refer to the supplied xterm.js 6.0.0 tree. This checklist defines required behavior, not current implementation proof. Each missing test or integration remains unrun and equivalence is NOT PROVEN.

## CSS family parser

Reference `source_repo/addons/addon-ligatures/src/parse.ts` and its 16 upstream tests. Preserve the actual algorithm, including its unusual behavior for a final quoted family without a comma. Keep UTF-16 code units, lone surrogates and supplementary code points intact. Cover single/double quotes, comma separators, empty input/families, whitespace collapsing in identifiers versus quoted strings, escaped quotes/backslashes/commas/newlines, dangling escape, unterminated strings, one through six hex digits, six-digit termination, exactly one swallowed whitespace after hex, out-of-range Unicode errors, and JavaScript whitespace including BOM and NBSP. A native Rust signature accepts UTF-16 slices, so non-string input cannot occur; browser bindings must reject non-string input rather than coerce it.

## Web-font selection and loading

Reference `source_repo/addons/addon-web-fonts/src/WebFontsAddon.ts` and `test/WebFontsAddon.test.ts`. `loadFonts` waits on document.fonts.ready before enumerating faces. Missing or empty arguments load every registered face. Family strings match unquoted family exactly, including case. Unknown family rejects with the exact source message. FontFace identity for deduplication uses only unquoted family, stretch, style, unicodeRange and weight. A new face is added immediately, before later request validation, and becomes available to subsequent request items. Equal descriptors select the existing first face even when source bytes differ. Preserve request order and duplicate loads in Promise.all. Propagate ready/load failures. Font selection does not imply successful loading.

## Web-font relayout and lifecycle

Construction defaults initialRelayout to true. Activate schedules relayout after ready only when enabled. Relayout with no terminal resolves without waiting. Split terminal fontFamily by commas exactly as source does, unquote only balanced matching quotes, and use source CSS quote/escape rules when rebuilding. Partition web families from clean families in original order. If none are web families, do not mutate options. Otherwise wait for all dirty faces to load; set clean fallback or monospace, then restore the original family exactly. Disposal clears the terminal and prevents post-await writes. Load failure prevents both assignments. Test dispose before ready, during loading, repeated dispose, empty fallback, multiple faces per family, duplicate families, and calls after dispose. Validate real browser FontFaceSet and renderer size changes, not just a native plan.

## Ligature ranges, shaping and cache

References `addon-ligatures/src/index.ts`, `LigaturesAddon.ts`, `font.ts`, and all `fontLigatures/` modules/tests. Default fallback list is the entire source list, sorted stably by descending UTF-16 length; an explicit empty list disables fallback. Match greedily at UTF-16 offsets and skip each matched range. Font resolution follows parsed families and stops at a generic family. Verify legacy navigator.fonts, permission denial/TypeError, modern queryLocalFonts, query failure, family first-face/blob selection, absent font and loading error. Font changes reset old shape data; stale async completion must not replace the current family. A found font refreshes every terminal row; no match does not refresh. Preserve GSUB lookup types 6 formats 1/2/3 and 8 format 1, coverage/class definitions, substitutions, contextual lookahead/backtrack, merging/flattening, and font cache limits/eviction. Compare all four supplied font fixtures with the original, retain seeded failing inputs, and test range clones cannot mutate cached entries.

## Ligature browser lifecycle

Activation before open rejects with the exact source error. Register one character joiner and set fontFeatureSettings to the supplied value or `\"calt\" on`. Loading and failed fonts use fallback. Dispose deregisters the stored joiner once and clears style, including repeated disposal and disposed terminal. Test custom/default settings, asynchronous family switches and actual rendered ligatures through the Rust-backed browser terminal in Chromium, Firefox and WebKit where supported.

## Evidence gates

Port every upstream assertion without edits to the reference. Record discovered/mapped/executed/passed/failed/skipped/unrun counts per file, fixture hashes, random seeds, commands/toolchain versions and native/browser platforms. Run fmt, warning-free Clippy, Rust tests and wasm32 build. Browser tests must invoke Rust/WASM, not the original addon. Source baseline results and Rust differential evidence remain distinct. An independent reviewer must inspect implementation and execute these checks before task acceptance.
