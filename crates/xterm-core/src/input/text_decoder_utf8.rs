// Copyright (c) 2019 The xterm.js authors. MIT license.
// Pending-prefix recovery also preserves mutations of the public interim array.
use super::{Utf32Output, Utf8ToUtf32};

impl Utf8ToUtf32 {
    /// None means input ended before the pending prefix completed. A negative
    /// position retains the source's complete-overlong-prefix retry behavior.
    pub(super) fn decode_pending<T: Utf32Output>(
        &mut self,
        input: &[u8],
        target: &mut T,
    ) -> Option<isize> {
        let first = self.interim[0];
        let (length, mask) = if first & 0xe0 == 0xc0 {
            (2, 0x1f)
        } else if first & 0xf0 == 0xe0 {
            (3, 0x0f)
        } else {
            (4, 0x07)
        };
        let mut cp = u32::from(first & mask);
        let mut position = 1;
        while position < self.interim.len() && self.interim[position] != 0 {
            cp = (cp << 6) | u32::from(self.interim[position] & 0x3f);
            position += 1;
        }
        let missing = length - position as isize;
        let mut start = 0_isize;
        let mut discarded = false;
        while start < missing {
            let &byte = input.get(start as usize)?;
            start += 1;
            if byte & 0xc0 != 0x80 {
                start -= 1;
                discarded = true;
                break;
            }
            // A fourth-byte write is ignored by the source's three-byte array.
            if let Some(slot) = self.interim.get_mut(position) {
                *slot = byte;
            }
            position += 1;
            cp = (cp << 6) | u32::from(byte & 0x3f);
        }
        if !discarded {
            match length {
                2 if cp < 0x80 => start -= 1,
                2 => target.push(cp),
                3 if cp >= 0x800 && !(0xd800..=0xdfff).contains(&cp) && cp != 0xfeff => {
                    target.push(cp)
                }
                4 if (0x10000..=0x10ffff).contains(&cp) => target.push(cp),
                _ => {}
            }
        }
        self.clear();
        Some(start)
    }
}
