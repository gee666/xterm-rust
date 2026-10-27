# Kitty keyboard policy

`crates/xterm-core/src/input/kitty_keyboard.rs` ports the pure encoding policy in `source_repo/src/common/input/KittyKeyboard.ts`. It uses the same UTF-16 event and result types as the legacy keyboard evaluator. It has no DOM, operating-system or asynchronous dependencies. Exact acceptance paths are in [kitty-acceptance.md](kitty-acceptance.md), criteria #85 through #89.

## API mapping

| Upstream | Rust |
| --- | --- |
| `KittyKeyboard.evaluate` | `kitty_keyboard::evaluate` |
| `KittyKeyboard.shouldUseProtocol` | `kitty_keyboard::should_use_protocol` |
| `KittyKeyboardFlags` | `kitty_keyboard::flags` constants |
| `KittyKeyboardEventType` | `kitty_keyboard::EventType` |
| `IKeyboardEvent` | shared `keyboard::KeyboardEvent<'a>` |
| result key, cancel and type | shared `keyboard::KeyboardResult` |

The Rust module exposes functions rather than an instance because upstream has no mutable state. Key maps are immutable data. `EventType` has Press, Repeat and Release with the original numeric values 1, 2 and 3. The adapter supplies the appropriate event type and Mac Option setting. DOM keyCode and event type strings are not read by this policy, but keyCode remains in the shared event type.

Flags are a `u32` bitfield, including unknown bits. A browser binding must perform JavaScript bitwise conversion before calling `evaluate`. `should_use_protocol` accepts that typed bitfield, not arbitrary JS numbers such as NaN, fractions or negative numbers. Bindings must evaluate the upstream positive-number check before converting an arbitrary JS API argument; conversion must not turn a negative flag into protocol activation.

The policy deliberately preserves upstream behavior rather than replacing it with a newer Kitty specification. Enter/Tab/Backspace can remain legacy bytes even with event reporting. Associated text can suppress the repeat suffix. F26-F35 remain unmapped because the source table stops at F25 despite its broader comment.

## UTF-16 and JavaScript details

Key and code are borrowed UTF-16 slices, and the result is `Option<Vec<u16>>`. A surrogate pair is not a one-unit key. A lone surrogate remains a unit, not a replacement character. Empty output is distinct from no output.

Physical Key recovery uses the shared Unicode 17.0 lowercase table from the legacy keyboard implementation. For Keyİ, the source lowercases to two units but reads only the first; Kitty encodes i, unlike the legacy dead-key branch that sends both units. The table's Unicode attribution and generation procedure are documented in [keyboard.md](keyboard.md).

The Numpad branch retains the source's lexical suffix comparison and radix-10 parseInt behavior, including synthetic codes such as Numpad123 and Numpad8suffix. Key numbers use JavaScript-compatible floating-point formatting from the workspace's existing `ryu-js` dependency. This is needed for very long synthetic decimal suffixes; it does not add a dependency.

Source lookup tables are ordinary JavaScript objects. Their inherited Object.prototype names can therefore produce unexpected sequences for synthetic key names such as toString. `kitty_keyboard/inherited.rs` preserves the observed Node 24/V8 native-function strings, and __proto__ produces [object Object]. This text is engine-dependent under JavaScript's native-function string contract. Firefox/WebKit and prototype-mutated JavaScript realms remain separate browser verification obligations, not proven by this Node oracle.

## Verification

With Rust tools on PATH and Node.js supporting `stripTypeScriptTypes`:

```sh
node scripts/verify-kitty-keyboard.mjs
cargo test -p xterm-core --test kitty_upstream --test kitty_edges
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace --all-targets --all-features
cargo test --workspace --doc
```

Victor independently owns the JS oracle and reviewed source fixtures in task #54. The script executes unchanged original test bodies after TypeScript erasure, captures every event and compares the full action, cancellation and UTF-16 payload against the compiled Rust example. Ordinary runs fail if a fixture drifts. Fixture regeneration requires `--generate` and review.

Observed Windows x86-64 evidence currently includes 165 original tests and 173 original assertions, all passed. There are 169 evaluation rows and four protocol-enablement rows, both counts pinned in Rust. The expanded independent matrix passed 508,249 full-result comparisons with seed 1263096916, hexadecimal 0x4b495454. This includes the inherited-name and missing-function-key matrix plus 5,000 seeded long decimal suffixes. Seven Rust tests passed, including the fixture replay tests and five additional regression families. See `results/kitty-keyboard/verification.json` for exact categories, source hashes, platform and tool versions.

`results/kitty/` retains this task's command logs, including failed workspace checks caused by other active modules. A successful targeted run does not mean a workspace gate passed. Temporary build products remain under `target/` or `tmp/`.

Terminal flag negotiation, actual DOM keydown/keyup dispatch, legacy/Win32 mode selection, cancellation, duplicate-event prevention and browser listener disposal belong to integration tasks #28 and #33. Native Linux/macOS and Chromium/Firefox/WebKit runtime evidence are not established by this policy test. Full terminal equivalence remains NOT PROVEN.
