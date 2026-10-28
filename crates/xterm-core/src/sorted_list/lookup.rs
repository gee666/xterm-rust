use super::{KeyReader, SortedKey, SortedList, SortedListError};
impl<T, R: KeyReader<T>> SortedList<T, R> {
    pub(super) fn search(&self, key: &SortedKey) -> Result<usize, SortedListError> {
        let (mut min, mut max) = (0, self.values.len());
        while min < max {
            let mid = min + (max - min - 1) / 2;
            let mid_key = self.read_key(self.values.get(mid))?;
            if mid_key.rel_gt(key) {
                max = mid;
            } else if mid_key.rel_lt(key) {
                min = mid + 1;
            } else {
                let mut first = mid;
                while first > 0 && self.read_key(self.values.get(first - 1))?.strict_eq(key) {
                    first -= 1;
                }
                return Ok(first);
            }
        }
        Ok(min)
    }
}
impl<T, F: Fn(&T) -> f64> SortedList<T, F> {
    /// Borrowed convenience iterator for nonthrowing f64 readers. Use KeyCursor
    /// and next_key_item for typed, fallible or source-reentrant iteration.
    pub fn get_key_iterator(
        &mut self,
        key: f64,
    ) -> Result<impl Iterator<Item = &T>, SortedListError> {
        self.flush()?;
        let start = self.search(&SortedKey::number(key))?;
        let get_key = &self.key;
        Ok(self.values[start..]
            .iter()
            .take_while(move |value| get_key(value) == key))
    }
    pub fn for_each_by_key(
        &mut self,
        key: f64,
        mut callback: impl FnMut(&T),
    ) -> Result<(), SortedListError> {
        for value in self.get_key_iterator(key)? {
            callback(value);
        }
        Ok(())
    }
}
