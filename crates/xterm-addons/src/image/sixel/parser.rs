// Copyright (c) 2021 Joerg Breitbart. MIT license.
// Port of node-sixel 0.16.0 wasm/decoder.cpp, with checked native storage.
use super::{
    DecodeError, RasterAttributes, SixelDecoder, SixelLevel, SixelMode, State, MAX_WIDTH, STRIDE,
};

fn code<T: Copy + Into<u32>>(value: T) -> u8 {
    (value.into() as u8) & 0x7f
}

impl SixelDecoder {
    pub(super) fn decode_chunk<T: Copy + Into<u32>>(
        &mut self,
        data: &[T],
    ) -> Result<(), DecodeError> {
        if self.mode == SixelMode::Undecided {
            self.scan_raster(data);
            if self.mode == SixelMode::Undecided {
                return Ok(());
            }
            self.reset_band()?;
            self.prepare_canvas()?;
            // The original replays the complete current chunk after choosing a
            // mode, not merely the suffix following the raster terminator.
        }
        let previous_state = self.state;
        let previous_color = self.color;
        let mut stored_cursor = self.cursor;
        let result = self.decode_data(data, &mut stored_cursor);
        if let Err(error) = result {
            if error != DecodeError::MemoryLimit {
                // C commits local state/color only at a normal chunk return.
                // Params and painted memory remain changed after a WASM trap.
                self.state = previous_state;
                self.color = previous_color;
                self.cursor = stored_cursor;
            }
        }
        result
    }

    fn decode_data<T: Copy + Into<u32>>(
        &mut self,
        data: &[T],
        stored_cursor: &mut i32,
    ) -> Result<(), DecodeError> {
        for &value in data {
            let byte = code(value);
            if byte.is_ascii_digit() {
                let last = &mut self.params[self.param_len - 1];
                *last = last.wrapping_mul(10).wrapping_add(i32::from(byte - b'0'));
                continue;
            }
            if (b'?'..=b'~').contains(&byte) {
                let bits = byte - b'?';
                if self.state == State::Repeat {
                    let count = if self.params[0] == 0 {
                        1
                    } else {
                        self.params[0]
                    };
                    if self.mode == SixelMode::Dynamic {
                        self.ensure_cleared(self.cursor.wrapping_add(count));
                    }
                    self.paint_repeat(bits, count as u32)?;
                    if self.mode == SixelMode::Dynamic {
                        self.band_bits |= bits;
                    }
                    self.cursor = self.cursor.wrapping_add(count);
                } else {
                    if self.state != State::Data {
                        self.apply_color()?;
                    }
                    if self.mode == SixelMode::Dynamic {
                        self.ensure_cleared(self.cursor);
                        self.band_bits |= bits;
                    }
                    self.paint_single(bits);
                    self.cursor = self.cursor.wrapping_add(1);
                }
                self.state = State::Data;
            } else {
                match byte {
                    b'!' | b'#' => {
                        if self.state == State::Color {
                            self.apply_color()?;
                        }
                        self.params[0] = 0;
                        self.param_len = 1;
                        self.state = if byte == b'!' {
                            State::Repeat
                        } else {
                            State::Color
                        };
                    }
                    b'$' => {
                        if self.mode == SixelMode::Dynamic {
                            self.current_width();
                        }
                        self.cursor = 4;
                    }
                    b'-' => {
                        if self.mode == SixelMode::Dynamic {
                            self.cursor = self.current_width() + 4;
                            *stored_cursor = self.cursor;
                        }
                        self.finish_band()?;
                        self.reset_band()?;
                        self.cursor = 4;
                    }
                    b';' => self.next_param(),
                    _ => {}
                }
            }
        }
        Ok(())
    }

    fn next_param(&mut self) {
        if self.param_len < self.params.len() {
            self.params[self.param_len] = 0;
            self.param_len += 1;
        }
    }

