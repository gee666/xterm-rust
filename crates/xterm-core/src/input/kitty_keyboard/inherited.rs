// Licensed under MIT. Object.prototype lookup compatibility for synthetic keys.
// Native function strings are the observed Node 24/V8 representation. Browser
// engine verification is still needed because JS permits whitespace variation.
pub(super) fn inherited_name(key: &[u16]) -> Option<&'static str> {
    [
        ("constructor", "function Object() { [native code] }"),
        (
            "__defineGetter__",
            "function __defineGetter__() { [native code] }",
        ),
        (
            "__defineSetter__",
            "function __defineSetter__() { [native code] }",
        ),
        (
            "hasOwnProperty",
            "function hasOwnProperty() { [native code] }",
        ),
        (
            "__lookupGetter__",
            "function __lookupGetter__() { [native code] }",
        ),
        (
            "__lookupSetter__",
            "function __lookupSetter__() { [native code] }",
        ),
        (
            "isPrototypeOf",
            "function isPrototypeOf() { [native code] }",
        ),
        (
            "propertyIsEnumerable",
            "function propertyIsEnumerable() { [native code] }",
        ),
        ("toString", "function toString() { [native code] }"),
        ("valueOf", "function valueOf() { [native code] }"),
        ("__proto__", "[object Object]"),
        (
            "toLocaleString",
            "function toLocaleString() { [native code] }",
        ),
    ]
    .into_iter()
    .find_map(|(name, value)| super::keys::equals(key, name).then_some(value))
}
