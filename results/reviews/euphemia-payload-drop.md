# Extended payload replacement review

Reviewer Euphemia. Task 73 was reopened after the audit found that `ExtendedAttrs::set_payload` could destroy a user-owned `Rc<dyn Any>` while its inner RefCell still had a mutable borrow.

## Source inspection

The current `crates/xterm-core/src/buffer/extended.rs` setter moves the previous payload out with `std::mem::replace`, finishes that statement, and then explicitly drops the previous owner. Rust drops the temporary RefMut at the statement boundary. The replacement is therefore visible before the old destructor runs, and no mutable attribute borrow crosses that destructor.

The added `cell_extended::native_payload_drop_observes_replacement_and_reenters_without_borrowing_conflict` regression has the required shape. The old payload holds a Weak reference to a separate RefCell containing a shared ExtendedAttrs handle. That outer RefCell is not borrowed by the original setter. Drop reads the newly installed payload value 2, changes the URL id above u32 range and replaces the payload with 3. The final assertions require both mutations to remain visible. This distinguishes the actual inner borrow bug from an invalid test that reborrows an already borrowed outer container, and avoids a strong reference cycle.

The original setter would retain its inner mutable borrow during old-payload destruction and fail when Drop calls `attrs.payload()`. The fix does not catch or suppress that error. It changes the ownership ordering so the reentrant operation can execute.

## Execution gate

After the approved target-cache relocation, I independently ran `cargo test -p xterm-core --test cell_extended` through the controlled native build slot. All five tests passed, including the new destructor reentry regression and the 4,096-row extended mutation fixture. No test was skipped. Exact stdout, command/process result and both filesystem headroom observations are retained in `results/build-slots/1793172336064-25128-75a37738-a122-42ae-ac6c-2c922f13eb60/`. The targeted fix now has independent source inspection and runtime evidence. This does not claim a fresh full workspace/Clippy/release matrix or browser execution.
