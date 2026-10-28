# Sorted key reader contract

Task 186 owns the collection adaptation. The existing f64 constructor remains available, and the standalone image `stable_sort_by_js_number` helper is unchanged.

`KeyReader<T>::read_key(Option<&T>)` returns `Result<SortedKey, SortedListError>`. None represents an exhausted-array undefined argument. `TryKey::new` wraps a fallible getter. `SortedList::from_reader` constructs a typed list. `SortedKey` is Undefined or an owned MarkerLine Number/String value. It reuses the portable scalar's numeric conversion, UTF-16 relational comparison and strict equality rather than copying a coercion implementation.

Sort comparison uses numeric subtraction. Binary-search relations use raw key relations, so two String keys compare lexicographically. Duplicate checks use strict equality and distinguish String from Number. Undefined is not silently replaced by NaN for early deletion or strict comparison.

Sorting plans a permutation before moving values. A getter error leaves the original pending batch order intact. After a successful sort, merge errors retain the sorted pending batch separately from the old committed array. Errors can retain their original object through `SortedListError::Callback` and the standard source chain.

`KeyCursor::new` accepts a typed key. `next_key_item` is the faithful lazy generator adapter and returns Value, Undefined yield, or completion. It closes permanently after a getter error. The native `next_by_key` convenience remains suitable when T represents every yielded value. `try_for_each_key_item_reentrant` also retains an out-of-range undefined callback argument. Default iteration contexts share the source module's work index on one UI thread; native callers can explicitly select an isolated context.

Pure undefined-element representation metadata is separate from a key getter returning undefined. Actual undefined array elements sort last without comparison getter calls.

The current index sorter follows the observed small-batch V8 initial-run and binary insertion rule. Task 186 is not complete until the 64+ mixed NaN/undefined policies and getter traces have been compared across supported runtimes under task 188. The existing 59 scoped tests pass, including six new typed-key/error/reentry cases and all 4,055 retained dynamic operations. This does not claim complete large-batch or cross-engine equivalence.
