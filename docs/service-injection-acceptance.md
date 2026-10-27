# Service registry and constructor injection acceptance

Scope: `source_repo/src/common/services/{ServiceRegistry,InstantiationService}.ts`. No upstream test file exists for these modules. New source-branch assertions and a direct TypeScript oracle must be counted separately from upstream tests.

## Registry paths

R1: Repeated decorator names return the same identity within a registry, including empty names and Unicode. Different names remain distinct. Rust uses an owned registry, not mutable global state. Requesting an existing name with a different Rust type returns an explicit type mismatch rather than silently changing identity.

R2: Constructor metadata starts empty. An inherited descriptor exposes parent dependencies until the child declares its first dependency; that declaration replaces inherited dependencies. Further declarations append, including duplicate indices and duplicate services. Decorator invocation with other than three arguments errors without changing metadata. Constructor dependencies expose index order from registration until createInstance sorts them stably in place.

## Collection paths

C1: Empty get is absent and has is false. First set returns absence; replacement returns the previous shared instance. Constructor entries process left to right. Iteration follows first insertion order; replacing a key does not reorder it. Distinct same-name keys from different owned registries do not alias. Instance identity survives get and replacement. Iteration is live. Callbacks can append keys, replace not-yet-visited values and reenter iteration without a collection borrow crossing callbacks.

## Instantiation paths

I1: Self-registration is observable, can be replaced, and does not leak through a strong ownership cycle. Missing services reject before constructor execution. JavaScript false, zero, negative zero, NaN and empty string reject as UNKNOWN; nonempty string, objects and other numbers succeed. Rust services use typed values; primitive truthiness must be defined and tested, not silently ignored.

I2: Dependencies sort stably by index, including duplicates. Static arguments precede resolved services. A first dependency at index N accepts exactly N static arguments. Both shortage and surplus reject with exact source message and one-based position. No dependencies accept any static count. Later index gaps are not checked by upstream and must not gain invented validation. Unknown-service failure precedes static-count failure. Constructor callbacks execute once only after validation; errors propagate.

## Evidence and integration

E1: Tests name every R/C/I case, preserve source errors, and use an independent oracle compiling original TypeScript without editing it. Record test and assertion counts, tool versions, commands, and raw trace mismatches. Native Windows local evidence is distinct from unrun Linux/macOS/WASM checks. All implementation files stay below 350 lines.

E2: The terminal engine must use the same registry/injection behavior where observable, or explain a proven static-construction substitution with API mapping and browser tests. A standalone utility pass does not close terminal/browser integration. Binding exposure of decorator arity, inheritance metadata, and JavaScript value identity remains explicitly unproven until integrated.