    fn scan_raster<T: Copy + Into<u32>>(&mut self, data: &[T]) {
        for &value in data {
            let byte = code(value);
            let starts_image =
                (b'?'..=b'~').contains(&byte) || matches!(byte, b'!' | b'#' | b'$' | b'-');
            if self.state == State::Data {
                if byte == b'"' {
                    self.params[0] = 0;
                    self.param_len = 1;
                    self.state = State::Raster;
                } else if starts_image {
                    self.level = SixelLevel::One;
                    self.mode = SixelMode::Dynamic;
                    self.raster = RasterAttributes::default();
                    return;
                }
            } else if self.state == State::Raster {
                if byte.is_ascii_digit() {
                    let last = &mut self.params[self.param_len - 1];
                    *last = last.wrapping_mul(10).wrapping_add(i32::from(byte - b'0'));
                } else if byte == b';' {
                    self.next_param();
                } else if self.param_len == 4 {
                    self.level = SixelLevel::Two;
                    self.mode = if self.truncate {
                        SixelMode::Truncated
                    } else {
                        SixelMode::Dynamic
                    };
                    self.raster = RasterAttributes {
                        numerator: self.params[0] as u32,
                        denominator: self.params[1] as u32,
                        width: self.params[2] as u32,
                        height: self.params[3] as u32,
                    };
                    self.state = State::Data;
                    return;
                }
                if starts_image {
                    self.level = SixelLevel::One;
                    self.mode = SixelMode::Dynamic;
                    self.raster = RasterAttributes {
                        numerator: self.params[0] as u32,
                        denominator: if self.param_len > 1 {
                            self.params[1] as u32
                        } else {
                            0
                        },
                        width: if self.param_len > 2 {
                            self.params[2] as u32
                        } else {
                            0
                        },
                        height: 0,
                    };
                    self.state = State::Data;
                    return;
                }
            }
        }
    }

    fn reset_band(&mut self) -> Result<(), DecodeError> {
        if self.mode == SixelMode::Dynamic {
            self.real_width.set(4);
            self.band_bits = 0;
            self.band[4..132].fill(self.fill);
            self.cleared_width = 4;
        } else {
            let stored_width = (self.raster.width as i32).min(MAX_WIDTH as i32) + 4;
            let count = ((stored_width - 3) / 2).max(0) as usize * 2;
            self.band[4..4 + count].fill(self.fill);
            // C computes the memcpy byte count in wrapping i32 arithmetic.
            // A negative width can wrap to a small valid copy instead of trap.
            let copy_words = (stored_width as u32).wrapping_mul(4) as usize / 4;
            for row in 1..6 {
                let destination = row * STRIDE + 4;
                if 4 + copy_words > self.band.len() || destination + copy_words > self.band.len() {
                    return Err(DecodeError::MemoryAccess);
                }
                self.band.copy_within(4..4 + copy_words, destination);
            }
        }
        Ok(())
    }

    fn ensure_cleared(&mut self, needed: i32) {
        while needed >= self.cleared_width as i32 && self.cleared_width < MAX_WIDTH {
            for row in 0..6 {
                let start = row * STRIDE + self.cleared_width;
                self.band[start..start + 128].fill(self.fill);
            }
            self.cleared_width += 128;
        }
    }

    fn paint_single(&mut self, bits: u8) {
        let cursor = self.cursor as u32;
        if cursor < MAX_WIDTH as u32 {
            for row in 0..6 {
                // Unset bits write the reserved prefix element in the source.
                let column = if bits & (1 << row) != 0 {
                    cursor as usize
                } else {
                    0
                };
                self.band[row * STRIDE + column] = self.color;
            }
        }
    }

    fn paint_repeat(&mut self, bits: u8, mut count: u32) -> Result<(), DecodeError> {
        let cursor = self.cursor as u32;
        if bits == 0 || cursor >= MAX_WIDTH as u32 {
            return Ok(());
        }
        if cursor.wrapping_add(count) >= MAX_WIDTH as u32 {
            count = MAX_WIDTH as u32 - cursor;
        }
        if count > MAX_WIDTH as u32 - cursor {
            // The first active plane fills all following WASM band memory
            // before trapping. Emulate its visible writes within bounded Vec
            // storage, never following the malicious count out of bounds.
            let first_row = bits.trailing_zeros() as usize;
            self.band[first_row * STRIDE + cursor as usize..].fill(self.color);
            return Err(DecodeError::MemoryAccess);
        }
        for row in 0..6 {
            if bits & (1 << row) != 0 {
                let start = row * STRIDE + cursor as usize;
                self.band[start..start + count as usize].fill(self.color);
            }
        }
        Ok(())
    }
}
