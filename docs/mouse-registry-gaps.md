# Mouse registry dynamic-name gap

The source stores protocol and encoding entries in ordinary JavaScript objects, not null-prototype objects or Maps. `activeProtocol` and `activeEncoding` check the result of a property lookup for truthiness. Inherited names such as `toString`, `constructor`, `hasOwnProperty`, and `__proto__` therefore do not necessarily follow the unknown-name error path. Assigning `__proto__` can change the registry prototype instead of adding an own property. `Object.keys` orders canonical integer names before other names.

The current Rust service has typed protocol and encoder registrations in ordered vectors. It rejects unknown inherited names and preserves insertion order for diagnostic name lists. Ordinary named custom registrations, shared mutable protocol identity and mutable encoder callbacks are covered separately. Those tests do not establish parity for all strings.

This is an unresolved compatibility gap, not an external blocker and not permission to mark criterion76 complete. A linked follow-up must characterize the exact source outputs, events and thrown errors, retain counterexamples, and implement explicit Rust/binding representations or an equivalent observable boundary. Inherited encoder calls can return non-string JavaScript values despite the TypeScript declaration. A `Vec<u16>` return alone cannot represent those values. Do not erase this distinction by converting arbitrary values to a success string, swallowing errors or silently returning empty reports.

Overall mouse-name and full-terminal equivalence remain NOT PROVEN until the follow-up and the remaining integration gates pass. The typed service report is scoped evidence only.
