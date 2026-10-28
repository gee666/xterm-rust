//! Source-derived merge graphs compare identity and mutation as well as values.
use super::{
    font_merge, font_merge_range,
    font_tables::GlyphValue,
    font_tree::{Arena, Entry, EntryId, LookupId, LookupResult, TreeId},
};
use serde_json::{json, Value};
use std::collections::HashMap;
fn array(value: &Value) -> &Vec<Value> {
    value.as_array().unwrap()
}
fn index(value: &Value) -> usize {
    usize::try_from(value.as_u64().unwrap()).unwrap()
}
fn glyph(value: &Value) -> GlyphValue {
    match value.as_str() {
        Some("NaN") => GlyphValue::NaN,
        Some("undefined") => GlyphValue::Undefined,
        _ => GlyphValue::Number(value.as_i64().unwrap()),
    }
}
fn decode(value: &Value) -> (Arena, Vec<TreeId>, Vec<LookupId>) {
    let mut arena = Arena::default();
    let trees: Vec<_> = array(&value["trees"])
        .iter()
        .map(|_| arena.tree().unwrap())
        .collect();
    let entries: Vec<_> = array(&value["entries"])
        .iter()
        .map(|_| arena.entry().unwrap())
        .collect();
    let lookups: Vec<_> = array(&value["lookups"])
        .iter()
        .map(|lookup| {
            arena
                .lookup(LookupResult {
                    substitutions: array(&lookup["substitutions"])
                        .iter()
                        .map(|value| {
                            if value.is_null() {
                                None
                            } else {
                                Some(glyph(value))
                            }
                        })
                        .collect(),
                    length: index(&lookup["length"]),
                    index: index(&lookup["index"]),
                    sub_index: index(&lookup["subIndex"]),
                    context_range: [
                        lookup["contextRange"][0].as_i64().unwrap(),
                        lookup["contextRange"][1].as_i64().unwrap(),
                    ],
                })
                .unwrap()
        })
        .collect();
    for (id, entry) in array(&value["entries"]).iter().enumerate() {
        *arena.get_mut(entries[id]) = Entry {
            lookup: entry["lookup"]
                .as_u64()
                .map(|value| lookups[usize::try_from(value).unwrap()]),
            forward: entry["forward"]
                .as_u64()
                .map(|value| trees[usize::try_from(value).unwrap()]),
            reverse: entry["reverse"]
                .as_u64()
                .map(|value| trees[usize::try_from(value).unwrap()]),
        };
    }
    for (id, tree) in array(&value["trees"]).iter().enumerate() {
        for entry in array(&tree["individual"]) {
            arena
                .set_individual(
                    trees[id],
                    entry[0].as_str().unwrap().parse().unwrap(),
                    entries[index(&entry[1])],
                )
                .unwrap();
        }
        for range in array(&tree["range"]) {
            arena
                .push_range(
                    trees[id],
                    range["range"][0].as_i64().unwrap(),
                    range["range"][1].as_i64().unwrap(),
                    entries[index(&range["entry"])],
                )
                .unwrap();
        }
    }
    let roots = array(&value["roots"])
        .iter()
        .map(|root| trees[index(root)])
        .collect();
    (arena, roots, lookups)
}
enum Compare {
    Tree(usize, TreeId),
    Entry(usize, EntryId),
    Lookup(usize, LookupId),
}
fn register<T: Copy + Eq + std::hash::Hash>(
    expected: usize,
    actual: T,
    forward: &mut HashMap<usize, T>,
    reverse: &mut HashMap<T, usize>,
) -> bool {
    if let Some(previous) = forward.get(&expected) {
        assert!(*previous == actual, "source alias lost");
        return false;
    }
    assert!(
        !reverse.contains_key(&actual),
        "distinct source objects collapsed"
    );
    forward.insert(expected, actual);
    reverse.insert(actual, expected);
    true
}
fn compare(arena: &Arena, roots: &[TreeId], expected: &Value) {
    let mut trees = HashMap::new();
    let mut reverse_trees = HashMap::new();
    let mut entries = HashMap::new();
    let mut reverse_entries = HashMap::new();
    let mut lookups = HashMap::new();
    let mut reverse_lookups = HashMap::new();
    assert_eq!(roots.len(), array(&expected["roots"]).len());
    let mut work: Vec<_> = array(&expected["roots"])
        .iter()
        .zip(roots)
        .map(|(left, right)| Compare::Tree(index(left), *right))
        .collect();
    while let Some(job) = work.pop() {
        match job {
            Compare::Tree(id, actual) => {
                if !register(id, actual, &mut trees, &mut reverse_trees) {
                    continue;
                }
                let tree = &expected["trees"][id];
                let values = arena.individuals(actual).unwrap();
                assert_eq!(values.len(), array(&tree["individual"]).len());
                for ((glyph, entry), expected) in values.iter().zip(array(&tree["individual"])) {
                    assert_eq!(
                        *glyph,
                        expected[0].as_str().unwrap().parse::<i64>().unwrap()
                    );
                    work.push(Compare::Entry(index(&expected[1]), *entry));
                }
                let mut cursor = 0;
                while let Some(range) = arena.range(actual, cursor) {
                    let expected = &tree["range"][cursor];
                    assert_eq!(json!([range.start, range.end]), expected["range"]);
                    work.push(Compare::Entry(index(&expected["entry"]), range.entry));
                    cursor += 1;
                }
                assert_eq!(cursor, array(&tree["range"]).len());
            }
            Compare::Entry(id, actual) => {
                if !register(id, actual, &mut entries, &mut reverse_entries) {
                    continue;
                }
                let expected = &expected["entries"][id];
                let entry = arena.get(actual);
                match entry.lookup {
                    Some(value) => work.push(Compare::Lookup(index(&expected["lookup"]), value)),
                    None => assert!(expected["lookup"].is_null()),
                }
                for (field, value) in [("forward", entry.forward), ("reverse", entry.reverse)] {
                    match value {
                        Some(value) => work.push(Compare::Tree(index(&expected[field]), value)),
                        None => assert!(expected[field].is_null()),
                    }
                }
            }
            Compare::Lookup(id, actual) => {
                if !register(id, actual, &mut lookups, &mut reverse_lookups) {
                    continue;
                }
                let result = arena.result(actual);
                let substitutions: Vec<_> = result
                    .substitutions
                    .iter()
                    .map(|value| value.map_or(Value::Null, super::tests::glyph))
                    .collect();
                assert_eq!(
                    json!({"substitutions":substitutions,"length":result.length,"index":result.index,"subIndex":result.sub_index,"contextRange":result.context_range}),
                    expected["lookups"][id]
                );
            }
        }
    }
}
#[test]
fn every_original_merge_graph_preserves_input_mutation_and_object_aliases() {
    let cases: Value = serde_json::from_str(include_str!(
        "../../tests/fixtures/ligature-merge-trace.json"
    ))
    .unwrap();
    for case in array(&cases) {
        let (mut arena, mut roots, _) = decode(&case["before"]);
        let result = font_merge::merge(&mut arena, &roots).unwrap();
        roots.push(result);
        compare(&arena, &roots, &case["after"]);
    }
    assert_eq!(array(&cases).len(), 5);
}
#[test]
fn directed_overlap_flatten_cycles_and_walk_priorities_match_source() {
    let cases: Value = serde_json::from_str(include_str!(
        "../../tests/fixtures/ligature-directed-tree.json"
    ))
    .unwrap();
    for (row, case) in array(&cases).iter().enumerate() {
        let (mut arena, mut roots, lookups) = decode(&case["before"]);
        match case["operation"].as_str().unwrap() {
            "merge" => {
                let result = font_merge::merge(&mut arena, &roots)
                    .unwrap_or_else(|error| panic!("row{row}: {error}"));
                roots.push(result);
                compare(&arena, &roots, &case["after"]);
            }
            "flatten" => {
                let result = arena.flatten(roots[0]).unwrap();
                roots.push(result);
                compare(&arena, &roots, &case["after"]);
                let (retained, roots) = arena.retain(&roots).unwrap();
                compare(&retained, &roots, &case["after"]);
            }
            "walk" => {
                let sequence: Vec<_> = array(&case["sequence"]).iter().map(glyph).collect();
                let result = super::font_walk::walk(
                    &arena,
                    roots[index(&case["root"])],
                    &sequence,
                    index(&case["start"]),
                );
                let expected = case["result"]
                    .as_u64()
                    .map(|id| lookups[usize::try_from(id).unwrap()]);
                assert_eq!(result, expected, "row{row}");
            }
            other => panic!("unknown fixture operation {other}"),
        }
    }
    assert_eq!(array(&cases).len(), 217);
}

#[test]
fn every_original_range_merge_assertion_matches() {
    let cases: Value = serde_json::from_str(include_str!(
        "../../tests/fixtures/ligature-range-trace.json"
    ))
    .unwrap();
    for case in array(&cases) {
        let mut ranges: Vec<[i64; 2]> = array(&case["before"])
            .iter()
            .map(|pair| [pair[0].as_i64().unwrap(), pair[1].as_i64().unwrap()])
            .collect();
        font_merge_range::merge(
            &mut ranges,
            case["start"].as_i64().unwrap(),
            case["end"].as_i64().unwrap(),
        )
        .unwrap();
        assert_eq!(json!(ranges), case["after"]);
        assert_eq!(case["sameObject"], true);
    }
    assert_eq!(array(&cases).len(), 8);
}
