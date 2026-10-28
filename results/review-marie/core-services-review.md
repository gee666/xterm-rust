# Independent review of task 29

Reviewer Marie read both Rust modules, CoreService.ts, LogService.ts, all core/log tests, the original-source oracle, docs/core-service-acceptance.md and docs/core-services.md. Code reviewed includes the frozen efd34c5 source plus current Event enum consumer update. The review found a real boundary gap: JavaScript showCursorImmediately uses nullish fallback and preserves raw nonboolean identity, whereas the initial Rust constructor coerced it to bool. Willie fixed this with CursorInitialization and added a raw identity test before acceptance.

Fresh final commands run locally on native Windows x64, rustc/cargo 1.85.1 and Node24.15.0:

- `cargo test -p xterm-core --test core_service --test log_service`: exit0,11core+9log tests,0failed/ignored.
- `node scripts/check-core-services.mjs`: exit0,3unchanged original cases/4assertions plus20616 differential operations,20000randomized,seed1129271877,0mismatches.
- `cargo clippy -p xterm-core --test core_service --test log_service --example core_service_oracle -- -D warnings`: exit0.
- Scoped rustfmt for both modules, both test files and example: exit0.

Logs are core-services-tests.log, core-services-oracle.log, core-services-clippy.log and core-services-fmt.log in this directory. The failed-command fmt log records my initial nonexistent path mistake and is not an implementation defect. Earlier attempts hit concurrent engine/Event/Buffer API migrations; only the final successful rerun supports acceptance. Source/implementation hashes and operation digest are in results/core-services/summary.json and the captured oracle log.

No unsafe code or new dependency in these modules. Callback paths release borrows before sinks/listeners, preserve UTF16, and propagate actual callback errors. Actual terminal/browser integration is task68, not proved by this service acceptance. No claim of full xterm equivalence, workspace-wide checks or Linux/macOS/browser runtime evidence is made.
