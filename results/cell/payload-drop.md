Changed `ExtendedAttrs::set_payload` to publish the replacement, release its RefCell borrow, then drop the old Rc. The regression reenters through the old payload's destructor and confirms the inner replacement is retained.

Ran the five focused cell targets: 33 passed. Focused Clippy with `-D warnings` passed. Euphemia independently ran `cell_extended`: 5 passed. Current release verification remains for CI.
