# Private mode captures across reset

Task 198 owns `core_service/mode_capture.rs` and the narrow capture/reset wiring in CoreService. Sophie owns the actual DECRQM 9001 query integration and source differential tests in task 102. Source references are `source_repo/src/common/services/CoreService.ts` and the DECRQM handler in `source_repo/src/common/InputHandler.ts`.

The source captures the DEC mode object before reading an observable vtExtensions option. That getter can change the existing mode object, or reset CoreService and replace it. These are different operations. A snapshot copied before the getter loses in-place changes; reading the current object after the getter loses the old object after reset. Holding a Rust RefCell borrow across that getter would instead panic if it reset or changed the modes.

`capture_private_modes()` returns a read-only `PrivateModesCapture` without retaining a RefCell borrow. Its `read()` clones the current private modes until the first reset after capture. Reset freezes the old value for all active captures before replacing the current modes. Later resets cannot overwrite a frozen capture. A capture taken after reset observes the new object and has its own replacement boundary.

This deliberately models a captured read-only object reference, not an arbitrary mutable JS object API. Native current-mode writes still use the established `dec_private_modes.borrow_mut()` contract. The guard borrows the core allocation for its Rust lifetime, but does not block its shared reset method. No unsafe code, host dependencies or callback-held borrows are involved.

The core retains only Weak references to active capture states. Capture removes dead entries before registering another guard. Reset takes and clears the live list, leaving each surviving guard with its own frozen value. Repeated queries without reset cannot accumulate dead guards. No mode-state clone occurs at capture time; reset only clones for outstanding captures. This extra storage is required by observable source replacement identity, not a second terminal state implementation.

Three private native tests cover:

- In-place updates after capture, including cursor overrides and wraparound, then reset and further current-object mutation.
- Nested captures spanning distinct resets, preserving each old state independently across later resets.
- One thousand scoped captures, weak-list pruning, mutation without an outstanding RefCell borrow, and list clearing at reset.

Observed native Windows command in the coordinated build slot:

```sh
cargo test -p xterm-core --lib core_service::mode_capture
cargo clippy -p xterm-core --lib -- -D warnings
```

Task 102 separately compares the real query reply when the vtExtensions getter resets or mutates mode state. Neither these three unit tests nor the guard API alone prove that integration. The three private tests passed with zero failures and 56 unrelated tests filtered out. Exact command/process/resource evidence is `results/build-slots/1793171600828-49096-3bfad8f6-c060-441c-9c16-8188ad8f8f9d/receipt.json` and `command.log`. Sophie separately observed all eight real query/getter callback cases and the fresh unmodified JS oracle, with reset/in-place/resize distinctions, in `results/terminal-modes/callback-reference.json`. No browser or full-equivalence result is inferred from those native/source checks.
