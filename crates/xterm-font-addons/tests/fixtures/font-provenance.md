# Font fixture provenance

`kongtext.regular.ttf` is an unchanged copy of the supplied `source_repo/demo/fonts/kongtext.regular.ttf`. SHA-256 is `a7d2b91d8ba1cdb527d5e856339d1cd85fd3c5b900816c3ee713a45e7b9b1eed`.

The font is by codeman38, zone38.net. Its original distribution permits bundling with software and forbids resale as a commercial font collection. The complete supplied license text is retained in `licenses/font-fixtures.txt`. That upstream notice also mentions BPdots, which is not bundled here.

This fixture tests real browser FontFace loading and failure/relayout behavior. Keeping the binary under tests makes the Rust/WASM tests buildable from a clean checkout without an ignored `source_repo` tree. Original-suite oracle generation still needs the source reference.
