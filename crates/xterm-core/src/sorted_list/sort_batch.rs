use std::cmp::Ordering;

// ECMAScript leaves inconsistent-comparator ordering implementation-defined.
// Match the observed V8 initial-run and binary insertion behavior for small
// batches, without passing an invalid comparator to Rust's total-order sort.
/// Stable source-number ordering for comparators containing NaN. This follows
/// the observed small-batch V8 rule; ECMAScript leaves inconsistent comparison
/// order implementation-defined. Use ordinary stable sorting for total keys.
pub fn with_unordered_keys<T>(values: &mut [T], key: &impl Fn(&T) -> f64) {
    let compare = |a: &T, b: &T| {
        (key(a) - key(b))
            .partial_cmp(&0.0)
            .unwrap_or(Ordering::Equal)
    };
    if values.len() < 2 {
        return;
    }
    let descending = compare(&values[1], &values[0]).is_lt();
    let mut run_end = 2;
    while run_end < values.len() {
        let order = compare(&values[run_end], &values[run_end - 1]);
        if if descending {
            !order.is_lt()
        } else {
            order.is_lt()
        } {
            break;
        }
        run_end += 1;
    }
    if descending {
        values[..run_end].reverse();
    }
    for index in run_end..values.len() {
        let (mut low, mut high) = (0, index);
        while low < high {
            let middle = low + (high - low) / 2;
            if compare(&values[index], &values[middle]).is_lt() {
                high = middle;
            } else {
                low = middle + 1;
            }
        }
        values[low..=index].rotate_right(1);
    }
}
