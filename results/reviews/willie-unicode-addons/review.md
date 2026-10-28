# Task 41 review

Found and fixed old-service Drop reentry under the addon borrow; permanent regression is `tests/unicode_addon_drop.rs`.

Source oracle passed. `cargo test -p xterm-core --test unicode_addons --test unicode_addon_manager --test unicode_v6 --test unicode_addon_drop`: 16 passed. Scoped Clippy with `-D warnings` and formatting passed.

Accepted native provider/lifecycle work. Browser bindings remain task213/criterion130; no full equivalence claim.
