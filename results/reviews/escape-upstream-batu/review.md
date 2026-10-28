# Escape parser assertion migration review

Batu independently inspected the Rust fixture, state/parameter/string/registration/basic-handler/identifier modules, original reset assertions, source-derived example generator and pinned TypeScript case mapper. Reran the source example check and mapping: all 21 examples, 23 steps and 69 literal events match; 161 discovered cases map to 147 synchronous cases and 14 explicitly separate async cases. These are mappings, not runtime assertion counts.

Independent native Windows debug and release runs passed all 43 Rust tests. Current Clippy is blocked by unrelated active decoration_cache.rs dead-code changes, retained in clippy.log. This review does not claim a warning-free workspace or Linux/macOS/browser proof.

One finding was raised. The original reset test sets OSC payload '#' and asserts it is cleared. The first Rust version omitted a corresponding payload-reset assertion. Ashley added a real active OSC payload before reset, confirms direct old OSC end produces no event after reset, and confirms a new OSC message has no stale '#'. The updated construction test was inspected and the 43-case debug rerun passed. The explicit default transition-table constructor assertion was also retained.

Representation mappings for typed parser states, real OSC instead of a permissive mock and typed DCS/APC labels are documented. No source assertions or source modules were changed. No unresolved migration finding remains within task90's synchronous scope. Async continuation and full terminal integration remain separate tasks.
