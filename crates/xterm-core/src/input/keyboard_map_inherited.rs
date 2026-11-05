// Licensed under MIT. Object.prototype lookup compatibility for synthetic keys.
// Native function strings use the documented Node 24/V8 representation. DOM
// event sources bypass this data and retain their own realm's raw map values.
pub(crate) fn inherited_name(key: &[u16]) -> Option<&'static str> {
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
    .find_map(|(name, value)| key.iter().copied().eq(name.encode_utf16()).then_some(value))
}
