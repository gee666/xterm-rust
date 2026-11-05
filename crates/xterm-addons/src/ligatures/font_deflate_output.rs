// Output adapters for the source-derived inflater. The32KiB history is fixed
// by DEFLATE's distance coding, not a configurable font acceptance limit.
pub const WINDOW: usize = 32_768;
pub(super) trait Output {
    fn remaining(&self) -> usize;
    fn push(&mut self, value: u8);
    fn copy(&mut self, count: usize, distance: usize);
}
pub(super) struct Prefix<'a> {
    pub bytes: &'a mut [u8],
    pub written: usize,
}
impl Output for Prefix<'_> {
    fn remaining(&self) -> usize {
        self.bytes.len() - self.written
    }
    fn push(&mut self, value: u8) {
        if let Some(slot) = self.bytes.get_mut(self.written) {
            *slot = value;
            self.written += 1;
        }
    }
    fn copy(&mut self, count: usize, distance: usize) {
        for _ in 0..count.min(self.bytes.len() - self.written) {
            let value = self
                .written
                .checked_sub(distance)
                .and_then(|index| self.bytes.get(index))
                .copied()
                .unwrap_or(0);
            self.push(value);
        }
    }
}
pub(super) struct Range<'a> {
    pub bytes: &'a mut [u8],
    pub start: usize,
    pub limit: usize,
    pub written: usize,
    pub history: [u8; WINDOW],
}
impl Output for Range<'_> {
    fn remaining(&self) -> usize {
        self.limit - self.written
    }
    fn push(&mut self, value: u8) {
        if self.written >= self.limit {
            return;
        }
        self.history[self.written % WINDOW] = value;
        if let Some(slot) = self
            .written
            .checked_sub(self.start)
            .and_then(|i| self.bytes.get_mut(i))
        {
            *slot = value;
        }
        self.written += 1;
    }
    fn copy(&mut self, count: usize, distance: usize) {
        for _ in 0..count.min(self.limit - self.written) {
            let value = if distance > 0 && distance <= self.written && distance <= WINDOW {
                self.history[(self.written - distance) % WINDOW]
            } else {
                0
            };
            self.push(value);
        }
    }
}
