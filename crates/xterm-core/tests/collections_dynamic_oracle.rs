use std::cell::Cell;
use std::collections::HashMap;
use std::rc::Rc;
use xterm_core::sorted_list::{IterationContext, KeyCursor, SortedList, SortedListError};

struct Entry {
    key: Cell<Option<f64>>,
    id: String,
}
type List = SortedList<Rc<Entry>, fn(&Rc<Entry>) -> f64>;
fn key(value: &Rc<Entry>) -> f64 {
    value.key.get().unwrap_or(f64::NAN)
}
fn number(value: &str) -> Option<f64> {
    if value == "u" {
        None
    } else {
        Some(value.parse().unwrap())
    }
}
fn error_name(error: &SortedListError) -> &'static str {
    match error {
        SortedListError::InvalidArrayLength => "RangeError",
        SortedListError::KeyAccess(message) if message.starts_with("callback ") => "Error",
        SortedListError::KeyAccess(_) => "TypeError",
    }
}
fn new_list(context: IterationContext) -> List {
    SortedList::new(key as fn(&Rc<Entry>) -> f64)
        .with_iteration_context(context)
        .with_undefined_key(|value| value.key.get().is_none())
        .with_undefined_key_error("Cannot read properties of undefined (reading 'key')")
}

#[test]
fn identity_dynamic_keys_and_reentrant_traces_match_source() {
    let fixture = include_str!("fixtures/collections-dynamic.tsv");
    let mut rows = fixture.lines();
    let expected_count: usize = rows
        .next()
        .unwrap()
        .strip_prefix("#cases\t")
        .unwrap()
        .parse()
        .unwrap();
    let context = IterationContext::default();
    let mut lists = [new_list(context.clone()), new_list(context)];
    let mut entries: HashMap<String, Rc<Entry>> = HashMap::new();
    let mut cursors: HashMap<String, (usize, KeyCursor)> = HashMap::new();
    let mut count = 0;
    for (index, row) in rows.enumerate() {
        let parts: Vec<_> = row.split('\t').collect();
        assert_eq!(parts.len(), 5, "row {index}");
        let (command, which, id, arg, expected) = (
            parts[0],
            parts[1].parse::<usize>().unwrap(),
            parts[2],
            parts[3],
            parts[4],
        );
        let mut visited = Vec::new();
        let actual: Result<String, SortedListError> = (|| {
            let list = &mut lists[which];
            match command {
                "I" => {
                    let value = entries.entry(id.to_owned()).or_insert_with(|| {
                        Rc::new(Entry {
                            key: Cell::new(number(arg)),
                            id: id.to_owned(),
                        })
                    });
                    list.insert(value.clone())?;
                    Ok("ok".into())
                }
                "M" => {
                    entries[id].key.set(number(arg));
                    Ok("ok".into())
                }
                "D" => Ok(list.delete_by(&entries[id], Rc::ptr_eq)?.to_string()),
                "V" => Ok(list
                    .values()?
                    .map(|v| v.id.clone())
                    .collect::<Vec<_>>()
                    .join(",")),
                "C" => {
                    list.clear();
                    Ok("ok".into())
                }
                "G" => {
                    cursors.insert(id.to_owned(), (which, KeyCursor::new(number(arg).unwrap())));
                    Ok("ok".into())
                }
                "N" => {
                    let (which, cursor) = cursors.get_mut(id).unwrap();
                    Ok(lists[*which]
                        .next_by_key(cursor)?
                        .map_or_else(|| "undefined".into(), |v| v.id.clone()))
                }
                "F" => {
                    let query = number(id).unwrap();
                    list.try_for_each_by_key_reentrant(query, |list, value| {
                        if visited.len() >= 128 {
                            return Err(SortedListError::KeyAccess("callback cap".into()));
                        }
                        visited.push(value.id.clone());
                        match arg {
                            "clear" => list.clear(),
                            "delete" => {
                                list.delete_by(&value, Rc::ptr_eq)?;
                            }
                            "delete-read" => {
                                list.delete_by(&value, Rc::ptr_eq)?;
                                list.values()?;
                            }
                            "nested" => list.for_each_by_key_reentrant(query, |_, inner| {
                                visited.push(format!("inner_{}", inner.id))
                            })?,
                            "fail" => {
                                return Err(SortedListError::KeyAccess("callback failure".into()))
                            }
                            _ => panic!("unknown generated callback {arg}"),
                        }
                        Ok(())
                    })?;
                    Ok(visited.join(","))
                }
                _ => panic!("unknown generated command {command}"),
            }
        })();
        let actual =
            actual.unwrap_or_else(|error| format!("{}:{}", error_name(&error), visited.join(",")));
        assert_eq!(
            actual, expected,
            "row {index}: {command} {which} {id} {arg}"
        );
        count += 1;
    }
    assert_eq!(count, expected_count);
    assert_eq!(count, 4055);
}
