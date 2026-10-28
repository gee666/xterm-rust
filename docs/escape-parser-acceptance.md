# Escape parser acceptance

This checklist covers `source_repo/src/common/parser/EscapeSequenceParser.ts` and its original test file. It does not claim complete terminal equivalence. Transition table data is task 53. Params storage is task 8. OSC, DCS and APC streaming dispatch are tasks 15, 18 and 17. Task 26 must integrate all of them.

## Transitions and sequence actions

- Match all 17 states and 18 actions. Compare every one of the 4257 packed transition entries with the unchanged upstream initializer. Check printable ASCII, DEL, executable C0 bytes, C1 controls, U+00A0 and supplementary/lone-surrogate UTF-32 values.
- Check anywhere cancellation by CAN/SUB, ESC clearing, C1 CSI/DCS/OSC/APC entry, ST termination, SOS/PM ignoring and malformed non-ASCII entry recovery.
- Cover every state/action test in the original suite, including clear, collect, parameter, ignore, error, hook/start, payload put and unhook/end. Initial state, reset, custom table constructors and error-abort state retention require assertions.
- CSI/DCS parameters preserve zero defaults, empty parameters, colon subparameters with -1 defaults, overflow clamping, capacity rejection, prefixes and intermediates. Compare both complete and split sequences, including one codepoint per call.
- Preserve the original executable and CSI fast paths even when they bypass custom tables. Compare contiguous PRINT/OSC_PUT/DCS_PUT/APC_PUT callback boundaries. Include embedded DEL differences and C0 payload permissions. ESC-ST must be swallowed by the default ESC handler.
- Check original example assertions for CSI text/executable mixtures, 7-bit and 8-bit strings, partial strings, Unicode, colon notation, CAN/SUB abort and invalid-sequence recovery.

## Registration and callback contracts

- Validate prefix range 0x3c..0x3f and at most one byte, intermediates range 0x20..0x2f and at most two bytes, and one final byte. CSI/DCS final range is 0x40..0x7e. ESC/APC final range is 0x30..0x7e. APC ignores prefixes. Test boundary and rejected strings, plus packed identifier and diagnostic string round trips.
- PRINT and executable set/clear/fallback behavior, executable optional-argument difference between fast and table paths, and error handler set/clear/abort must match upstream.
- ESC/CSI registrations run newest first. True stops bubbling, false continues, and only an exhausted synchronous list calls fallback. Registration tokens explicitly dispose once and tolerate repeated disposal. Clear removes future lookup, without changing an already retained list. Dropping a token must not unregister. Duplicate callback identities and callback-driven list mutation require tests.
- Outer DCS/OSC/APC registration, clear and fallback forwarding must use validated identifiers and their actual Rust subparsers. Reset must abort active payload handlers and retain asynchronous cleanup for an adapter executor. Dispose must clear registrations without calling active payload end handlers.
- Preceding grapheme join state survives PRINT, PARAM, COLLECT, CLEAR, IGNORE and starts; valid execute/dispatch and finished string commands clear it. Print callbacks must be able to update it safely.

## Asynchronous continuation and evidence

- Sync callbacks remain synchronous. Pending futures pause before later bytes or lower-priority callbacks execute. Await and resume with the same chunk and boolean result for ESC, CSI, OSC, DCS and APC. Check several pending callbacks, mixed sync/async ordering, true suppression and false fallthrough, and string abort cleanup.
- No continuation result while paused is a typed error, and subsequent calls remain failed until reset. Reset while paused resumes at the next codepoint in the same chunk from the initial state. Check repeated pending and reset while cleanup remains outstanding.
- Preserve upstream's resumed ESC/CSI behavior: asynchronous false exhaustion does not invoke fallback. Any departure must be reported rather than silently corrected.
- Run original tests unchanged with exact discovered/passed/failed/pending counts. Map each original test to Rust assertions or explicitly record an unported gap. Maintain deterministic differential fixtures, seed and chunk boundaries, full callback/error/state/collect/parameter comparisons and retained failing rows.
- Independent review must inspect Rust code, execute focused checks and examine original-to-Rust coverage. Run formatting, warning-free focused Clippy and workspace integration tests. Record exact command outcomes and tool versions, not inferred success.
