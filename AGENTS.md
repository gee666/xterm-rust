# Rust port instructions

## Code quality bar

- No dead code. Delete unused, unreachable, stubbed, or half-wired implementations. Do not hide incomplete behavior behind successful no-ops, placeholder returns, ignored tests, or blanket lint suppression.
- Keep the architecture clear. Separate terminal state, parsing, buffers, public APIs, platform integration, rendering, and addons. The core must not depend on a browser, a particular operating system, or a demo application.
- Avoid senseless duplication. Share behavior rather than copying it to change a few lines. Explain intentional duplication briefly.
- Follow established repository patterns. Explain a new dependency or architectural approach before adopting it. Prefer maintained, portable dependencies with compatible licenses and a clear need.
- Comments explain why, not what the signature says. Remove stale comments and generated filler. Document public contracts, compatibility decisions, and non-obvious invariants in plain language.
- No crutches. Use named constants for protocol values and configuration for user choices. Do not use sleeps, unbounded retries, swallowed errors, or catch-all fallbacks to disguise bugs. Protocol constants and Unicode tables are data, not configurable policy.
- Use meaningful Rust types. Model state and errors with enums and structs; validate boundaries; propagate recoverable failures with Result and represent absence with Option. Avoid unchecked conversions, gratuitous clones, unnecessary allocations, and panic-based handling of untrusted terminal input. Justify any unwrap or expect in production with an invariant. Assertions in tests are appropriate.
- Keep builds and lint warning-free. Run cargo fmt --all -- --check, cargo clippy --workspace --all-targets --all-features -- -D warnings, and cargo test --workspace --all-targets --all-features. Test supported feature combinations and target-specific code separately when an all-features build cannot represent them. Do not silence warnings instead of fixing causes.
- Keep files at most 350 lines and functions focused. A genuinely cohesive module, generated table, or upstream fixture may exceed the limit only with a documented reason. Do not split files mechanically to conceal complexity.
- Choose the simplest design that preserves reliability and source behavior. Do not introduce a framework or abstraction without a concrete need.

## Rust safety and portability

- Prefer safe Rust. Unsafe is allowed only when a required FFI boundary or a measured, otherwise unattainable requirement justifies it. Isolate each unsafe operation in the smallest possible private module and require explicit unsafe blocks even inside unsafe functions with unsafe_op_in_unsafe_fn denied.
- Every unsafe block needs an adjacent SAFETY comment stating the actual invariants and why they hold. Unsafe public functions need a rustdoc Safety section. Document why a safe alternative is insufficient, who owns memory and resources, lifetimes, aliasing, bounds, alignment, threading, and ABI obligations as relevant. Independent review and focused tests are required. Run Miri or applicable sanitizers where supported; record unsupported checks rather than claiming they passed.
- Use ownership, borrowing, and RAII for cleanup. Avoid global mutable state, leaked allocations, reference cycles, and locks held across callbacks or await points. Define callback ordering, cancellation, disposal, and reentrancy behavior to match upstream.
- Preserve JavaScript semantics deliberately. Account for UTF-16 indexing, byte-stream chunk boundaries, number ranges and conversions, Unicode widths, event ordering, option defaults, errors, and lifecycle behavior. Idiomatic Rust naming must not silently alter the contract.
- Support Windows, Linux, and macOS without hardcoded paths, shell assumptions, platform newline changes, or locale-dependent behavior. Use cfg-gated adapters only where necessary and keep target-specific dependencies out of the shared core. Declare the Rust edition, minimum supported Rust version, and feature policy.
- Use reproducible dependency resolution and record toolchain versions. Preserve upstream license notices and attribution, including notices for copied code, fixtures, and generated Unicode data.

## Porting and verification

- Treat source_repo as the read-only behavioral reference. Inventory the browser, headless, common, typings, and every addon before declaring coverage. Rust must implement the functionality; a wrapper that delegates the terminal engine back to xterm.js is not a Rust port.
- Map every upstream API, option, event, addon, and test suite to its Rust implementation and evidence. Browser-only contracts require a working browser/WASM integration, not a nominal headless substitute. Document binding names and any language-specific representation changes.
- Keep source tests and assertions intact as the reference. Port assertions faithfully, add integration and end-to-end tests, and compare Rust and JavaScript outputs on shared deterministic and randomized inputs. Record seeds and retain failing cases. Never weaken assertions or omit a failing feature to obtain a green build.
- A passing smoke test is not proof of equivalence. Full equivalence requires complete coverage mapping, passing differential and applicable upstream suites, end-to-end checks, and native Windows, Linux, and macOS evidence. Cross-compilation is not a native runtime test. Mark missing, skipped, unrun, or blocked checks explicitly and do not claim completion while gaps remain.
- Provide documented commands that let a user run Rust tests and a working interactive example. Compare original and Rust speed, build/distribution size, and lines of code with reproducible scripts, the same workloads, release settings, raw results, and clear distinctions between JavaScript bundles, WASM bundles, and native binaries. Do not invent results or promise a speedup.
- All bash commands must have a timeout. Put temporary files under the repository-root tmp/ directory. Never commit local credentials or private swarm runtime files. Do not edit this file again without explicit user permission.
