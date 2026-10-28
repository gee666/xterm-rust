Reviewed the reset/ungated corrections in `serialize/addon.rs` and `serialize/view.rs` against the public terminal and buffer namespace source. The implementation now uses the actual cached public view and does not invent a proposed-API buffer gate.

Ran `cargo test -p xterm-core --test serialize_live --test serialize_live_callbacks --test serialize_live_reentrancy --test serialize_header_order --test serialize_ownership --test serialize_live_reset`: 14 passed. Accepted native task 88. Browser binding and browser execution remain task 131.
