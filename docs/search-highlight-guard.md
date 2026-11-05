# Highlight search engine term guard

Task493 follows accepted480. Full347/322 and full-port equivalence remain NOT PROVEN.

The original SearchAddon validates a raw term before clearing highlights. Each subsequent SearchEngine.find separately checks raw `length === 0`. Rust previously converted the raw term to UTF16 before clearing decorations, omitted every highlight engine guard and retained a single matching-option snapshot across highlight calls.

The highlight path now retains the SearchTermHandle. Each engine entry captures its host, reads the existing strict-zero guard, clears the captured host on an empty guard, and skips cache, matching flags and text. Throwing guards propagate unchanged after old decoration cleanup, before selection clear or matching work. Nonempty guards initialize the cache before flags and matching text. Subsequent highlight calls repeat this order.

No dependency, unsafe code or JavaScript matching implementation is added. Ordinary engine guard480 is unchanged. Captured nonempty engine/cache ownership across reactivation, successful boxed/overridden matching methods, matching per-line short circuits and full SearchAddon suites remain347/322.

## Tests and scope

Three native functions expand18 scenarios over real terminal, selection, decorations and virtual clock services. They cover empty/error guards, reactivation, repeated guard calls after one or two matches, exact enum error cutoff and cache initialization before option errors. The retained59 functions are unchanged.

The shared fixture has84 directed cases inside one WASM test function per engine. It covers both search directions, positive/negative zero, activation during guard, six exact thrown identities, late highlight eligibility/argument failures, twelve nonnumeric strict guards stopped by an intentional regex getter sentinel and primitive hit/miss controls. These cases do not claim raw matching equivalence after that sentinel. Repeated raw-guard calls after actual matching are currently native-only because successful browser raw matching remains unimplemented.

`results/search-highlight-guard/inheritance.json` verifies all43 immutable480 archive pins and both776-file reference manifests. Four inherited files are intentionally modified by493; verification uses immutable Git objects rather than claiming unchanged working bytes.

Verification commands and observed results will be recorded after execution. No current native/browser pass is claimed by this draft.
