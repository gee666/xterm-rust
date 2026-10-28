# SGR acceptance contract

Task #80 ports InputHandler.charAttributes and its four helpers from source_repo/src/common/InputHandler.ts. Engine task #28 must call it from CSI m, supply the live kitty extension option and forward unknown-parameter diagnostics. Passing an attribute-only test does not prove live parser or browser integration.

## SGR1. Ordered packed mutations

Test every branch and combinations in source order: 0,1,2,3,4,5,7,8,9,21,22,23,24,25,27,28,29,30-37,39,40-47,49,53,55,58,59,90-97,100-107,221,222. Unknown parameters, including unsupported blink6, must not change attributes. The actual caller must invoke each diagnostic before later attribute mutations. A logger failure retains only the already-applied prefix; reentrant mutations and replacement of the captured attribute object must preserve source target identity. The kitty option getter runs only on221/222 and may fail or reenter before those mutations.

Single SGR0 and embedded SGR0 reset foreground/background to defaults. They clone extended attributes and clear underline style/color while retaining URL id and underline variant bits. Clone drops payload, matching ExtendedAttrs.clone. Other flag operations must not mutate or clone unrelated extended state. Normal color changes retain every non-color bit, including protection; SGR0 resets protection as source does.

Underline4 without subparams selects single. Styles0-5 map exactly; -1 and values above5 select single. 21 selects double;24 selects none. All style paths clone extended state, update FG underline and BG HAS_EXTENDED, and preserve source URL-driven effective style behavior. 59 clones extended state, writes the source masked -1 underline-color bits, then updates HAS_EXTENDED.

221 resets only bold and222 resets only faint when kittySgrBoldFaintControl is enabled or absent. Explicit false leaves those flags unchanged and reports both as unknown diagnostics. Tests must retain both option states.

## SGR2. Extended color normalization

Exercise targets38,48,58, modes2,5 and every unsupported mode with semicolon, colon and mixed separators. Port the accumulator and return-advance algorithm, including omitted color-space fields, missing values, extra subparams, truncated sequences and leading/following ordinary attributes. Do not replace it with a cleaner parser that changes which parameters are consumed.

Palette indexes and RGB channels use their low8bits, not clamp-to255. Default -1 subparams normalize to0. RGB and palette changes retain all style bits. Unknown modes leave color unchanged, but target58 still clones extended state as source does. Target58 does not independently update HAS_EXTENDED; preserve that observable packed-bit behavior.

Test full fg/bg/extended bits, URL id, payload and clone/share identity, not just rendered color. Use the actual Params storage and bounds. Add seeded operation traces with state resets, existing flags, extended state and both kitty option values. Record seeds and retain failing sequences.

## SGR3. Faithful assertions and integration

Port the source SGR assertions without weakening expected values. Record exact source case/assertion counts separately from new differential cases. Reuse unchanged InputHandler methods as the JS oracle, not a hand-coded second model. The oracle must retain all source helper bodies and actual AttributeData/Params semantics.

Run default and disabled-kitty traces through the actual integrated Rust terminal and compare full resulting cells/attributes with JS. Verify unknown-parameter diagnostics reach the real logger in order. Run native tests, source checks, rustfmt and warning-free Clippy with independent code review. Browser rendering and platform execution remain separate project gates.
