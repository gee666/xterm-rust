use super::{InvalidArrayLength, KeyReader, SortedList, SortedListError};
use std::cmp::Ordering;
impl<T, R: KeyReader<T>> SortedList<T, R> {
    pub(super) fn flush_inserted(&mut self) -> Result<(), SortedListError> {
        if self.inserted.is_empty() {
            return Ok(());
        }
        // Sort only a permutation until comparisons succeed. JS Array.sort
        // commits no reordered elements when a comparison throws.
        let mut order = Vec::with_capacity(self.inserted.len());
        let mut undefined = Vec::new();
        for (index, value) in self.inserted.iter().enumerate() {
            if self.is_undefined_value(value) {
                undefined.push(index);
            } else {
                order.push(index);
            }
        }
        try_binary_sort(&mut order, |a, b| -> Result<Ordering, SortedListError> {
            let a = self.read_key(self.inserted.get(a))?;
            let b = self.read_key(self.inserted.get(b))?;
            Ok(a.numeric_difference(&b)
                .partial_cmp(&0.0)
                .unwrap_or(Ordering::Equal))
        })?;
        order.extend(undefined);
        // This is a complete destination-to-source permutation. Apply each
        // cycle with swaps, without cloning T or allocating a second value array.
        for start in 0..order.len() {
            let mut position = start;
            while order[position] != position {
                let next = order[position];
                order[position] = position;
                if next == start {
                    break;
                }
                self.inserted.swap(position, next);
                position = next;
            }
        }
        // The pending batch is already sorted before merge reads. A later
        // getter failure must retain that batch and the old array separately.
        let total = self
            .values
            .len()
            .checked_add(self.inserted.len())
            .ok_or(InvalidArrayLength)?;
        let mut plan = Vec::with_capacity(total);
        let (mut old, mut added) = (0, 0);
        while plan.len() < total {
            let choose_added = if old >= self.values.len() {
                true
            } else {
                let added_key = self.read_key(self.inserted.get(added))?;
                let old_key = self.read_key(self.values.get(old))?;
                added_key.rel_le(&old_key)
            };
            plan.push(choose_added);
            if choose_added {
                added += 1;
            } else {
                old += 1;
            }
        }
        let mut old = std::mem::take(&mut self.values).into_iter();
        let mut added = std::mem::take(&mut self.inserted).into_iter();
        self.values.reserve(plan.len());
        for choose_added in plan {
            let value = if choose_added {
                added.next()
            } else {
                old.next()
            }
            .expect("merge plan selects one existing source entry");
            self.values.push(value);
        }
        Ok(())
    }
}
/// Stable binary insertion with the V8 small-batch initial-run rule. Large
/// inconsistent-comparator engine policies are audited by tasks186/188; this
/// routine does not assume Rust's comparator is a total order or swallow errors.
fn try_binary_sort<E>(
    order: &mut [usize],
    mut compare: impl FnMut(usize, usize) -> Result<Ordering, E>,
) -> Result<(), E> {
    if order.len() < 2 {
        return Ok(());
    }
    let descending = compare(order[1], order[0])?.is_lt();
    let mut end = 2;
    while end < order.len() {
        let result = compare(order[end], order[end - 1])?;
        if if descending {
            !result.is_lt()
        } else {
            result.is_lt()
        } {
            break;
        }
        end += 1;
    }
    if descending {
        order[..end].reverse();
    }
    for index in end..order.len() {
        let pivot = order[index];
        let (mut low, mut high) = (0, index);
        while low < high {
            let middle = low + (high - low) / 2;
            if compare(pivot, order[middle])?.is_lt() {
                high = middle;
            } else {
                low = middle + 1;
            }
        }
        order[low..=index].rotate_right(1);
    }
    Ok(())
}
