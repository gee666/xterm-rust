# Win32 prototype reference cases

Run `node scripts/win32-prototype-reference.mjs --generate-reference --browsers`.

This executes unchanged Win32InputMode source, not Rust. It captured 212 cases in Node24.15.0, Chromium148.0.7778.96, Firefox150.0.2 and WebKit26.4. Files are under `results/win32-prototype-reference/`. The Rust comparison is unrun and remains task185 after implementation184.

Cases cover inherited native functions and __proto__, undefined/falsy/primitive/string values, lossless UTF-16, cached raw objects whose coercion changes after later field reads, String-hint conversion, exact thrown markers, Symbol/nonprimitive conversion errors, actual table receiver identity, own-entry mutation and separate encoder instances.

The generator temporarily modifies only an isolated reference realm's prototype and restores its descriptors in finally. Browser instances close in finally. Nothing delegates a Rust engine to JavaScript, and no reference capture is reported as a Rust result.
