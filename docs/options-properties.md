# Mutable option data

`OptionValue::property_map` and `empty_object` create shared mutable native data bags. `set_property`, `delete_property` and `property_keys` expose mutation without replacing the root option or firing an option-change event. All default bags use this storage, including `WindowsPty::into`.

Cloned handles, peer services and subsequently constructed services observe the same mutated defaults. Replacing a service's root option detaches only that service. Replaced values drop after the storage borrow ends, so native destructors can reenter safely.

Native data bags preserve own-property insertion order with canonical integer keys first. They reject native data cycles to avoid leaking Rc graphs. They are not a general JavaScript prototype heap. For host objects, `PropertyAccess` delegates getters, setters, deletion and enumeration to the actual host object. The WASM adapter therefore retains JS receiver behavior, thrown values and cyclic JS graphs.

Checks run in `swarm/Athena`: 43 native options tests passed; the original 10 cases / 22 assertions and 31,961 differential lines passed. WASM options 15, event 6 and property 1 cases passed in Chromium, Firefox and WebKit. Scoped native and WASM Clippy passed with `-D warnings`.

This is component verification. Full Terminal integration and cross-GC ownership remain separate work; full-port equivalence is not proven.
