//! Ordering for string-key registries backed by JavaScript ordinary objects.

/// Registrations are unique and supplied in insertion order. Array-index names
/// precede other names numerically; the remaining names keep insertion order.
/// This orders own string keys only, not prototype lookup or `__proto__` writes.
pub(crate) fn ordered_names<'a>(names: impl IntoIterator<Item = &'a str>) -> Vec<String> {
    let mut indexed = Vec::new();
    let mut named = Vec::new();
    for name in names {
        if let Some(index) = array_index(name) {
            indexed.push((index, name));
        } else {
            named.push(name);
        }
    }
    indexed.sort_unstable_by_key(|entry| entry.0);
    indexed
        .into_iter()
        .map(|entry| entry.1)
        .chain(named)
        .map(str::to_owned)
        .collect()
}

fn array_index(value: &str) -> Option<u32> {
    let index: u32 = value.parse().ok()?;
    (index != u32::MAX && index.to_string() == value).then_some(index)
}

#[cfg(test)]
mod tests {
    use super::{array_index, ordered_names};

    #[test]
    fn only_canonical_uint32_names_below_max_are_indices() {
        for (name, index) in [("0", 0), ("1", 1), ("4294967294", u32::MAX - 1)] {
            assert_eq!(array_index(name), Some(index));
        }
        for name in [
            "",
            "-0",
            "-1",
            "+0",
            "+1",
            "00",
            "01",
            "1.0",
            "1e0",
            " 1",
            "1 ",
            "4294967295",
            "4294967296",
            "18446744073709551616",
            "١",
            "１",
        ] {
            assert_eq!(array_index(name), None, "{name:?}");
        }
    }

    #[test]
    fn named_keys_keep_insertion_order_after_numeric_keys() {
        assert_eq!(
            ordered_names(["text", "10", "02", "2", "4294967295", "0", ""]),
            ["0", "2", "10", "text", "02", "4294967295", ""]
        );
        assert!(ordered_names([]).is_empty());
    }
}
