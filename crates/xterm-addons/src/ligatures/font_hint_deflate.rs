// Discarded hint-table validation follows tinf in the supplied opentype.js.
// Copyright (c) 2015-present Devon Govett. MIT license.
// See licenses/tiny-inflate-MIT.txt and licenses/opentype-js-MIT.txt.
// This source-derived adaptation retains no glyph hints or output bytes.
use super::FontError;

const CODE_LENGTH_ORDER: [usize; 19] = [
    16, 17, 18, 0, 8, 7, 9, 6, 10, 5, 11, 4, 12, 3, 13, 2, 14, 1, 15,
];
const END_BLOCK: u16 = 256;
const LENGTH_SYMBOL: u16 = 257;
const MAX_CODE_LENGTH: usize = 15;

struct Bits<'a> {
    bytes: &'a [u8],
    index: usize,
    bit: u8,
}
impl Bits<'_> {
    fn read(&mut self, count: u8) -> Result<u16, FontError> {
        let mut value = 0;
        for shift in 0..count {
            // Source bitwise operations turn undefined source bytes into zero.
            let byte = self.bytes.get(self.index).copied().unwrap_or(0);
            value |= u16::from((byte >> self.bit) & 1) << shift;
            self.bit += 1;
            if self.bit == 8 {
                self.index = self.index.checked_add(1).ok_or(FontError::Bounds)?;
                self.bit = 0;
            }
        }
        Ok(value)
    }
    fn stored(&mut self) -> Result<(), FontError> {
        if self.bit != 0 {
            self.index = self.index.checked_add(1).ok_or(FontError::Bounds)?;
            self.bit = 0;
        }
        let start = self.index;
        let word = |offset: usize| {
            let low = self.bytes.get(start.checked_add(offset)?)?;
            let high = self.bytes.get(start.checked_add(offset + 1)?)?;
            Some(u16::from_le_bytes([*low, *high]))
        };
        let length = word(0);
        // Arithmetic with *either* missing inverse byte yields NaN. JS ~NaN
        // is -1, regardless of the other byte, not a zero-padded inverse.
        let inverse = word(2).unwrap_or(0);
        if length != Some(!inverse) {
            return Err(FontError::Compression);
        }
        // Equality above establishes an actual two-byte length. Missing payload
        // is stored as zero by the source's typed array, with no further reads
        // affecting construction, so skip it rather than materializing it.
        let length = usize::from(length.ok_or(FontError::Compression)?);
        self.index = start.checked_add(4 + length).ok_or(FontError::Bounds)?;
        Ok(())
    }
}
struct Tree {
    counts: [u16; 16],
    values: [u16; 288],
}
impl Tree {
    fn new() -> Self {
        Self {
            counts: [0; 16],
            values: [0; 288],
        }
    }
    fn build(&mut self, lengths: &[u8]) {
        self.counts.fill(0);
        for &length in lengths {
            self.counts[usize::from(length)] += 1;
        }
        self.counts[0] = 0;
        let mut offsets = [0; 16];
        let mut sum = 0;
        for (offset, count) in offsets.iter_mut().zip(self.counts) {
            *offset = sum;
            sum += usize::from(count);
        }
        for (symbol, &length) in lengths.iter().enumerate() {
            if length != 0 {
                let offset = &mut offsets[usize::from(length)];
                // All callers build from at most288 literal symbols or32
                // distance symbols, using lengths in0..=15.
                self.values[*offset] = symbol as u16;
                *offset += 1;
            }
        }
    }
    fn decode(&self, input: &mut Bits<'_>) -> Result<Option<u16>, FontError> {
        let (mut sum, mut current) = (0, 0);
        for length in 1..=MAX_CODE_LENGTH {
            current = 2 * current + i32::from(input.read(1)?);
            let count = i32::from(self.counts[length]);
            sum += count;
            current -= count;
            if current < 0 {
                return Ok(usize::try_from(sum + current)
                    .ok()
                    .and_then(|index| self.values.get(index))
                    .copied());
            }
        }
        // Source reads table[16] as undefined, leaving NaN and an undefined
        // symbol after consuming this last bit. Do not loop on a missing code.
        input.read(1)?;
        Ok(None)
    }
}
fn dynamic(
    input: &mut Bits<'_>,
    literals: &mut Tree,
    distances: &mut Tree,
    codes: &mut Tree,
) -> Result<(), FontError> {
    let literal_count = usize::from(input.read(5)?) + 257;
    let distance_count = usize::from(input.read(5)?) + 1;
    let code_count = usize::from(input.read(4)?) + 4;
    let mut lengths = [0; 320];
    for &index in &CODE_LENGTH_ORDER[..code_count] {
        lengths[index] = input.read(3)? as u8;
    }
    codes.build(&lengths[..19]);
    let mut count = 0;
    while count < literal_count + distance_count {
        let symbol = codes.decode(input)?;
        let (length, repeated) = match symbol {
            Some(16) => (
                count.checked_sub(1).map_or(0, |index| lengths[index]),
                usize::from(input.read(2)?) + 3,
            ),
            Some(17) => (0, usize::from(input.read(3)?) + 3),
            Some(18) => (0, usize::from(input.read(7)?) + 11),
            // Undefined becomes zero when assigned to the source Uint8Array.
            value => (value.unwrap_or(0) as u8, 1),
        };
        for _ in 0..repeated {
            if let Some(slot) = lengths.get_mut(count) {
                *slot = length;
            }
            count += 1;
        }
    }
    literals.build(&lengths[..literal_count]);
    distances.build(&lengths[literal_count..literal_count + distance_count]);
    Ok(())
}
fn data(input: &mut Bits<'_>, literals: &Tree, distances: &Tree) -> Result<(), FontError> {
    loop {
        let exhausted = input.index >= input.bytes.len();
        match literals.decode(input)? {
            Some(END_BLOCK) => return Ok(()),
            Some(value) if value < END_BLOCK => {}
            value => {
                if let Some(index) = value.and_then(|v| v.checked_sub(LENGTH_SYMBOL)) {
                    if index < 30 {
                        let bits = if index == 28 {
                            0
                        } else {
                            index.saturating_sub(4) / 4
                        };
                        input.read(bits as u8)?;
                    }
                }
                if let Some(distance) = distances.decode(input)? {
                    if distance < 30 {
                        input.read((distance.saturating_sub(2) / 2) as u8)?;
                    }
                }
                // Source does not reject distances beyond the output array.
                // All copied bytes are discarded by this constructor check.
            }
        }
        if exhausted {
            // Once input is exhausted, each read is zero and the same tree
            // always decodes the same symbol. If it was not END_BLOCK above,
            // source output continues forever. Reject instead of hanging.
            return Err(FontError::Compression);
        }
    }
}
pub(super) fn validate(bytes: &[u8]) -> Result<(), FontError> {
    let mut input = Bits {
        bytes,
        index: 0,
        bit: 0,
    };
    let mut fixed_literals = Tree::new();
    let mut fixed_distances = Tree::new();
    let mut lengths = [8; 288];
    lengths[144..256].fill(9);
    lengths[256..280].fill(7);
    fixed_literals.build(&lengths);
    fixed_distances.build(&[5; 32]);
    let (mut literals, mut distances, mut codes) = (Tree::new(), Tree::new(), Tree::new());
    loop {
        let final_block = input.read(1)? != 0;
        match input.read(2)? {
            0 => input.stored()?,
            1 => data(&mut input, &fixed_literals, &fixed_distances)?,
            2 => {
                dynamic(&mut input, &mut literals, &mut distances, &mut codes)?;
                data(&mut input, &literals, &distances)?;
            }
            _ => return Err(FontError::Compression),
        }
        if final_block {
            return Ok(());
        }
    }
}
