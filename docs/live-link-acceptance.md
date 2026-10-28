# Live browser link objects

This is unfinished implementation work, not an external blocker. It controls the follow-up to task35 and the utility web-links binding.

## Source contracts

Read `source_repo/src/browser/Linkifier.ts`, `browser/Types.d.ts` or its actual Types declaration file, `browser/services/Services.ts`, the public ILink/ILinkProvider/IBufferRange declarations in `source_repo/typings/xterm.d.ts`, and `source_repo/addons/addon-web-links/src/WebLinksAddon.ts`. Preserve the actual declarations and assertions; do not infer immutability from Rust's current types.

The current Rust `Rc<Link>` stores immutable text/range snapshots. Holding one stable JavaScript range object in callbacks fixes identity but does not fix subsequent hit-testing when that object changes. A one-time copy, or copying only when hover/activate fires, is insufficient.

## Required behavior

- A provider's original link object remains the live source of text/range/callback properties while it is cached, current, or retained for mousedown matching. Preserve original object identity and weak controller lifetime.
- Changing range.start/end between pointer moves changes later hit tests. The same-cell mousemove guard still applies: mutation alone must not invent a new hover event until source would reevaluate.
- Mousedown and current links referencing the same object both observe its latest fields at mouseup. Replacement objects compare their current text and all four current range coordinates, matching linkEquals rather than Rust snapshot age.
- Callback arguments use the current link text and the exact original range object. Hover, leave and activation must share range identity. Mutations made through that object must affect later cache/hover/selection behavior.
- OSC callback range objects have the same behavior even though Rust created the initial range. Source-generated URL text remains lossless UTF-16. Custom ILink objects may expose live text/range getters.
- Preserve getter and Proxy execution order and exceptions. `_linkAtPosition` reads range/start/y, range/start/x, range/end/y, range/end/x in source expression order. Overlap filtering short-circuits x getters when the range extends above/below the requested row. Do not add eager reads at registration or provider reply time.
- Optional hover/leave/dispose properties are looked up at the same phase as source, with the original receiver. Preserve optional-nullish behavior and thrown-value identity. A captured wrapper must not freeze a replaceable JS function member.
- Native Rust-only providers remain usable without a browser dependency. Keep typed native snapshots as one implementation of the shared contract; do not duplicate the priority/cache algorithm merely to add a live browser path.
- Default and partial decoration objects retain the source defaults and dynamic setter timing. Underline callbacks can mutate state before pointer handling. Callback exceptions retain the source's partial current/cache state.
- All cache references, current link, mousedown reference, pending provider callbacks, selection reads and disposal traversal must use the same live identity. Avoid an alias map that grows on every mousemove or retains disposed terminals through a strong callback cycle.

## Proof

Run source and Rust-backed browser traces with deterministic initial links and mutations between events. Include equal replacement objects, URI changes, wrapped ranges, mousedown/mouseup changes, getters with ordered logs, throwing getters/Proxy traps, optional callbacks replaced during hover, mutations from callback ranges, nonzero scrollback and disposal. Retain exact traces and counterexamples under `results/links/` with seed and source hashes. Test Chromium, Firefox and WebKit through the exported Rust Terminal and the web-links addon. Existing static-range tests remain intact. Do not weaken their assertions or mark this work complete because a mutable object is returned.

The implementation approach must be agreed between Cameron's link controller/algorithm and Lane's addon adapter, and exposed through Xavier's terminal registration API. `docs/browser-link-contract.md` records the current component APIs. Independent review task173 must inspect this follow-up as part of source equivalence.
