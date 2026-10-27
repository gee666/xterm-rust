# Kitty keyboard acceptance checklist

This checklist covers `source_repo/src/common/input/KittyKeyboard.ts` and all assertions in `KittyKeyboard.test.ts`. It is not a claim that the terminal's Kitty protocol negotiation or browser event wiring works. Those integration checks belong to browser input task #33 and terminal task #28. Policy task #51 and independent verification task #54 must both pass before the portable policy is accepted.

## Flag routing and suppression

For every combination of defined enhancement flags 0 through 31, modifiers 0 through 15, and Press, Repeat, Release:

- No REPORT_EVENT_TYPES means release returns SendKey, cancel=false and key=None before any mapping.
- Shift, Control, Alt and Meta key names produce no output without REPORT_ALL_KEYS_AS_ESCAPE_CODES, even when event reporting is enabled.
- CapsLock, NumLock and ScrollLock follow the same suppression rule as modifiers.
- Enter, Tab and Backspace release produce no output unless REPORT_ALL_KEYS_AS_ESCAPE_CODES is set.
- REPORT_EVENT_TYPES implies disambiguation for press and repeat. The same flag must not swallow modified keys.
- REPORT_ALL_KEYS_AS_ESCAPE_CODES forces CSI-u for eligible text keys, but arrow, navigation and F1-F12 legacy encodings still run first.
- Unknown flag bits do not change a result except where they overlap defined bits. `should_use_protocol(0)` is false; any nonzero bitfield is true.
- Every returned result has SendKey action. Cancel is true only for an encoded legacy functional sequence or CSI-u, not for legacy text/control bytes or suppressed input.

## Key lookup and encoding routes

- ArrowUp/Down/Right/Left map A/B/C/D. Home and End map H/F. Unmodified press uses CSI letter with no numeric parameter.
- F1-F4 map P/Q/R/S and use SS3 only with no modifiers and no required event suffix. All other variants use CSI 1;modifier letter.
- Insert/Delete/PageUp/PageDown/F5-F12 use exact tilde parameters 2,3,5,6,15,17,18,19,20,21,23,24.
- Escape/Enter/Tab/Backspace map 27/13/9/127. All six lock/system mappings, all F13-F25 entries actually present upstream, every KP_* alias, all eight left/right modifiers and seven media keys must agree with source.
- The upstream comment says F13-F35, but F26-F35 have no implementation entries. They must remain unmapped instead of inventing support.
- Physical Numpad mappings take priority over physical modifier mappings, named functional keys and printable text. Physical Shift/Control/Alt/Meta mappings take priority over key names.
- Numpad lexical suffix comparison and radix-10 parseInt behavior are preserved. Include Numpad123, Numpad8suffix, Numpad0x10, Numpad9suffix, empty suffix, trailing non-digits, leading zeroes and long decimal overflow/rounding cases.
- Plain-object inherited names are tested because source does not use own-property lookup. Record the oracle engine and native-function text rather than silently dropping this observable behavior.

## CSI-u fields

- Modifier encoding is 1 + Shift/Alt/Ctrl/Super bits 1/2/4/8, with 0 meaning no modifier field. Include each individual bit and all combinations.
- REPORT_ALTERNATE_KEYS adds the original single UTF-16 key unit only for shifted nonfunctional, nonmodifier keys. Functional/numpad keys never gain alternate or associated text fields.
- REPORT_ASSOCIATED_TEXT adds text only for a single UTF-16 unit, nonrelease, nonfunctional, nonmodifier, non-Control event. With no modifiers, retain the empty modifier field before text.
- A reported repeat gets :2 unless associated text is present. A reported release gets :3 and never associated text. Press never gets an event suffix.
- Empty and omitted fields differ. Compare the entire UTF-16 payload, not a normalized or parsed sequence.
- Legacy Enter/Tab/Backspace bytes and ordinary unmodified text preserve cancel=false. Alt/Ctrl/Meta prevent ordinary text fallback. Shift alone can return shifted text when disambiguation does not require CSI-u.

## UTF-16, physical keys and Mac Option

- `.key.length` means UTF-16 units, not Unicode scalar count. A surrogate pair is not a one-unit printable key. Lone high/low surrogates remain valid units and must not become U+FFFD.
- Plain A-Z key names use lowercase ASCII codepoints. Other key units remain unchanged without physical-key recovery.
- Physical Digit recovery requires exactly six UTF-16 units, prefix Digit and one ASCII digit. Physical Key recovery requires exactly four units and prefix Key.
- Shift enables physical-key recovery. Mac Option enables it only when both mac_option_as_alt and alt_key are true. Linux Alt and Mac Option disabled preserve the composed key instead.
- Include Keyİ whose JS lowercase expands to i plus dot; only charCodeAt(0) is encoded. Test all 65,536 BMP units through physical Key recovery and direct text. Keep lowercase tied to the documented Unicode 17.0 oracle rather than rustc Unicode version.
- Test dead keys, empty keys, unidentified names, non-BMP text, malformed physical codes and arbitrary unknown flag bits. No panics on terminal/browser-provided UTF-16 data.

## Required policy evidence

1. `node scripts/verify-kitty-keyboard.mjs` executes every unchanged upstream test body and assertion. Its fixtures must not drift without an explicit reviewed regeneration.
2. `cargo test -p xterm-core --test kitty_upstream --test kitty_edges` checks every captured full result and the five additional regression families. Assert exact captured-row counts so fixture removal cannot silently pass.
3. The independent verifier compares full SendKey/cancel/UTF-16 output for the complete named-key/modifier/flag/event matrix, physical-code/Mac matrix, three BMP passes and 25,000 seeded synthetic events. Record exact counts and seed in `results/kitty-keyboard/verification.json`; retain failing cases.
4. Run workspace fmt, Clippy with -D warnings, all-target/all-feature tests and doctests. Preserve commands, exit statuses and logs. Shared-tree failures belong to their module owners and are not a passing gate.
5. Independent review inspects source routing, every table entry, formatter fields and the fixture/oracle mechanism, then reruns targeted tests and differential checks. No unsafe code is needed for this policy.

## Required integration evidence, still separate

- Terminal CSI protocol query/set/push/pop/reset sequences update the active flags exactly, including nested state, alternate buffer and reset behavior where upstream defines them.
- Actual Rust-backed browser keydown/keyup chooses Press/Repeat/Release and mac_option_as_alt correctly, calls this policy only when Kitty is active, emits its exact key bytes and honors cancellation without duplicate input events.
- Install/dispose/reopen must not retain stale key handlers. Legacy and Win32 mode precedence must match CoreBrowserTerminal.
- Run actual Chromium, Firefox and WebKit input cases, not synthetic calls to this pure policy. Capture browser engine differences for inherited native-function names if exposed there.
- Native Windows/Linux/macOS execution and Rust-backed browser wiring remain unverified until observed. Portable-policy success cannot satisfy these integration requirements.
