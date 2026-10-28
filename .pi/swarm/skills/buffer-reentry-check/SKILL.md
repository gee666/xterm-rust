---
name: buffer-reentry-check
description: Use when verifying source-legal Buffer resize observers, nested resizes, marker/list replacement, raw option getter reentry and callback error identity.
---

# Buffer resize callback checks

Reuse the source esbuild/Mocha approach from buffer-check and buffer-service-oracle. Do not edit source_repo or replace callback failures by successful no-ops.

Run `node scripts/buffer-resize-reentry-oracle.mjs --generate` only when reviewing source fixture changes. Normal verification omits `--generate` and checks deterministic fixture equality. The driver saves source hashes and case names to results/buffer-reentry/oracle.json. Temporary reference bundles stay under tmp/buffer-resize-reentry.

The ten source scenarios cover trim/larger/smaller observers reading and mutating the actual Buffer, nested resizing, scrollback numeric conversion and Windows getter reentry, thrown observer partial state, and replacement of the entire line list during widening. Every checkpoint includes full packed cells, UTF-16 strings, extended attributes, row identities, cursor/base/display/saved/region/dimension state and sparse undefined rows.

Run `cargo test -p xterm-core --test buffer_resize_reentry` and the ordinary `buffer_resize_differential` suite. Both owned resize and shared resize must retain all source and seeded ordinary transition assertions. Also run scoped Clippy and rustfmt, and record debug/release execution separately. Use the shared target directory and honor the disk reserve/build pause.

The public service must call Buffer::resize_shared without holding a Buffer RefMut. Preparing smaller reflow can hold a short state borrow because it emits no application callbacks; its layout events run afterward through the current shared line list. Widening layout deletion and capacity trim callbacks also execute outside state borrows. Host error objects must survive through Error::source and EmitError::Callback.

No claim about browser/native end-to-end coverage follows from these component tests. Application host scheduling and parser/addon integration remain task99 evidence.
